//! Memory allocation and garbage collection.

pub use self::handle::*;
pub use self::heap::*;

mod handle;
mod heap;

#[cfg(test)]
mod tests
{
    use crate::object::DeBruijn;
    use super::*;

    #[test]
    fn construction()
    {
        Heap::with_new(|heap| {

            heap.with_new_array_scope(|scope: [_; 4]| {

                heap.new_symbol(scope[0], b"Add").unwrap();
                heap.new_symbol(scope[1], b"Pi").unwrap();
                heap.new_variable(scope[2], DeBruijn(0)).unwrap();

                let function = scope[0];
                let arguments = [scope[1], scope[2]].into_iter();
                heap.new_application(scope[3], function, arguments).unwrap();

            });

        });
    }
}
