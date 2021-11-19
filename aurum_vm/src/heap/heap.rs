use super::UnsafeHandle;
use crate::object::DeBruijn;

use alloc::vec::Vec;
use unsafe_ref_cell::UnsafeRefCell;
use core::cell::Cell;
use core::marker::PhantomData;

const INTERNED_VARIABLE_COUNT: usize = 16;

/// Uniquely identifies a heap at compile-time.
///
/// By abusing an invariant lifetime,
/// we can distinguish different heaps at the type level.
/// This prevents us from mixing up objects from different heaps,
/// which is important for the garbage collector to work safely.
pub type HeapId<'h> = PhantomData<fn(&'h ()) -> &'h ()>;

/// Collection of objects that may point to each other.
pub struct Heap<'h>
{
    /// Uniquely identifies this heap.
    heap_id: HeapId<'h>,

    /// Stack of scopes managed by `with_scope`.
    /// It is important that the stack is managed *only* by `with_scope`,
    /// as the push and pop must happen in the same order
    /// as scope creation and destruction.
    pub (super) scopes: UnsafeRefCell<Vec<*const [Cell<UnsafeHandle<'h>>]>>,

    /// See the corresponding methods for more information.
    interned_null: Cell<UnsafeHandle<'h>>,
    interned_variables: Cell<[UnsafeHandle<'h>; INTERNED_VARIABLE_COUNT]>,
}

impl<'h> Heap<'h>
{
    // Looking for the methods that create objects of different kinds?
    // Those can be found in the `crate::object::*` modules.

    /// Create a new heap and pass it to the given function.
    ///
    /// This method passes the heap to a callback rather than returning it,
    /// as the choice of `'h` must be up to this method and not the caller,
    /// to ensure that the heap identifier is unique.
    pub fn with_new<F, R>(then: F) -> R
        where F: for<'fresh_h> FnOnce(&Heap<'fresh_h>) -> R
    {
        // Create the heap.
        let this = Heap{

            heap_id: PhantomData,
            scopes: UnsafeRefCell::new(Vec::new()),

            // These will be initialized below.
            interned_null: Cell::new(UnsafeHandle::dangling()),
            interned_variables: Cell::new([UnsafeHandle::dangling(); 16]),

        };

        this.with_new_array_scope(|[scoped]| {

            // TODO: Make sure no GC takes place until
            //       interned fields have been initialized

            // Initialize the interned null object.
            this.new_symbol(scoped, b"Null").unwrap();
            this.interned_null.set(scoped.as_unsafe_handle());

            // Initialize the interned variable objects.
            for i in 0 .. INTERNED_VARIABLE_COUNT {
                let de_bruijn = DeBruijn(i as u32);
                this.new_variable_not_interned(scoped, de_bruijn);
                this.interned_variables.as_array_of_cells()[i]
                    .set(scoped.as_unsafe_handle());
            }

        });

        // Call the continuation.
        then(&this)
    }

    /// Interned Null object.
    ///
    /// This handle is used to initialize new scopes;
    /// every scope starts out with all Null handles.
    pub fn interned_null(&self) -> UnsafeHandle<'h>
    {
        self.interned_null.get()
    }

    /// Interned variable objects with small De Bruijn indices.
    ///
    /// The [`new_variable`][`Heap::new_variable`]
    /// method automatically consults this array.
    pub fn interned_variable(&self, de_bruijn: DeBruijn)
        -> Option<UnsafeHandle<'h>>
    {
        self.interned_variables.as_array_of_cells()
            .get(de_bruijn.0 as usize)
            .map(Cell::get)
    }
}
