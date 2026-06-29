use std::borrow::Borrow;
use std::cell::Cell;
// use crate::utils::Cow2::{Borrowed2, Owned2};

pub fn escape_whitespaces(data: impl Borrow<str>, escape_spaces: bool) -> String {
    let data = data.borrow();
    let mut res = String::with_capacity(data.len());
    data.chars().for_each(|ch| match ch {
        ' ' if escape_spaces => res.push('\u{00B7}'),
        '\t' => res.push_str("\\t"),
        '\n' => res.push_str("\\n"),
        '\r' => res.push_str("\\r"),
        _ => res.push(ch),
    });
    res
}

pub fn cell_update<T: Copy, F>(cell: &Cell<T>, f: F) -> T
where
    F: FnOnce(T) -> T,
{
    let old = cell.get();
    let new = f(old);
    cell.set(new);
    new
}