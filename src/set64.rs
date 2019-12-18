// Copyright 2017-2018 David Roundy
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A set that is compact in size.

use std;
use std::marker::PhantomData;

/// This describes a type which can be stored in 64 bits without loss.
/// It is defined for all signed and unsigned integer types, as well
/// as `char`.  In each case, we store sets consisting exclusively of
/// "small" integers efficiently.
/// ```
pub trait Fits64 : Copy {
    /// Convert back *from* a u64.  This is unsafe, since it is only
    /// infallible (and lossless) if the `u64` originally came from
    /// type `Self`.
    #[inline]
    unsafe fn from_u64(x: u64) -> Self;
    /// Convert to a `u64`.  This should be infallible.
    #[inline]
    fn to_u64(self) -> u64;
}

macro_rules! define_fits {
    ($ty: ty) => {
        impl Fits64 for $ty {
            unsafe fn from_u64(x: u64) -> Self { x as $ty }
            fn to_u64(self) -> u64 { self as u64 }
        }
    };
}
define_fits!(u64);
define_fits!(u32);
define_fits!(u16);
define_fits!(u8);
define_fits!(usize);
impl Fits64 for char {
    unsafe fn from_u64(x: u64) -> Self {
        std::char::from_u32(x as u32).unwrap()
    }
    fn to_u64(self) -> u64 { self as u64 }
}
macro_rules! define_ifits {
    ($ty: ty, $uty: ty) => {
        impl Fits64 for $ty {
            unsafe fn from_u64(x: u64) -> Self {
                let abs = (x >> 1) as $ty;
                let neg = (x & 1) as $ty;
                // println!("x {} (abs is {} neg is {}) -> {}",
                //          x, abs, neg, abs*(neg*(-2)+1));
                abs*(neg*(-2)+1)
            }
            fn to_u64(self) -> u64 {
                let a = (self.abs() as u64) << 1;
                let b = (self as $uty >> (8*std::mem::size_of::<Self>()-1)) as u64;
                // println!("self {} (a {} b {}) -> {}", self, a, b, a+b);
                a + b
            }
        }
    };
}
define_ifits!(i8, u8);
define_ifits!(i16, u16);
define_ifits!(i32, u32);
define_ifits!(i64, u64);
define_ifits!(isize, usize);

/// A set type that can store any type that fits in a `u64`.  This set
/// type is very space-efficient in storing small or closely spaced
/// integers, while not being bad at storing large integers.
///
/// **Major caveat** The `Set64` type defines iterators (`drain()` and
/// `iter()`) that iterate over `T` rather than `&T`.  This is a break
/// with standard libray convention, and can be annoying if you are
/// translating code from `HashSet` to `Set64`.  The motivation for
/// this is several-fold:
///
/// 1. `Set64` does not store `T` directly in its data structures
/// (which would waste space), so there is no reference to the data to
/// take.  This does not make it impossible, but does mean we would
/// have to fabricate a `T` and return a reference to it, which is
/// awkward and ugly.
///
/// 2. There is no inefficiency involved in returning `T`, since it is
/// necessarily no larger than a pointer.
///
/// # Examples
///
/// ```
/// use tinyset::Set64;
///
/// let a: Set64<char> = "Hello world".chars().collect();
///
/// for x in "Hello world".chars() {
///     assert!(a.contains(&x));
/// }
/// for x in a {
///     assert!("Hello world".contains(x));
/// }
/// ```
///
/// # Storage details
///
/// Internally a `Set64` is identical to a [SetU64], so read there for
/// details.  In short, small sets are the size of a pointer with no
/// heap storage.  Densely packed sets are around a bit per member.
/// Intermediate sets have intermediate storage.  The worst case
/// scenario is large integers widely spaced apart, in which case the
/// storage is similar to a [`std::collections::HashSet`].
#[derive(Debug, Clone)]
pub struct Set64<T: Fits64>(crate::setu64::SetU64, PhantomData<T>);

impl<T: Fits64> Default for Set64<T> {
    /// Creates an empty set..
    fn default() -> Self {
        Set64(crate::setu64::SetU64::new(), PhantomData)
    }
}

impl<T: Fits64> Set64<T> {
    /// Creates an empty set..
    pub fn new() -> Self {
        Self::default()
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_capacity(_cap: usize) -> Self {
        Self::new()
    }
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn insert(&mut self, elem: T) -> bool {
        self.0.insert(elem.to_u64())
    }
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Returns true if the set contains a value.
    pub fn contains<R: std::borrow::Borrow<T>>(&self, value: R) -> bool {
        let x = value.borrow().clone().to_u64();
        self.0.contains(x)
    }
    /// Removes an element, and returns true if that element was present.
    pub fn remove(&mut self, value: &T) -> bool {
        let x = value.clone().to_u64();
        self.0.remove(x)
    }
    /// Iterate
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=T> + 'a {
        self.0.iter().map(|x| unsafe { T::from_u64(x) })
    }
    /// Drain
    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item=T> + 'a {
        self.0.drain().map(|x| unsafe { T::from_u64(x) })
    }
}

impl<T: Fits64> PartialEq for Set64<T> {
    fn eq(&self, other: &Set64<T>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for k in other.iter() {
            if !self.contains(k) {
                return false;
            }
        }
        true
    }
}
impl<T: Fits64> Eq for Set64<T> {}

impl<T: Fits64> std::hash::Hash for Set64<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut membs: Vec<u64> = self.iter().map(|i| i.to_u64()).collect();
        membs.sort();
        for memb in membs {
            memb.hash(state);
        }
    }
}

impl<T: Fits64> std::iter::FromIterator<T> for Set64<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (sz,_) = iter.size_hint();
        let mut c = Set64::with_capacity(sz);
        for i in iter {
            c.insert(i);
        }
        c
    }
}

/// An iterator.
pub struct IntoIter<T: Fits64>( crate::setu64::IntoIter, PhantomData<T> );

impl<T: Fits64> Iterator for IntoIter<T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<T> {
        self.0.next().map(|x| unsafe { T::from_u64(x) })
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }
    #[inline]
    fn last(self) -> Option<T> {
        self.0.last().map(|x| unsafe { T::from_u64(x) })
    }
}

impl<T: Fits64> IntoIterator for Set64<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter(self.0.into_iter(), PhantomData)
    }
}

impl<'a, 'b, T: Fits64> std::ops::Sub<&'b Set64<T>> for &'a Set64<T> {
    type Output = Set64<T>;

    /// Returns the difference of `self` and `rhs` as a new `Set64<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tinyset::Set64;
    ///
    /// let a: Set64<u32> = vec![1, 2, 3].into_iter().collect();
    /// let b: Set64<u32> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set = &a - &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2];
    /// for x in set {
    ///     assert!(expected.contains(&x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn sub(self, rhs: &Set64<T>) -> Set64<T> {
        let mut s = Set64::with_capacity(self.len());
        for v in self.iter() {
            if !rhs.contains(&v) {
                s.insert(v);
            }
        }
        s
    }
}

impl<T: Fits64> Extend<T> for Set64<T> {
    /// Adds a bunch of elements to the set
    ///
    /// # Examples
    ///
    /// ```
    /// use tinyset::Set64;
    ///
    /// let mut a: Set64<u32> = vec![1, 2, 3].into_iter().collect();
    /// a.extend(vec![3, 4, 5]);
    ///
    /// let mut i = 0;
    /// let expected = [1, 2, 3, 4, 5];
    /// for x in a {
    ///     assert!(expected.contains(&x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        for i in iter {
            self.insert(i);
        }
    }
}

impl<'a, 'b, T: Fits64> std::ops::BitOr<&'b Set64<T>> for &'a Set64<T> {
    type Output = Set64<T>;

    /// Returns the union of `self` and `rhs` as a new `Set64<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tinyset::Set64;
    ///
    /// let a: Set64<u32> = vec![1, 2, 3].into_iter().collect();
    /// let b: Set64<u32> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set = &a | &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2, 3, 4, 5];
    /// for x in set {
    ///     assert!(expected.contains(&x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitor(self, rhs: &Set64<T>) -> Set64<T> {
        let mut s: Set64<T> = Set64::with_capacity(self.len() + rhs.len());
        for x in self.iter() {
            s.insert(x);
        }
        for x in rhs.iter() {
            s.insert(x);
        }
        s
    }
}
