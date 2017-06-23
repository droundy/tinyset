//! smallset is a collection that is efficient for small numbers of
//! elements, while still scaling well for large numbers.  It is
//! basically interchangeable with `HashSet`, although it requires
//! that its elements implement the `Copy` trait, since it is
//! optimized for small elements.
//!
//! # Example
//!
//! ```
//! use smallset::SmallSet;
//! let mut s: SmallSet<usize> = SmallSet::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```

// #![deny(missing_docs)]

use std::collections::HashSet;
use std::hash::Hash;

pub const CAPACITY: usize = 8;

#[derive(Debug, Clone)]
pub struct SmallSet<T: Copy + Eq + Hash> {
    inner: SS<T>,
}

#[derive(Debug, Clone)]
enum SS<T: Copy+Eq+Hash> {
    Small(usize, [T;CAPACITY]),
    Large(HashSet<T>),
}

pub struct Iter<'a, T: 'a+Copy+Eq+Hash> {
    inner: It<'a, T>,
}

enum It<'a, T: 'a+Copy+Eq+Hash> {
    Small(std::slice::Iter<'a, T>),
    Large(std::collections::hash_set::Iter<'a, T>),
}

impl<T: Copy+Eq+Hash> SmallSet<T> {
    pub fn new() -> SmallSet<T> {
        SmallSet { inner: SS::Small(0, unsafe { std::mem::uninitialized() }) }
    }
    pub fn with_capacity(cap: usize) -> SmallSet<T> {
        if cap > CAPACITY {
            SmallSet { inner: SS::Large(HashSet::with_capacity(cap)) }
        } else {
            SmallSet::new()
        }
    }
    pub fn len(&self) -> usize {
        match self.inner {
            SS::Large(ref s) => s.len(),
            SS::Small(len, _) => len,
        }
    }
    /// Reserves capacity for at least `additional` more elements to be
    /// inserted in the HashSet. The collection may reserve more space
    /// to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        match self.inner {
            SS::Large(ref mut s) => {
                s.reserve(additional);
                return;
            },
            SS::Small(len, arr) => {
                let mut s = HashSet::with_capacity(additional+CAPACITY);
                for i in 0..len {
                    s.insert(arr[i]);
                }
                *self = SmallSet { inner: SS::Large(s) }
            },
        }
    }
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn insert(&mut self, elem: T) -> bool {
        match self.inner {
            SS::Large(ref mut s) => {
                return s.insert(elem);
            },
            SS::Small(ref mut len, ref mut arr) => {
                for i in 0 .. *len {
                    if arr[i] == elem {
                        return false;
                    }
                }
                if *len < CAPACITY-1 {
                    arr[*len] = elem;
                    *len += 1;
                    return true;
                }
            },
        }
        match self.inner {
            SS::Large(_) => unreachable!(),
            SS::Small(len, arr) => {
                let mut s = HashSet::with_capacity(1+CAPACITY);
                for i in 0..len {
                    s.insert(arr[i]);
                }
                s.insert(elem);
                *self = SmallSet { inner: SS::Large(s) };
                true
            },
        }
    }
    /// Returns true if the set contains a value.
    pub fn contains(&self, elem: &T) -> bool {
        match self.inner {
            SS::Large(ref s) => {
                s.contains(elem)
            },
            SS::Small(len, ref arr) => {
                for i in 0 .. len {
                    if arr[i] == *elem {
                        return true;
                    }
                }
                false
            },
        }
    }
    /// Returns an iterator over the set.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            inner:
            match self.inner {
                SS::Large(ref s) => {
                    It::Large(s.iter())
                },
                SS::Small(len, ref arr) => {
                    It::Small(arr[0..len].iter())
                },
            }
        }
    }
}

impl<T: Hash+Copy+Eq> std::iter::FromIterator<T> for SmallSet<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (sz,_) = iter.size_hint();
        let mut c = SmallSet::with_capacity(sz);
        for i in iter {
            c.insert(i);
        }
        c
    }
}


impl<'a, T: 'a+Eq+Hash+Copy> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        match self.inner {
            It::Large(ref mut it) => it.next(),
            It::Small(ref mut it) => it.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.inner {
            It::Large(ref it) => it.size_hint(),
            It::Small(ref it) => it.size_hint(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut ss: SmallSet<usize> = SmallSet::new();
        ss.insert(5);
        assert!(ss.contains(&5));
        assert!(!ss.contains(&4));
        ss.insert(3);
        println!("now {:?}", &ss);
        assert!(ss.contains(&3));
        assert!(ss.contains(&5));
        assert!(ss.len() == 2);
        for num in ss.iter() {
            assert!(ss.contains(num));
        }
    }
    #[test]
    fn size_unwasted() {
        println!("small size: {}", std::mem::size_of::<SmallSet<usize>>());
        println!(" hash size: {}", std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<SmallSet<usize>>() <=
                2*std::mem::size_of::<HashSet<usize>>());
    }
}
