use crate::heap::Heap;
use crate::heap::ScopedHandle;
use super::Flags;
use super::FreeCache;
use super::Header;
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
            self.new(into, payload_size, |payload| {

                // The extra field stores the length of the name.
                let mut extra = MaybeUninit::uninit_array();
                MaybeUninit::write_slice(&mut extra, &name_len.to_ne_bytes());

                // The payload stores the bytes of the name.
                MaybeUninit::write_slice(
                    slice::from_raw_parts_mut(
                        payload as *mut MaybeUninit<u8>,
                        name.len(),
                    ),
                    name
                );

                Header{
                    kind: Kind::Symbol,
                    flags: Flags::empty(),
                    free_cache: FreeCache::EMPTY,
                    extra,
                }

            });
        }

        Ok(())
    }
}
