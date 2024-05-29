use std::cmp::Ordering;

// https://docs.rs/itertools/0.9.0/src/itertools/lib.rs.html#2061
pub trait Sorted: Iterator {
    fn sorted_by_vec<F>(self, cmp: F) -> Vec<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
    {
        let mut v = Vec::from_iter(self);
        v.sort_by(cmp);
        v
    }
}

impl<I: Iterator> Sorted for I {}
