use crate::heap::ScopedHandle;
use crate::heap::UnsafeHandle;
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

alloc_methods!
{
    //! applications

    /// Create an application with the given function and arguments.
    #[scoped_alias = new_application]
    pub fn alloc_application<'s>(
        &self,
        function: ScopedHandle<'h, 's>,
        arguments: impl ExactSizeIterator<Item=ScopedHandle<'h, 's>> + TrustedLen,
    ) -> Result<UnsafeHandle<'h>, NumArgumentsError>
    {
        let payload_size = payload_size(arguments.len())?;
        let handle = unsafe {
            self.alloc(
                Kind::Application,
                payload_size as usize,
                |free_cache, extra, payload| {

                    // The extra field stores the number of fields,
                    // which is 1 (for the function) + the number of arguments.
                    let num_fields = payload_size / PTR_SIZE;
                    MaybeUninit::write_slice(extra, &num_fields.to_ne_bytes());

                    // The payload first stores the function,
                    // then all the arguments in order.
                    // Update the free cache while at it.
                    let payload = payload as *mut UnsafeHandle;
                    let fields = iter::once(function).chain(arguments);
                    for (i, field) in fields.enumerate() {
                        *free_cache |= field.free_cache();
                        *payload.add(i) = field.as_unsafe_handle();
                    }

                },
            )
        };
        Ok(handle)
    }
}
