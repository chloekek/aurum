use crate::heap::Heap;
use crate::heap::ScopedHandle;
use crate::heap::UnsafeHandle;
use super::Flags;
use super::FreeCache;
use super::Header;
use super::Kind;

use core::iter::TrustedLen;
use core::iter;
use core::mem::MaybeUninit;
use core::mem::size_of;

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
                let payload = payload as *mut UnsafeHandle;
                let fields = iter::once(function).chain(arguments);
                for (i, field) in fields.enumerate() {
                    free_cache |= field.header().free_cache;
                    *payload.add(i) = field.as_unsafe_handle();
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
