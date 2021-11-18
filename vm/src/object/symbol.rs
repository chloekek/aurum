use crate::heap::UnsafeHandle;
use super::FreeCache;
use super::Kind;

use core::mem::MaybeUninit;
use core::slice;

/// Raised when attempting to create a symbol with a name that is too long.
#[derive(Debug)]
pub struct SymbolLenError;

/// Symbols do not have free variables.
const FREE_CACHE: FreeCache = FreeCache::EMPTY;

/// The length of the symbol’s name is stored in the extra field.
fn extra(name_len: u32) -> [MaybeUninit<u8>; 4]
{
    let mut result = MaybeUninit::uninit_array();
    MaybeUninit::write_slice(&mut result, &name_len.to_ne_bytes());
    result
}

/// The payload stores just the name of the symbol.
fn payload_size(name_len: u32) -> usize
{
    name_len as usize
}

alloc_methods!
{
    //! symbols

    /// Create a symbol with the given name.
    #[scoped_alias = new_symbol]
    pub fn alloc_symbol(&self, name: &[u8])
        -> Result<UnsafeHandle<'h>, SymbolLenError>
    {
        let name_len: u32 = name.len().try_into().map_err(|_| SymbolLenError)?;
        let handle = unsafe {
            self.alloc(
                Kind::Symbol,
                FREE_CACHE,
                extra(name_len),
                payload_size(name_len),
                |payload| {
                    MaybeUninit::write_slice(
                        slice::from_raw_parts_mut(
                            payload as *mut MaybeUninit<u8>,
                            name.len(),
                        ),
                        name
                    );
                },
            )
        };
        Ok(handle)
    }
}
