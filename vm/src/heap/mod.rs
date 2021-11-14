//! Memory allocation and garbage collection.

pub use self::handle::*;
pub use self::heap::*;

mod handle;
mod heap;
