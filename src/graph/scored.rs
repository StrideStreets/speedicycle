use std::cmp::Ordering;

/// `MaxScored<K, T>` holds a score `K` and a scored object `T` in
/// a pair for use with a `BinaryHeap`.

#[derive(Clone, Copy, Debug)]
pub struct MaxScored<K, T>(pub K, pub T);

impl<K: PartialOrd, T> PartialEq for MaxScored<K, T> {
    #[inline]
    fn eq(&self, other: &MaxScored<K, T>) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<K: PartialOrd, T> Eq for MaxScored<K, T> {}

impl<K: PartialOrd, T> PartialOrd for MaxScored<K, T> {
    #[inline]
    fn partial_cmp(&self, other: &MaxScored<K, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: PartialOrd, T> Ord for MaxScored<K, T> {
    fn cmp(&self, other: &MaxScored<K, T>) -> Ordering {
        let a = &self.0;
        let b = &other.0;

        if a == b {
            Ordering::Equal
        } else if a < b {
            Ordering::Less
        } else if a > b {
            Ordering::Greater
        } else if a.ne(a) && b.ne(b) {
            // these are the NaN cases
            Ordering::Equal
        } else if a.ne(a) {
            // Order NaN GREATER, so that it is first in the MaxScore order
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}
