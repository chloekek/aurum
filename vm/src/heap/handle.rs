use crate::object::Object;

use core::cell::Cell;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

/// Pointer to an object with no guarantees.
///
/// Much like with the primitive [pointer] type,
/// dereferencing an unsafe handle is not guaranteed to be safe.
/// The garbage collector may destroy or relocate objects,
/// which impacts the validity of existing unsafe handles.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UnsafeHandle<'h>
{
    pointer: NonNull<UnsafeCell<Object<'h>>>,
}

impl<'h> UnsafeHandle<'h>
{
    /// Create a dangling handle.
    pub fn dangling() -> Self
    {
        // Can’t use NonNull::dangling(), as Object is unsized.
        let ptr = 8usize as *mut UnsafeCell<Object<'h>>;
        unsafe { Self{pointer: NonNull::new_unchecked(ptr)} }
    }

    /// Create a handle from a pointer.
    pub fn new(pointer: NonNull<UnsafeCell<Object<'h>>>) -> Self
    {
        Self{pointer}
    }

    /// Access the handle as a pointer.
    pub fn as_ptr(self) -> NonNull<UnsafeCell<Object<'h>>>
    {
        self.pointer
    }
}

/// Pointer to a handle that is part of a scope.
///
/// Note that this is a pointer to a handle, not a handle itself.
/// This way, scoped handles remain valid when the garbage collector
/// rewrites handles after relocating objects.
/// For more information about scopes, see [`Scope`].
///
/// [`Scope`]: `super::Scope`
#[derive(Clone, Copy)]
pub struct ScopedHandle<'h, 's>
{
    handle: &'s Cell<UnsafeHandle<'h>>,
}

impl<'h, 's> ScopedHandle<'h, 's>
{
    /// Create a scoped handle from the underlying handle.
    ///
    /// # Safety
    ///
    /// The underlying handle must be part of a scope.
    pub (super) unsafe fn new(handle: &'s Cell<UnsafeHandle<'h>>) -> Self
    {
        Self{handle}
    }

    /// Convert the scoped handle to the underlying handle.
    ///
    /// This may return a different handle than previous time,
    /// as the garbage collector rewrites handles after relocating objects.
    pub fn as_unsafe_handle(self) -> UnsafeHandle<'h>
    {
        self.handle.get()
    }

    /// Modify this handle to refer to the same object as the other handle.
    pub fn copy_from(self, other: ScopedHandle<'h, 's>)
    {
        self.handle.set(other.as_unsafe_handle());
    }

    /// Modify this handle to refer to the same object as the other handle.
    pub unsafe fn copy_from_unsafe_handle(self, other: UnsafeHandle<'h>)
    {
        self.handle.set(other);
    }
}
