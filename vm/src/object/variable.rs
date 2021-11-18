use crate::heap::Heap;
use crate::heap::ScopedHandle;
use super::DeBruijn;
use super::Kind;

use core::mem::MaybeUninit;

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
        // There is no payload for variables.
        // All information is stored in the header.
        const PAYLOAD_SIZE: usize = 0;
        unsafe {
            self.new(
                into,
                Kind::Variable,
                PAYLOAD_SIZE,
                |free_cache, extra, _payload| {

                    // The free cache stores just this variable.
                    *free_cache = free_cache.insert(de_bruijn);

                    // The De Bruijn index is stored in the extra field.
                    MaybeUninit::write_slice(extra, &de_bruijn.0.to_ne_bytes());

                    // The payload remains empty.
                    { }

                },
            );
        }
    }
}
