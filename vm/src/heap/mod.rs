//! Memory allocation and garbage collection.

pub use self::handle::*;
pub use self::heap::*;
pub use self::scope::*;

// The order of these declarations influences
// the order of the Heap impls in in rustdoc.
mod heap;
mod scope;
mod alloc;

mod handle;
