//! Memory allocation and garbage collection.
//!
//! # Handles
//!
//! A handle is simply a non-null pointer to an object.
//! It also provides interior mutability, as objects are mutable.
//! There are different types of handles.
//! They assume different conditions and provide different sets of features.
//! These conditions are features are summarized in the table below.
//!
//! | Handle type      | Conditions (cumulative)   | Provides (cumulative)                 |
//! |------------------|---------------------------|---------------------------------------|
//! | [`UnsafeHandle`] | None                      | Unsafe mutable access to the object   |
//! | [`ScopedHandle`] | Object won’t be destroyed | Safe copying of parts of the object   |
//! | [`PinnedHandle`] | Object won’t be relocated | Safe borrowing of parts of the object |

pub use self::handle::*;
pub use self::heap::*;
pub use self::scope::*;

// The order of these declarations influences
// the order of the Heap impls in in rustdoc.
mod heap;
mod scope;
mod alloc;

mod handle;
