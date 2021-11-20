use crate::object::Flags;
use crate::object::Header;
use crate::object::Object;
use crate::object::Payload;

use core::cell::Cell;
use core::marker::PhantomData;
use core::ptr::NonNull;
use scopeguard::defer;

/// Pointer to an object with no guarantees.
///
/// Much like with the primitive [pointer] type,
/// dereferencing an unsafe handle is not guaranteed to be safe.
/// The garbage collector may destroy or relocate objects,
/// which impacts the validity of existing unsafe handles.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct UnsafeHandle<'h>
{
    pointer: NonNull<Object<'h>>,
}

impl<'h> UnsafeHandle<'h>
{
    /// Create a dangling handle.
    #[inline]
    pub fn dangling() -> Self
    {
        // Can’t use NonNull::dangling(), as Object is unsized.
        let ptr = 8usize as *mut Object<'h>;
        unsafe { Self{pointer: NonNull::new_unchecked(ptr)} }
    }

    /// Create a handle from a pointer.
    #[inline]
    pub fn new(pointer: NonNull<Object<'h>>) -> Self
    {
        Self{pointer}
    }

    /// Access the handle as a pointer.
    #[inline]
    pub fn as_ptr(self) -> *mut Object<'h>
    {
        self.pointer.as_ptr()
    }

    /// Get the header of the object referenced by this handle.
    ///
    /// # Safety
    ///
    /// The handle must point to an object.
    #[inline]
    pub unsafe fn header(self) -> *mut Header
    {
        &mut (*self.as_ptr()).header
    }

    /// Get the payload of the object referenced by this handle.
    ///
    /// # Safety
    ///
    /// The handle must point to an object.
    #[inline]
    pub unsafe fn payload(self) -> *mut Payload
    {
        &mut (*self.as_ptr()).payload
    }
}

impl<'h> PartialEq for UnsafeHandle<'h>
{
    #[inline]
    fn eq(&self, other: &Self) -> bool
    {
        self.pointer == other.pointer
    }
}

impl<'h> Eq for UnsafeHandle<'h>
{
}

/// Pointer to an object whose [`PINNED`] flag is set.
///
/// The garbage collector will not destroy or relocate pinned objects.
/// As a result, we can simply point to them as long as they are pinned.
/// This lets you borrow cells of components of the object,
/// which is especially important with large objects such as arrays.
///
/// [`PINNED`]: `crate::object::Flags::PINNED`
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PinnedHandle<'h, 'p>
{
    _pin_frame: PhantomData<&'p ()>,
    handle: UnsafeHandle<'h>,
}

impl<'h, 'p> PinnedHandle<'h, 'p>
{
    /// Convert the pinned handle to the underlying handle.
    #[inline]
    pub fn as_unsafe_handle(self) -> UnsafeHandle<'h>
    {
        self.handle
    }

    /// Get the header of the object referenced by this handle.
    #[inline]
    pub fn header(self) -> Header
    {
        // SAFETY: The handle refers to an object, as it is pinned.
        unsafe { *self.as_unsafe_handle().header() }
    }

    /// Get the payload of the object referenced by this handle.
    #[inline]
    pub fn payload(self) -> *mut Payload
    {
        // SAFETY: The handle refers to an object, as it is pinned.
        unsafe { self.as_unsafe_handle().payload() }
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
    #[inline]
    pub unsafe fn new(handle: &'s Cell<UnsafeHandle<'h>>) -> Self
    {
        Self{handle}
    }

    /// Convert the scoped handle to the underlying handle.
    ///
    /// This may return a different handle than previous time,
    /// as the garbage collector rewrites handles after relocating objects.
    #[inline]
    pub fn as_unsafe_handle(self) -> UnsafeHandle<'h>
    {
        self.handle.get()
    }

    /// Modify this handle to refer to the same object as the other handle.
    #[inline]
    pub fn copy_from(self, other: ScopedHandle<'h, 's>)
    {
        self.handle.set(other.as_unsafe_handle());
    }

    /// Modify this handle to refer to the same object as the other handle.
    #[inline]
    pub unsafe fn copy_from_unsafe_handle(self, other: UnsafeHandle<'h>)
    {
        self.handle.set(other);
    }

    /// Obtain a pinned handle to the object referenced by this handle.
    ///
    /// If the object is not already pinned, it will be pinned.
    /// If so, the object will also be unpinned once `then` returns.
    pub fn with_pin<F, R>(self, then: F) -> R
        where F: for<'p> FnOnce(PinnedHandle<'h, 'p>) -> R
    {
        let pinned_handle = PinnedHandle{
            _pin_frame: PhantomData,
            handle: self.as_unsafe_handle(),
        };

        unsafe {
            let header = self.as_unsafe_handle().header();
            let flags: *mut Flags = &mut (*header).flags;
            let already_pinned = (*flags).contains(Flags::PINNED);

            if !already_pinned {
                (*flags).insert(Flags::PINNED);
            }

            defer! {
                if !already_pinned {
                    (*flags).remove(Flags::PINNED);
                }
            }

            then(pinned_handle)
        }
    }

    /// Get the header of the object referenced by this handle.
    #[inline]
    pub fn header(self) -> Header
    {
        // SAFETY: The handle refers to an object, as it is scoped.
        unsafe { *self.as_unsafe_handle().header() }
    }
}
