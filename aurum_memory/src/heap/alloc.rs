use crate::object::Header;
use crate::object::Object;
use crate::object::Payload;
use super::Heap;
use super::ScopedHandle;
use super::UnsafeHandle;

use alloc::alloc::alloc;
use alloc::alloc::handle_alloc_error;
use core::alloc::Layout;
use core::ptr::NonNull;

impl<'h> Heap<'h>
{
    /// Allocate memory for an object and initialize it.
    ///
    /// Memory is allocated on the garbage collected heap
    /// for an object of the given payload size.
    /// The `init` function is called to initialize the payload.
    /// The object header is set to the header returned by `init`,
    /// and then the relevant flags of the object are set.
    ///
    /// You would not normally use this method.
    /// Instead use one of the `new_*` methods.
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
        payload_size: usize,
        init: impl FnOnce(*mut Payload) -> Header,
    ) -> UnsafeHandle<'h>
    {
        // TODO: Replace this with a pointer bump allocation.

        let layout = Layout::from_size_align_unchecked(8 + payload_size, 8);

        let pointer = alloc(layout);
        let pointer = pointer as *mut Object<'h>;

        if pointer.is_null() {
            handle_alloc_error(layout);
        }

        (*pointer).header = init(&mut (*pointer).payload);

        UnsafeHandle::new(NonNull::new_unchecked(pointer))
    }

    /// Similar to [`alloc`][`Self::alloc`],
    /// but point the given scoped handle to the new object.
    pub unsafe fn new<'s>(
        &self,
        into: ScopedHandle<'h, 's>,
        payload_size: usize,
        init: impl FnOnce(*mut Payload) -> Header,
    )
    {
        let object = self.alloc(payload_size, init);
        into.copy_from_unsafe_handle(object);
    }
}
