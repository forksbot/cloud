use log::{error};
use std::collections::BTreeSet;
use serde::{Serializer, Deserializer, Deserialize};

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
            if let Err(e) = write!(&mut result, "{}", first_elt) {
                error!("{:?}", e);
            }
            for elt in iter {
                result.push_str(sep);
                if let Err(e) = write!(&mut result, "{}", elt) {
                    error!("{:?}", e);
                }
            }
            result
        }
    }
}

pub fn scope_serialize<S>(x: &BTreeSet<String>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let string = x.iter().fold(String::new(), |mut f, g| {
        f += g;
        f += " ";
        f
    });
    s.serialize_str(string.trim_end())
}

pub fn scope_deserialize<'de, D>(deserializer: D) -> Result<BTreeSet<String>, D::Error>
    where
        D: Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    Ok(str_sequence
        .split(' ')
        .map(|item| item.to_owned())
        .filter(|f| !f.is_empty())
        .collect())
}
