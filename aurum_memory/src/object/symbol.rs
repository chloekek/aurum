use crate::heap::Heap;
use crate::heap::PinnedHandle;
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
    #[inline]
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

/// Methods for inspecting symbol objects.
impl<'h, 'p> PinnedHandle<'h, 'p>
{
    /// Get the name of the symbol object.
    ///
    /// If the object is not a symbol, this method returns [`None`].
    #[inline]
    pub fn as_symbol(self) -> Option<&'p [u8]>
    {
        let header = self.header();
        match header.kind {
            Kind::Symbol => {
                let extra = header.extra;
                let extra = unsafe { MaybeUninit::array_assume_init(extra) };
                let name_len = u32::from_ne_bytes(extra);
                let payload = self.payload();
                let name = unsafe {
                    slice::from_raw_parts(
                        payload as *const u8,
                        name_len as usize,
                    )
                };
                Some(name)
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    use alloc::vec::Vec;
    use proptest::proptest;

    proptest!
    {
        #[test]
        fn roundtrip(name: Vec<u8>)
        {
            Heap::with_new(|heap| {
                heap.with_new_array_scope(|[handle]| {
                    heap.new_symbol(handle, &name).unwrap();
                    handle.with_pin(|handle| {
                        assert_eq!(handle.as_symbol(), Some(name.as_ref()));
                    });
                });
            });
        }
    }
}
