use crate::heap::UnsafeHandle;
use super::DeBruijn;
use super::FreeCache;
use super::Kind;

use core::mem::MaybeUninit;

/// Variable objects always have one free variable: themselves.
fn free_cache(de_bruijn: DeBruijn) -> FreeCache
{
    FreeCache::EMPTY.insert(de_bruijn)
}

/// The De Bruijn index is stored in the extra field.
fn extra(de_bruijn: DeBruijn) -> [MaybeUninit<u8>; 4]
{
    let mut result = MaybeUninit::uninit_array();
    MaybeUninit::write_slice(&mut result, &de_bruijn.0.to_ne_bytes());
    result
}

/// Variables store all their information in the extra field,
/// so they have a payload size of zero.
const PAYLOAD_SIZE: usize = 0;

alloc_methods!
{
    #![doc = "variables"]

    #[doc = "Create a variable with the given De Bruijn index."]
    #[doc = ""]
    #[doc = "If the De Bruijn index is sufficiently small,"]
    #[doc = "an interned object is returned and no allocation takes place."]
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

    #[doc = "Create a variable with the given De Bruijn index."]
    #[scoped_alias = new_variable_not_interned]
    pub fn alloc_variable_not_interned(&self, de_bruijn: DeBruijn)
        -> Result<UnsafeHandle<'h>, !>
    {
        let handle = unsafe {
            self.alloc(
                Kind::Variable,
                free_cache(de_bruijn),
                extra(de_bruijn),
                PAYLOAD_SIZE,
                |_payload| { },
            )
        };
        Ok(handle)
    }
}
