use crate::heap::Heap;
use crate::heap::ScopedHandle;
use super::Kind;

use core::mem::MaybeUninit;
use core::slice;

/// Raised when attempting to create a symbol with a name that is too long.
#[derive(Debug)]
pub struct SymbolLenError;

/// Methods for creating symbol objects.
impl<'h> Heap<'h>
{
    /// Create a symbol with the given name.
    pub fn new_symbol<'s>(&self, into: ScopedHandle<'h, 's>, name: &[u8])
        -> Result<(), SymbolLenError>
    {
        const ERR: SymbolLenError = SymbolLenError;
        let payload_size = name.len();
        let name_len: u32 = name.len().try_into().map_err(|_| ERR)?;
        unsafe {
            self.new(
                into,
                Kind::Symbol,
                payload_size,
                |_free_cache, extra, payload| {

                    // The free cache remains empty.
                    { }

                    // The extra field stores the length of the name.
                    MaybeUninit::write_slice(extra, &name_len.to_ne_bytes());

                    // The payload stores the bytes of the name.
                    MaybeUninit::write_slice(
                        slice::from_raw_parts_mut(
                            payload as *mut MaybeUninit<u8>,
                            name.len(),
                        ),
                        name
                    );

                },
            );
        }
        Ok(())
    }
}
