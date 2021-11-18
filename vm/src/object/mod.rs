//! In-memory representation of objects.

pub use self::de_bruijn::*;
pub use self::symbol::*;
pub use self::variable::*;

use crate::heap::HeapId;

use bitflags::bitflags;
use core::mem::MaybeUninit;

mod application;
mod de_bruijn;
mod symbol;
mod variable;

/// In-memory representation of an object.
#[repr(C, align(8))]
pub struct Object<'h>
{
    pub heap_id: HeapId<'h>,
    pub header: Header,
    pub payload: Payload,
}

/// Metadata at the very start of each object.
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct Header
{
    pub kind: Kind,
    pub flags: Flags,
    pub free_cache: FreeCache,
    pub extra: [MaybeUninit<u8>; 4],
}

/// Determines the types of the extra and payload fields of the object.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum Kind
{
    Symbol,
    Variable,
    Application,
}

bitflags!
{
    /// Various flags that an object may have.
    pub struct Flags: u8
    {
        /// Used during a garbage collection cycle.
        ///
        /// Specifically, set on objects that are reachable from roots.
        /// Does not persist outside of garbage collection cycles.
        /// The alternative to storing this in the objects would be
        /// to keep a set of pointers to objects during garbage collection.
        /// However, such an approach would require _Ω(n)_ extra memory
        /// where _n_ is the number of roots.
        const MARKED = 1 << 0;

        /// As long as an object has this flag,
        /// the garbage collector will not
        /// destroy or relocate the object.
        const PINNED = 1 << 1;
    }
}

extern
{
    /// Placeholder for the different payload types of an object.
    pub type Payload;
}

#[cfg(test)]
mod tests
{
    use super::*;

    use core::mem::size_of;

    #[test]
    fn header_size()
    {
        assert_eq!(size_of::<Header>(), 8);
    }
}
