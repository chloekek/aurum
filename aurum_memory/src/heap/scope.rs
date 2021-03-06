use super::Heap;
use super::handle::ScopedHandle;
use super::handle::UnsafeHandle;

use alloc::vec;
use core::cell::Cell;
use core::iter::TrustedLen;
use core::mem::MaybeUninit;
use core::mem::transmute;
use scopeguard::defer;

impl<'h> Heap<'h>
{
    /// Shared implementation of `with_new_*_scope` methods.
    ///
    /// This method controls the lifetime of the [`Scope`] object,
    /// and makes sure that the scope is no longer used after `then` returns.
    fn with_scope<F, R>(&self, scope: &[Cell<UnsafeHandle<'h>>], then: F) -> R
        where F: FnOnce(&Scope<'h>) -> R
    {
        // SAFETY: We only borrow these for short periods of time.
        unsafe { self.scopes.borrow_mut() }.push(scope);
        defer! { unsafe { self.scopes.borrow_mut() }.pop(); }

        // SAFETY: The scope is registerd with the heap.
        let scope = unsafe { Scope::new(scope) };

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
    ///
    /// # Examples
    ///
    /// You can use destructuring syntax to obtain the handles:
    ///
    /// ```
    /// # use aurum_memory::heap::Heap;
    /// # use aurum_memory::object::DeBruijn;
    /// # Heap::with_new(|heap| {
    /// heap.with_new_array_scope(|[add, pi, x, app]| {
    ///     heap.new_symbol(add, b"Add").unwrap();
    ///     heap.new_symbol(pi, b"Pi").unwrap();
    ///     heap.new_variable(x, DeBruijn(0));
    ///     heap.new_application(app, add, [pi, x]).unwrap();
    /// });
    /// # });
    /// ```
    pub fn with_new_array_scope<F, R, const N: usize>(&self, then: F) -> R
        where F: for<'s> FnOnce([ScopedHandle<'h, 's>; N]) -> R
    {
        let scope = Cell::new([self.interned_null(); N]);
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
        let scope = vec![Cell::new(self.interned_null()); size];
        self.with_scope(&scope, then)
    }
}

/// Collection of handles to objects that will not be destroyed.
///
/// The `with_new_*_scope` methods on [`Heap`] create scopes on the call stack.
/// These methods will automatically register the scope with the heap,
/// and unregister it when it is no longer to be used.
/// This ensures that the garbage collector is aware of all scopes,
/// and that handles in a scope always point to existing objects.
///
/// Scopes can also be borrowed from pinned objects
/// that contain handles to other objects.
/// The parent object being pinned ensures that
/// the objects it points to won’t be destroyed.
/// An example of this is pinned application objects:
/// you can safely [borrow] their list of argument handles.
///
/// Objects referenced by any scope are not destroyed by the garbage collector.
/// Moreover, when the garbage collector relocates objects in memory,
/// it will make sure to update any handles to those objects in all scopes.
/// Thus, scopes provide a safe way to work with handles
/// in the presence of the garbage collector.
/// The [`ScopedHandle`] type encapsulates this.
///
/// [`Heap`]: `super::Heap`
/// [borrow]: `super::PinnedHandle::as_application`
#[repr(transparent)]
pub struct Scope<'h>
{
    handles: [Cell<UnsafeHandle<'h>>],
}

impl<'h> Scope<'h>
{
    /// Create a scope from a collection of handles.
    ///
    /// # Safety
    ///
    /// The objects referred to by the handles must not be destroyed
    /// for the entire lifetime of the scope.
    #[inline]
    pub unsafe fn new<'s>(handles: &'s [Cell<UnsafeHandle<'h>>]) -> &'s Self
    {
        // SAFETY: These types have the same representation.
        transmute(handles)
    }

    /// Retrieve the handle at the given index.
    ///
    /// If the index is out of bounds, this method returns [`None`].
    #[inline]
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
    #[inline]
    pub unsafe fn get_unchecked<'s>(&'s self, index: usize)
        -> ScopedHandle<'h, 's>
    {
        let handle = self.handles.get_unchecked(index);
        // SAFETY: The handle is part of this scope.
        ScopedHandle::new(handle)
    }

    /// Iterator over the handles in this scope.
    #[inline]
    pub fn iter<'s>(&'s self)
        -> impl Iterator<Item=ScopedHandle<'h, 's>>
                + ExactSizeIterator
                + TrustedLen
    {
        // SAFETY: The handle is part of this scope.
        self.handles.iter().map(|h| unsafe { ScopedHandle::new(h) })
    }
}
