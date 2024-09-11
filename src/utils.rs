use std::fmt::Write;

/// Concatenate display of the contents
pub(crate) struct Concat<T>(pub T);

impl<A, B, C> std::fmt::Display for Concat<(A, B, C)>
where
    A: std::fmt::Display,
    B: std::fmt::Display,
    C: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let t = &self.0;
        write!(f, "{}{}{}", t.0, t.1, t.2)
    }
}

pub(crate) fn join<T>(sep: &str, iter: T) -> String
where
    T: IntoIterator,
    T::Item: std::fmt::Display,
{
    let mut out = String::new();
    let mut iter = iter.into_iter();
    if let Some(fst) = iter.next() {
        out.write_str(fst).unwrap();
        for elt in iter {
            write!(&mut out, "{}{}", sep, elt).unwrap();
        }
    }
    out
}
