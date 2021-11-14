/// Convenient macro for defining heap methods
/// that allocate and initialize new objects.
/// For each method, this macro generates a method that returns unsafe handle
/// and a method that assigns the object to a given scoped handle.
macro_rules! alloc_methods
{
    (
        #![doc = $impl_doc:expr]
        $(
            $(#[doc = $doc:expr])*
            #[scoped_alias = $new_name:ident]
            pub fn $alloc_name:ident(
                &$self_name:ident
                $(, $param_name:ident: $param_type:ty)*
                $(,)?
            ) -> Result<UnsafeHandle<'h>, $error_type:ty>
            $body:block
        )*
    ) => {
        #[doc = "Methods for creating"]
        #[doc = $impl_doc]
        #[doc = "on the heap."]
        impl<'h> $crate::heap::Heap<'h>
        {
            $(
                $(#[doc = $doc])*
                pub fn $alloc_name(&$self_name, $($param_name: $param_type),*)
                    -> Result<UnsafeHandle<'h>, $error_type>
                $body

                $(#[doc = $doc])*
                pub fn $new_name<'s>(
                    &$self_name,
                    into: $crate::heap::ScopedHandle<'h, 's>,
                    $($param_name: $param_type),*
                ) -> Result<(), $error_type>
                {
                    let object = $self_name.$alloc_name($($param_name),*)?;
                    unsafe { Ok(into.copy_from_unsafe_handle(object)) }
                }
            )*
        }
    };
}
