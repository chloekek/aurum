use crate::heap::Heap;
use crate::heap::PinnedHandle;
use crate::heap::Scope;
use crate::heap::ScopedHandle;
use crate::heap::UnsafeHandle;
use super::Flags;
use super::FreeCache;
use super::Header;
use super::Kind;

use core::cell::Cell;
use core::iter::TrustedLen;
use core::iter;
use core::mem::MaybeUninit;
use core::mem::size_of;
use core::slice;

/// Raised when attempting to create an application with too many arguments.
#[derive(Debug)]
pub struct NumArgumentsError;

/// Convenient constant for computing payload size.
const PTR_SIZE: u32 = size_of::<UnsafeHandle>() as u32;

/// The payload stores the function and the arguments.
fn payload_size(num_arguments: usize) -> Result<u32, NumArgumentsError>
{
    const ERR: NumArgumentsError = NumArgumentsError;
    let as_u32: u32 = num_arguments.try_into().map_err(|_| ERR)?;
    let num_fields = as_u32.checked_add(1).ok_or(ERR)?;
    num_fields.checked_mul(PTR_SIZE).ok_or(ERR)
}

/// Methods for creating application objects.
impl<'h> Heap<'h>
{
    /// Create an application with the given function and arguments.
    pub fn new_application<'s, I>(
        &self,
        into: ScopedHandle<'h, 's>,
        function: ScopedHandle<'h, 's>,
        arguments: impl IntoIterator<IntoIter=I>,
    ) -> Result<(), NumArgumentsError>
        where I: ExactSizeIterator<Item=ScopedHandle<'h, 's>> + TrustedLen
    {
        let arguments = arguments.into_iter();
        let payload_size = payload_size(arguments.len())?;

        unsafe {
            self.new(into, payload_size as usize, |payload| {

                // The extra field stores the number of fields,
                // which is 1 (for the function) + the number of arguments.
                let num_fields = payload_size / PTR_SIZE;
                let mut extra = MaybeUninit::uninit_array();
                MaybeUninit::write_slice(&mut extra, &num_fields.to_ne_bytes());

                // The payload first stores the function,
                // then all the arguments in order.
                let mut free_cache = FreeCache::EMPTY;
                let payload = payload as *mut Cell<UnsafeHandle>;
                let fields = iter::once(function).chain(arguments);
                for (i, field) in fields.enumerate() {
                    free_cache |= field.header().free_cache;
                    *payload.add(i) = Cell::new(field.as_unsafe_handle());
                }

                Header{
                    kind: Kind::Application,
                    flags: Flags::empty(),
                    free_cache,
                    extra,
                }

            });
        }

        Ok(())
    }
}

/// Methods for inspecting application objects.
impl<'h, 'p> PinnedHandle<'h, 'p>
{
    /// Get the function and the arguments of the application object.
    ///
    /// If the object is not an application, this method returns [`None`].
    pub fn as_application(self) -> Option<(ScopedHandle<'h, 'p>, &'p Scope<'h>)>
    {
        let header = self.header();
        match header.kind {
            Kind::Application => {
                let extra = header.extra;
                let extra = unsafe { MaybeUninit::array_assume_init(extra) };
                let num_fields = u32::from_ne_bytes(extra);
                let fields = self.payload() as *const Cell<UnsafeHandle>;

                // SAFETY: There’s always at least one field.
                let function = unsafe { &*fields };
                let arguments = unsafe {
                    slice::from_raw_parts(
                        fields.add(1),
                        num_fields as usize - 1,
                    )
                };

                // SAFETY: The handles reside in a pinned object.
                let function = unsafe { ScopedHandle::new(function) };
                let arguments = unsafe { Scope::new(arguments) };

                Some((function, arguments))
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use super::super::DeBruijn;

    use proptest::arbitrary::any as pany;
    use proptest::collection::vec as pvec;
    use proptest::proptest;

    proptest!
    {
        #[test]
        fn roundtrip(
            function_db: u32,
            argument_db in pvec(pany::<u32>(), 0 .. 32),
        )
        {
            Heap::with_new(|heap| {
            heap.with_new_array_scope(|[application, function]| {
            heap.with_new_boxed_scope(argument_db.len(), |arguments| {

                // Create the function object.
                heap.new_variable(function, DeBruijn(function_db));

                // Create the argument objects.
                for (&a, h) in argument_db.iter().zip(arguments.iter()) {
                    heap.new_variable(h, DeBruijn(a));
                }

                // Create the application object.
                heap.new_application(application, function, arguments.iter())
                    .unwrap();

                // Check that the correct object was created.
                application.with_pin(|application| {
                    let result = application.as_application().unwrap();
                    assert_eq!(
                        result.0.as_unsafe_handle(),
                        function.as_unsafe_handle(),
                    );
                    assert!(
                        Iterator::eq(
                            result.1.iter().map(|sh| sh.as_unsafe_handle()),
                            arguments.iter().map(|sh| sh.as_unsafe_handle()),
                        )
                    );
                });

            }); }); });
        }
    }
}
