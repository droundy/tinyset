//! david-set is a collection that is efficient for small numbers of
//! elements, while still scaling well for large numbers.  It is
//! basically interchangeable with `HashSet`, although it requires
//! that its elements implement the `Copy` trait, since otherwise I
//! would have to learn to write correct `unsafe` code, which would be
//! scary.
//!
//! # Example
//!
//! ```
//! use david_set::Set;
//! let mut s: Set<usize> = Set::new();
//! s.insert(1);
//! assert!(s.contains(&1));
//! ```

#![deny(missing_docs)]

use std::collections::HashSet;
use std::hash::Hash;
use std::borrow::Borrow;

/// The number of elements stored in an array before moving up to the
/// `HashSet` implementation.
pub const CAPACITY: usize = 8;

/// A set that is a `HashSet` when it has many elements, but is just
/// an array for small set sizes.
///
/// As with the `HashSet` type, a `Set` requires that the
/// elements implement the Eq and Hash traits.  This can frequently be
/// achieved by using #[derive(PartialEq, Eq, Hash)]. In addition,
/// `Set` requires that the elements implement the `Copy` trait,
/// and really they should be pretty small, since Set always
/// stores room for `CAPACITY` elements.
#[derive(Debug, Clone)]
pub struct Set<T: Copy + Eq + Hash> {
    inner: SS<T>,
}

#[derive(Debug, Clone)]
enum SS<T: Copy+Eq+Hash> {
    Small(usize, [T;CAPACITY]),
    Large(HashSet<T>),
}

/// An iterator for consuming sets.
pub struct IntoIter<T: Copy+Eq+Hash> {
    inner: IntoIt<T>,
}

enum IntoIt<T: Copy+Eq+Hash> {
    Small(std::vec::IntoIter<T>),
    Large(std::collections::hash_set::IntoIter<T>),
}

/// An iterator for sets.
pub struct Iter<'a, T: 'a+Copy+Eq+Hash> {
    inner: It<'a, T>,
}

enum It<'a, T: 'a+Copy+Eq+Hash> {
    Small(std::slice::Iter<'a, T>),
    Large(std::collections::hash_set::Iter<'a, T>),
}

impl<T: Copy+Eq+Hash> Set<T> {
    /// Creates an empty set..
    pub fn new() -> Set<T> {
        Set { inner: SS::Small(0, unsafe { std::mem::uninitialized() }) }
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_capacity(cap: usize) -> Set<T> {
        if cap > CAPACITY {
            Set { inner: SS::Large(HashSet::with_capacity(cap)) }
        } else {
            Set::new()
        }
    }
    /// Returns the number of elements in the set.
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
                *self = Set { inner: SS::Large(s) }
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
                if *len < CAPACITY {
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
                *self = Set { inner: SS::Large(s) };
                true
            },
        }
    }
    /// Removes an element, and returns true if that element was present.
    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
        where
        T: Borrow<Q>, Q: Hash + Eq,
    {
        match self.inner {
            SS::Large(ref mut s) => s.remove(value),
            SS::Small(ref mut len, ref mut arr) => {
                for i in 0..*len {
                    if arr[i].borrow() == value {
                        *len -= 1;
                        for j in i..*len {
                            arr[j] = arr[j+1];
                        }
                        return true;
                    }
                }
                false
            },
        }
    }
    /// Returns true if the set contains a value.
    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
        where
        T: Borrow<Q>, Q: Hash + Eq,
    {
        match self.inner {
            SS::Large(ref s) => {
                s.contains(value)
            },
            SS::Small(len, ref arr) => {
                for i in 0 .. len {
                    if arr[i].borrow() == value {
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
    /// Clears the set, returning all elements in an iterator.
    pub fn drain(&mut self) -> IntoIter<T> {
        let mut s = Set::new();
        std::mem::swap(&mut s, self);
        IntoIter {
            inner:
            match s.inner {
                SS::Large(s) => {
                    IntoIt::Large(s.into_iter())
                },
                SS::Small(len, ref arr) => {
                    IntoIt::Small(Vec::from(&arr[0..len]).into_iter())
                },
            }
        }
    }
}

impl<T: Hash+Copy+Eq> std::iter::FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (sz,_) = iter.size_hint();
        let mut c = Set::with_capacity(sz);
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

impl<T: Eq+Hash+Copy> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.inner {
            IntoIt::Large(ref mut it) => it.next(),
            IntoIt::Small(ref mut it) => it.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.inner {
            IntoIt::Large(ref it) => it.size_hint(),
            IntoIt::Small(ref it) => it.size_hint(),
        }
    }
}

impl<'a, T: Eq+Hash+Copy> IntoIterator for &'a Set<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<T: Eq+Hash+Copy> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Creates a consuming iterator, that is, one that moves each value out
    /// of the set in arbitrary order. The set cannot be used after calling
    /// this.
    ///
    /// # Examples
    ///
    /// ```
    /// use david_set::Set;
    /// let mut set: Set<u32> = Set::new();
    /// set.insert(2);
    /// set.insert(5);
    ///
    /// // Not possible to collect to a Vec<String> with a regular `.iter()`.
    /// let v: Vec<_> = set.into_iter().collect();
    ///
    /// // Will print in an arbitrary order.
    /// for x in &v {
    ///     println!("{}", x);
    /// }
    /// ```
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner:
            match self.inner {
                SS::Large(s) => {
                    IntoIt::Large(s.into_iter())
                },
                SS::Small(len, arr) => {
                    IntoIt::Small(Vec::from(&arr[0..len]).into_iter())
                },
            }
        }
    }
}

impl<'a, 'b, T: Eq+Hash+Copy> std::ops::Sub<&'b Set<T>> for &'a Set<T> {
    type Output = Set<T>;

    /// Returns the difference of `self` and `rhs` as a new `Set<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use david_set::Set;
    ///
    /// let a: Set<u32> = vec![1, 2, 3].into_iter().collect();
    /// let b: Set<u32> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set = &a - &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn sub(self, rhs: &Set<T>) -> Set<T> {
        let mut s = Set::with_capacity(self.len());
        for v in self.iter() {
            if !rhs.contains(v) {
                s.insert(*v);
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut ss: Set<usize> = Set::new();
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
        println!("small size: {}", std::mem::size_of::<Set<usize>>());
        println!(" hash size: {}", std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<Set<usize>>() <=
                2*std::mem::size_of::<HashSet<usize>>());
    }
}
