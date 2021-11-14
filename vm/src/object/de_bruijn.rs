/// A De Bruijn index references a variable.
#[derive(Clone, Copy)]
pub struct DeBruijn(pub u32);

/// Set of variables that are free in an object.
///
/// Each object embeds a free variables cache.
/// The purpose of a free variables cache is to quickly answer the question:
/// “does a particular variable appear free in the object?”
/// Variables with a De Bruijn index less than 8 can be stored in the cache.
/// Attempts to insert variables with higher De Bruijn indices
/// causes the set to enter an “unknown” state,
/// from which it can no longer answer the question.
///
/// Unless the cache is in the “unknown” state,
/// it _must_ accurately reflect the object.
/// There must not be false positives or false negatives.
/// This also holds for variables with De Bruijn indices larger than 7.
/// Without this invariant, the simplifier would produce incorrect results.
///
/// The free variables cache being in the “unknown” state
/// may not be taken as proof that it contains variables
/// with De Bruijn indices larger than 7.
/// This allows the simplifier to skip computing the free variables cache
/// when it is not needed or when it is too expensive to compute it
/// (it can simply use [`UNKNOWN`](`FreeCache::UNKNOWN`) directly).
#[derive(Clone, Copy)]
pub struct FreeCache
{
    /// The “unknown” state is represented by 0xFF.
    bits: u8,
}

impl FreeCache
{
    /// The free variables cache with no variables.
    pub const EMPTY: Self = Self{bits: 0};

    /// The free variables cache in the “unknown” state.
    pub const UNKNOWN: Self = Self{bits: 0xFF};

    /// Insert a variable into the free variables cache.
    ///
    /// If the variable is already in the set, the set is returned unchanged.
    /// If the variable has a De Bruijn index larger than 7,
    /// the free variables cache in the “unknown” state is returned.
    #[must_use = "insert returns a new free variables cache"]
    pub fn insert(self, de_bruijn: DeBruijn) -> Self
    {
        if de_bruijn.0 >= 8 {
            Self::UNKNOWN
        } else {
            Self{bits: self.bits | 1 << de_bruijn.0}
        }
    }

    /// Check whether the free variables cache contains a variable.
    ///
    /// If the free variables cache is in the “unknown” state,
    /// this method returns [`None`].
    /// Otherwise this function returns true or false
    /// depending on whether the variable is in the cache.
    pub fn contains(self, de_bruijn: DeBruijn) -> Option<bool>
    {
        if self.bits == Self::UNKNOWN.bits {
            None
        } else if de_bruijn.0 >= 8 {
            Some(false)
        } else {
            Some(self.bits & 1 << de_bruijn.0 != 0)
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    use alloc::format;
    use proptest::prop_assume;
    use proptest::proptest;

    proptest!
    {
        #[test]
        fn free_cache_answers_true(de_bruijn in 0u32 .. 7)
        {
            let de_bruijn = DeBruijn(de_bruijn);
            let cache = FreeCache::EMPTY.insert(de_bruijn);
            assert_eq!(cache.contains(de_bruijn), Some(true));
        }

        #[test]
        fn free_cache_answers_false(insert in 0u32 .. 7, check: u32)
        {
            prop_assume!(insert != check);
            let insert = DeBruijn(insert);
            let check = DeBruijn(check);
            let cache = FreeCache::EMPTY.insert(insert);
            assert_eq!(cache.contains(check), Some(false));
        }

        #[test]
        fn free_cache_answers_none(de_bruijn: u32)
        {
            let de_bruijn = DeBruijn(de_bruijn);
            assert_eq!(FreeCache::UNKNOWN.contains(de_bruijn), None);
        }

        #[test]
        fn free_cache_becomes_unknown(de_bruijn in 8u32 ..)
        {
            let de_bruijn = DeBruijn(de_bruijn);
            let cache = FreeCache::EMPTY.insert(de_bruijn);
            assert_eq!(cache.contains(de_bruijn), None);
        }
    }
}
