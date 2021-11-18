use crate::object;
use super::ScopedHandle;
use super::UnsafeHandle;

use alloc::alloc::alloc;
use alloc::alloc::handle_alloc_error;
use alloc::vec::Vec;
use alloc::vec;
use core::alloc::Layout;
use core::cell::Cell;
use core::cell::RefCell;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::mem::transmute;
use core::ptr::NonNull;
use scopeguard::defer;

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
    scopes: RefCell<Vec<*const [Cell<UnsafeHandle<'h>>]>>,

    /// Interned Null object.
    ///
    /// This handle is used to initialize new scopes;
    /// every scope starts out with all Null handles.
    pub interned_null: UnsafeHandle<'h>,

    /// Interned variable objects with small De Bruijn indices.
    ///
    /// The [`alloc_variable`][`Heap::alloc_variable`]
    /// and [`new_variable`][`Heap::new_variable`]
    /// methods automatically consult this array.
    pub interned_variables: [UnsafeHandle<'h>; INTERNED_VARIABLE_COUNT],
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
        let mut this = Heap{

            heap_id: PhantomData,
            scopes: RefCell::new(Vec::new()),

            // These will be initialized below.
            interned_null: UnsafeHandle::dangling(),
            interned_variables: [UnsafeHandle::dangling(); 16],

        };

        // TODO: Make sure no GC takes place until
        //       interned fields have been initialized

        // Initialize the interned null object.
        this.interned_null = this.alloc_symbol(b"Null").unwrap();

        // Initialize the interned variable objects.
        for i in 0 .. INTERNED_VARIABLE_COUNT {
            let de_bruijn = object::DeBruijn(i as u32);
            let object = this.alloc_variable_not_interned(de_bruijn);
            this.interned_variables[i] = object.into_ok();
        }

        // Call the continuation.
        then(&this)
    }

    /// Allocate memory for an object and initialize it.
    ///
    /// Memory is allocated on the garbage collected heap
    /// for an object of the given payload size.
    /// The kind field of the object is initialized
    /// and then the `init` function is called.
    ///
    /// You would not normally use this method.
    /// Instead use one of the `alloc_*` methods,
    /// or better yet, one of the `new_*` methods.
    /// They will initialize the object for you
    /// and are therefore much safer to use.
    ///
    /// # Safety
    ///
    /// The payload size must not be too large so as to
    /// overflow a [usize] when the object header size is added.
    ///
    /// Several conditions must hold regarding the `init` function:
    ///
    ///  - It must not call this method, even indirectly.
    ///  - It must not panic (but may abort the process).
    ///  - Once it returns, the object must be properly initialized
    ///    (when the garbage collector kicks in later,
    ///     it must not find an improperly initialized object).
    pub unsafe fn alloc(
        &self,
        kind:         object::Kind,
        payload_size: usize,
        init: impl FnOnce(
            &mut object::FreeCache,
            &mut [MaybeUninit<u8>; 4],
            *mut object::Payload,
        ),
    ) -> UnsafeHandle<'h>
    {
        // TODO: Replace this with a pointer bump allocation.

        let flags = object::Flags::empty();
        let free_cache = object::FreeCache::EMPTY;
        let extra = MaybeUninit::uninit_array();

        let layout = Layout::from_size_align_unchecked(8 + payload_size, 8);
        let pointer = alloc(layout);

        if pointer.is_null() {
            handle_alloc_error(layout);
        }

        let pointer = pointer as *mut object::Object<'h>;

        (*pointer).header = object::Header{kind, flags, free_cache, extra};
        init(
            &mut (*pointer).header.free_cache,
            &mut (*pointer).header.extra,
            &mut (*pointer).payload,
        );

        let pointer = pointer as *mut UnsafeCell<object::Object<'h>>;
        let pointer = NonNull::new_unchecked(pointer);
        UnsafeHandle::new(pointer)
    }

    /// Shared implementation of `with_new_*_scope` methods.
    ///
    /// This method controls the lifetime of the [`Scope`] object,
    /// and makes sure that the scope is no longer used after `then` returns.
    fn with_scope<F, R>(&self, scope: &[Cell<UnsafeHandle<'h>>], then: F) -> R
        where F: FnOnce(&Scope<'h>) -> R
    {
        self.scopes.borrow_mut().push(scope);
        defer! { self.scopes.borrow_mut().pop(); }

        // SAFETY: The scope is registerd with the heap.
        // SAFETY: These types have the same representation.
        let scope = unsafe { transmute(scope) };

        then(scope)
    }

    /// Create a new scope on the stack and pass it to the given function.
    ///
    /// The scope is destroyed as soon as the given function returns or panics.
    /// For more information about scopes, see [`Scope`].
    ///
    /// Since the number of handles is known statically (by `N`),
    /// this function will create all the scoped handles for you,
    /// so you don’t need to [`Scope::get`] them yourself.
    pub fn with_new_array_scope<F, R, const N: usize>(&self, then: F) -> R
        where F: for<'s> FnOnce([ScopedHandle<'h, 's>; N]) -> R
    {
        let scope = Cell::new([self.interned_null; N]);
        let scope = scope.as_array_of_cells();

        self.with_scope(scope, |scope| {

            let mut scoped_handles = MaybeUninit::uninit_array::<N>();

            for (i, s) in scoped_handles.iter_mut().enumerate() {
                // SAFETY: We created a scope with N handle slots.
                s.write(unsafe { scope.get_unchecked(i) });
            }

            // SAFETY: We initialized all N elements of the array.
            let scoped_handles = unsafe {
                MaybeUninit::array_assume_init(scoped_handles)
            };

            then(scoped_handles)

        })
    }

    /// Create a new scope on the heap and pass it to the given function.
    ///
    /// The scope is destroyed as soon as the given function returns or panics.
    /// For more information about scopes, see [`Scope`].
    pub fn with_new_boxed_scope<F, R>(&self, size: usize, then: F) -> R
        where F: FnOnce(&Scope<'h>) -> R
    {
        let scope = vec![Cell::new(self.interned_null); size];
        self.with_scope(&scope, then)
    }
}

/// Collection of handles that are treated as roots.
///
/// The only way to obtain a scope is by calling
/// one of the `with_new_*_scope` methods on [`Heap`].
/// These methods will automatically register the scope with the heap,
/// and unregister it when it is no longer to be used.
/// This ensures that the garbage collector is aware of all scopes,
/// and that handles in a scope always point to existing objects.
///
/// Objects referenced by any scope are not destroyed by the garbage collector.
/// Moreover, when the garbage collector relocates objects in memory,
/// it will make sure to update any handles to those objects in all scopes.
/// Thus, scopes provide a safe way to work with handles
/// in the presence of the garbage collector.
/// The [`ScopedHandle`] type encapsulates this.
///
/// [`Heap`]: `super::Heap`
#[repr(transparent)]
pub struct Scope<'h>
{
    handles: [Cell<UnsafeHandle<'h>>],
}

impl<'h> Scope<'h>
{
    /// Retrieve the handle at the given index.
    ///
    /// If the index is out of bounds, this method returns [`None`].
    pub fn get<'s>(&'s self, index: usize) -> Option<ScopedHandle<'h, 's>>
    {
        match self.handles.get(index) {
            // SAFETY: The handle is part of this scope.
            Some(h) => unsafe { Some(ScopedHandle::new(h)) },
            None => None,
        }
    }

    /// Retrieve the handle at the given index.
    ///
    /// # Safety
    ///
    /// If the index is out of bounds, the behavior is undefined.
    pub unsafe fn get_unchecked<'s>(&'s self, index: usize)
        -> ScopedHandle<'h, 's>
    {
        let handle = self.handles.get_unchecked(index);
        // SAFETY: The handle is part of this scope.
        ScopedHandle::new(handle)
    }
 }
