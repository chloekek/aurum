//! In-memory representation of objects.

#[macro_use] mod macros;

pub use self::de_bruijn::*;
pub use self::symbol::*;
pub use self::variable::*;

use crate::heap::HeapId;

use bitflags::bitflags;
use core::mem::MaybeUninit;

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
    Indirection,
    Symbol,
    Variable,
}

bitflags!
{
    pub struct Flags: u8
    {
        const MARKED = 1 << 0;
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
