/// An [`Iterator`] implementation that provides a join method
///
/// [`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
pub fn join<T, I>(mut iter: T, sep: &str) -> String
    where T: Iterator<Item=I>, I: std::fmt::Display
{
    use std::fmt::Write;
    match iter.next() {
        None => String::new(),
        Some(first_elt) => {
            // estimate lower bound of capacity needed
            let (lower, _) = iter.size_hint();
            let mut result = String::with_capacity(sep.len() * lower);
            write!(&mut result, "{}", first_elt).unwrap();
            for elt in iter {
                result.push_str(sep);
                write!(&mut result, "{}", elt).unwrap();
            }
            result
        }
    }
}


