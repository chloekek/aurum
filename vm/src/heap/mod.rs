//! Memory allocation and garbage collection.

pub use self::handle::*;
pub use self::heap::*;

mod handle;
mod heap;

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn construction()
    {
        Heap::with_new(|heap| {
            heap.with_new_array_scope(|_scope: [_; 4]| {
            });
        });
    }
}
