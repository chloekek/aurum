//! Combine the blazing speed of [`UnsafeCell`]
//! with a clown-inspired twist on the safety of [`RefCell`].

#![no_std]
#![warn(missing_docs)]

#[allow(unused)] use core::cell::RefCell;
#[allow(unused)] use core::cell::RefMut;
#[allow(unused)] use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;

/// Behaves just like [`UnsafeCell`],
/// but with [`RefCell`]-like runtime checks
/// when debug assertions are enabled.
pub struct UnsafeRefCell<T>
{
    #[cfg(debug_assertions)]
    inner: RefCell<T>,

    #[cfg(not(debug_assertions))]
    inner: UnsafeCell<T>,
}

impl<T> UnsafeRefCell<T>
{
    /// Create a new cell containing the given value.
    pub const fn new(value: T) -> Self
    {
        Self::new_internal(value)
    }

    #[cfg(debug_assertions)]
    const fn new_internal(value: T) -> Self
    {
        Self{inner: RefCell::new(value)}
    }

    #[cfg(not(debug_assertions))]
    const fn new_internal(value: T) -> Self
    {
        Self{inner: UnsafeCell::new(value)}
    }

    /// Consume the cell, returning the contained value.
    pub fn into_inner(self) -> T
    {
        self.inner.into_inner()
    }

    /// Mutably borrow the contained value.
    ///
    /// The borrow lasts until the return value is dropped.
    ///
    /// # Safety
    ///
    /// There must not exist other borrows of the value.
    /// In debug mode, such a condition causes a panic.
    /// In release mode, the behavior under such a condition is undefined.
    pub unsafe fn borrow_mut(&self) -> UnsafeRefMut<T>
    {
        self.borrow_mut_internal()
    }

    #[cfg(debug_assertions)]
    unsafe fn borrow_mut_internal(&self) -> UnsafeRefMut<T>
    {
        UnsafeRefMut{inner: self.inner.borrow_mut()}
    }

    #[cfg(not(debug_assertions))]
    unsafe fn borrow_mut_internal(&self) -> UnsafeRefMut<T>
    {
        UnsafeRefMut{inner: &mut *self.inner.get()}
    }
}

/// Mutably borrowed content of [`UnsafeRefCell`].
pub struct UnsafeRefMut<'a, T>
{
    #[cfg(debug_assertions)]
    inner: RefMut<'a, T>,

    #[cfg(not(debug_assertions))]
    inner: &'a mut T,
}

impl<'a, T> Deref for UnsafeRefMut<'a, T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for UnsafeRefMut<'a, T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        self.inner.deref_mut()
    }
}
