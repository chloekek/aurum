use crate::heap::UnsafeHandle;
use super::DeBruijn;
use super::Kind;

use core::mem::MaybeUninit;

alloc_methods!
{
    //! variables

    /// Create a variable with the given De Bruijn index.
    ///
    /// If the De Bruijn index is sufficiently small,
    /// an interned object is returned and no allocation takes place.
    #[scoped_alias = new_variable]
    pub fn alloc_variable(&self, de_bruijn: DeBruijn)
        -> Result<UnsafeHandle<'h>, !>
    {
        let interned = self.interned_variables.get(de_bruijn.0 as usize);
        match interned {
            Some(&result) => Ok(result),
            None => self.alloc_variable_not_interned(de_bruijn),
        }
    }

    /// Create a variable with the given De Bruijn index.
    #[scoped_alias = new_variable_not_interned]
    pub fn alloc_variable_not_interned(&self, de_bruijn: DeBruijn)
        -> Result<UnsafeHandle<'h>, !>
    {
        // There is no payload for variables.
        // All information is stored in the header.
        const PAYLOAD_SIZE: usize = 0;

        let handle = unsafe {
            self.alloc(
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
            )
        };

        Ok(handle)
    }
}
