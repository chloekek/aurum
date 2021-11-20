use crate::heap::Heap;
use crate::heap::ScopedHandle;
use super::DeBruijn;
use super::Flags;
use super::FreeCache;
use super::Header;
use super::Kind;

use core::mem::MaybeUninit;

// Variables store all info in the header.
const PAYLOAD_SIZE: usize = 0;

/// Methods for creating variable objects.
impl<'h> Heap<'h>
{
    /// Create a variable with the given De Bruijn index.
    ///
    /// If the De Bruijn index is sufficiently small,
    /// an interned object is returned and no allocation takes place.
    pub fn new_variable<'s>(
        &self,
        into: ScopedHandle<'h, 's>,
        de_bruijn: DeBruijn,
    )
    {
        let interned = self.interned_variable(de_bruijn);
        match interned {
            Some(result) => unsafe { into.copy_from_unsafe_handle(result) },
            None => self.new_variable_not_interned(into, de_bruijn),
        }
    }

    /// Create a variable with the given De Bruijn index.
    pub fn new_variable_not_interned<'s>(
        &self,
        into: ScopedHandle<'h, 's>,
        de_bruijn: DeBruijn,
    )
    {
        unsafe {
            self.new(into, PAYLOAD_SIZE, |_payload| {

                // The De Bruijn index is stored in the extra field.
                let mut extra = MaybeUninit::uninit_array();
                let extra_bytes = de_bruijn.0.to_ne_bytes();
                MaybeUninit::write_slice(&mut extra, &extra_bytes);

                // The variable appears free in itself.
                let free_cache = FreeCache::EMPTY.insert(de_bruijn);

                Header{
                    kind: Kind::Variable,
                    flags: Flags::empty(),
                    free_cache,
                    extra,
                }

            });
        }
    }
}

/// Methods for inspecting variable objects.
impl<'h, 's> ScopedHandle<'h, 's>
{
    /// Get the De Bruijn index of the variable object.
    ///
    /// If the object is not a variable, this method returns [`None`].
    pub fn as_variable(self) -> Option<DeBruijn>
    {
        let header = self.header();
        match header.kind {
            Kind::Variable => {
                let extra = header.extra;
                let extra = unsafe { MaybeUninit::array_assume_init(extra) };
                Some(DeBruijn(u32::from_ne_bytes(extra)))
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    use proptest::proptest;

    proptest!
    {
        #[test]
        fn roundtrip(de_bruijn: u32)
        {
            let de_bruijn = DeBruijn(de_bruijn);
            Heap::with_new(|heap| {
                heap.with_new_array_scope(|[handle]| {
                    heap.new_variable(handle, de_bruijn);
                    assert_eq!(handle.as_variable(), Some(de_bruijn));
                });
            });
        }
    }
}
