#![deny(missing_docs)]
//! This is a crate for the tiniest sets ever.

mod iter;
pub use iter::IntoIter;

const fn num_bits<T>() -> u64 {
    std::mem::size_of::<T>() as u64 * 8
}

fn log_2(x: u64) -> u64 {
    if x == 0 {
        1
    } else {
        num_bits::<u64>() as u64 - x.leading_zeros() as u64
    }
}

#[test]
fn test_log_2() {
    assert_eq!(log_2(0), 1);
    assert_eq!(log_2(1), 1);
    assert_eq!(log_2(7), 3);
    assert_eq!(log_2(8), 4);
}

fn compute_array_bits(mx: u64) -> u64 {
    if log_2(mx) < 2 {
        return 62;
    } else if log_2(mx) > 62 {
        return 0;
    }
    let mut bits = num_bits::<u64>() - log_2(mx);
    while num_bits::<u64>() - log_2(mx) < bits {
        bits += 1;
    }
    bits
}

#[test]
fn confirm_doctest_bits() {
    assert_eq!(51, compute_array_bits(5000));
}

fn split_u64(x: u64, bits: u64) -> (u64, u64) {
    if bits > 0 {
        (x / bits, (x % bits))
    } else {
        (x, 0)
    }
}

fn unsplit_u64(k: u64, offset: u64, bits: u64) -> u64 {
    if bits > 0 {
        k * bits + offset
    } else {
        k
    }
}

/// A set of `u64`
///
/// ## Implementation
///
/// The implementation and size of a `SetU64` is an internal detail that
/// is *not stable*, but may guide your use.  It is optimized for size,
/// while maintaining the scaling of a hashmap.
///
/// The implementation
/// is designed for the use case of storing indexes into a `Vec`.  This use
/// case tends to involve small integers (they must be less than its `len`),
/// which could include any number of such integers.  They also have a greater
/// than average likelihood of including sequential or close integers, particularly if
/// values are pushed to the `Vec` while their indexes are added to sets.
///
/// ### Small sets
///
/// Very sets of up to seven small numbers are stored on the stack in a single
///  tagged pointer.
/// This is 8 bytes on a 64-bit system, and 4 bytes on a 32-bit system.
/// On a 32-bit system (which I won't discuss further here, look in the code!)
/// the elemets must be smaller in order to be stored without allocation.
///
/// Sets stored in a single word have 3 bits dedicated to the number of elements
/// in the set, with the remaining bits used to represent the value of the
/// smallest element in the set, followed by the differences between subsequent
/// elements in the ordered set.  Twice as many bits (rounded up) are dedicated to the fist
/// element as to the differences.
///  
/// On a 64-bit system, this works out to...
///
/// - We can store a set with a single integer less than about 10<sup>18</sup>.
///   Thus we should be able to store just about any zero-element or one-element
///   values on the stack.
/// - We can store two values on the stack, provided the lesser is less than 10<sup>12</sup>,
///   and the difference between them is less than a million (10<sup>6</sup>).
///   This starkly highlights the optimization for closely spaced numbers.  It is
///   also tweakable in the code, and could be changed.
/// - We can store three values on the stack, if the first is less than about
///   3×10<sup>7</sup> (30 million), and the following two values have gaps
///   of less than 4096.
/// - ...
/// - To hold 7 values in 64 bits, we limit the first value to about 500 thousand,
///   and the following differences must be less than 128.  So your seven numbers
///   will seriously need to be either all quite small or quite closely packed.
///
/// ### Larger sets
///
/// Larger sets are stored on the heap.  We currently have three different heap formats,
/// which will be chosen based on the distribution of your values.  The format is only
/// changed when reallocation is required, so it may be challenging to predict the format
/// of a given set, particularly as the reallocation size is randomized in order to
/// mitigate the risk of hash collision attacks.  The three formats are:
///
/// 1. *`Internal::Dense`* The set is stored as a bitmap up to a maximum value.
///    This format is chosen when the number of elements exceeds 1/127 of the maximum
///    value (see the implementation of [`SetU64::with_capacity_and_max`]), which means
///    that less than a byte will be used per element.
/// 2. *`Internal::Heap`* The set is stored as a kind of Robin Hood hash map (without hashing!) from most
///    significant bits to bitmaps holding sets of the stored least significant bits.
/// 3. *`Internal::Big`* An ordinary Robin Hood hash set (without hashing!), with a dynamic sentinal value
///    indicating that a bucket is empty.  This is used only when the maximum value in
///    the set is very large.
///
/// #### `Internal::Heap` format
///
/// I will describe here some details of the `Internal::Heap` format, which is likely the
/// most common one, and is definitely the most complicated.  The data is stored in an array
/// of `u64` buckets.  Based on the largest element
/// present, we divide the bits of each of those elements into a key and a bitmap.  The array
/// is a Robin Hood hashmap between keys and bitmaps.  The key
/// represents the most significant value of the elements stored, and the bitmap stores the
/// set of values which have that same most significant value.
///
/// As an example, consider a set with a maximum of 5000.  This maximum requires 13
/// bits to represent, but we don't *need* 13 bits to store the key, because that would leave 51 bits for
/// the bitmap, so each bucket would hold 51 possible values, enabling us to store values of
/// up to `51*(1 << 13)`.  So instead we could use just 7 bits to hold the keys, leaving 57
/// bits per bucket, which would thus enable us to store values up to about 54 thousand. Which
/// bucket size we use will depend on the order in which elements were added, since we only
/// reallocate when needed either because we have an element that we cannot fit, or because our
/// hashmap is too full.  When allocating, we tend to leave room for the maximum to increase.
///
/// Assuming we use 13 bits per bucket, then let's talk through the process of inserting the
/// value 137.  Since we have 51 elements per bucket, the key is found by dividing by 51,
/// which gives us a key of 137/51 = 2.  We then look up the bucket with key 2 using a pretty
/// standard Robin Hood algorithm (except that the keys are a portion of a word).  Once we have
/// that bucket, we will identify that the bit corresponding to our value is bit number `137 % 51`,
/// which we will set (and also check the value of, to track the number of elements in the set)
/// and determine the return value.
///
/// This format allows us to efficiently store sets in which there are contiguous chunks of
/// elements, and when the elements are widely spaced it at least takes no more than 64 bits
/// per 64-bit value, plus hash-set overhead.  Its complexity also makes it significantly
/// slower for insertion (or `collect()`) than a standard `HashSet`, but it also can take
/// considerably less space.
pub struct SetU64(*mut S);

unsafe impl Send for SetU64 {}
unsafe impl Sync for SetU64 {}

use crate::copyset::impl_set_methods;
impl_set_methods!(SetU64);

#[repr(C)]
#[derive(Debug)]
struct S {
    b: Sbeginning,
    array: u64,
}

#[repr(C)]
#[derive(Debug)]
struct Sbeginning {
    sz: usize,
    cap: usize,
    bits: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tiny {
    sz: u8,
    sz_spent: u8,
    bits: usize,
    last: usize,
}

fn mask(bits: usize) -> u64 {
    (1 << bits) - 1
}

impl Iterator for Tiny {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let bitsplits = BITSPLITS[self.sz as usize];
        if self.sz_spent < self.sz {
            let nbits = bitsplits[self.sz_spent as usize];
            let difference = self.bits & mask(nbits as usize) as usize;
            if self.sz_spent == 0 {
                self.last = difference;
            } else {
                self.last = self.last + 1 + difference
            }
            self.bits = self.bits >> nbits;
            self.sz_spent += 1;
            Some(self.last as u64)
        } else {
            None
        }
    }
    fn count(self) -> usize {
        self.sz as usize
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz as usize, Some(self.sz as usize))
    }
    fn min(mut self) -> Option<u64> {
        self.next()
    }
    fn max(self) -> Option<u64> {
        self.last()
    }
}

#[cfg(target_pointer_width = "64")]
static BITSPLITS: [&[u64]; 8] = [
    &[],
    &[61],
    &[40, 21],
    &[31, 15, 15],
    &[25, 12, 12, 12],
    &[21, 10, 10, 10, 10],
    &[21, 8, 8, 8, 8, 8],
    &[19, 7, 7, 7, 7, 7, 7],
];

#[cfg(target_pointer_width = "32")]
static BITSPLITS: [&[u64]; 8] = [
    &[],
    &[29],
    &[20, 9],
    &[15, 7, 7],
    &[12, 5, 5, 5],
    &[10, 4, 4, 4, 4],
    &[9, 4, 4, 4, 4, 4],
    &[8, 3, 3, 3, 3, 3, 3],
];

impl Tiny {
    #[cfg(test)]
    fn debug_me(self, msg: &str) {
        println!("{}: {:?} => {:?}", msg, self, self.collect::<Vec<_>>());
    }
    fn to_usize(self) -> usize {
        self.sz as usize | self.bits << 3
    }
    fn from_usize(x: usize) -> Self {
        Tiny {
            sz: x as u8 & 7,
            bits: x >> 3,
            sz_spent: 0,
            last: 0,
        }
    }
    fn from_singleton(x: u64) -> Option<Self> {
        if log_2(x) > BITSPLITS[1][0] {
            None
        } else {
            Some(Tiny {
                sz: 1,
                bits: x as usize,
                sz_spent: 0,
                last: 0,
            })
        }
    }
    fn new_sorted_deduped(v: &[u64]) -> Option<Self> {
        if v.len() == 0 {
            return None;
        } else if v.len() > BITSPLITS.len() - 1 {
            return None;
        }
        let sz = v.len() as u8;
        let mut last = 0;
        let mut offset = 0;
        let mut bits: usize = 0;
        let bitsplits = BITSPLITS[sz as usize];
        for (x, nbits) in v.iter().cloned().zip(bitsplits.iter().cloned()) {
            let y = if offset == 0 { x } else { x - last - 1 };
            if log_2(y) > nbits {
                return None;
            }
            bits = bits | (y as usize) << offset;
            offset += nbits;
            last = x;
        }
        Some(Tiny {
            sz,
            bits,
            sz_spent: 0,
            last: 0,
        })
    }
    fn insert(mut self, e: u64) -> Option<Self> {
        if e > usize::MAX as u64 {
            return None;
        }
        let mut e = e as usize;
        let old_bitsplits = BITSPLITS[self.sz as usize];
        if let Some(new_bitsplits) = BITSPLITS.get(self.sz as usize + 1) {
            let mut new = Tiny {
                bits: 0,
                sz: self.sz + 1,
                last: 0,
                sz_spent: 0,
            };
            let backup = self.clone();
            let mut offset = 0;
            let mut old_iter = old_bitsplits.iter().cloned();
            let mut new_iter = new_bitsplits.iter().cloned();
            while let Some(newb) = new_iter.next() {
                if let Some(oldb) = old_iter.next() {
                    let mut n = self.bits & mask(oldb as usize) as usize;
                    if e == n {
                        return Some(backup);
                    } else if log_2(n as u64) > newb {
                        if e < n {
                            return None;
                        }
                        e -= n + 1;
                        self.bits = self.bits >> oldb;
                        for oldb in old_iter {
                            let n = self.bits & mask(oldb as usize) as usize;
                            if n == e {
                                return Some(backup);
                            }
                            if e < n {
                                return None;
                            }
                            e -= n + 1;
                            self.bits = self.bits >> oldb;
                        }
                        return None;
                    } else if e < n {
                        new.bits = new.bits | (e << offset);
                        offset += newb;
                        n = n - e - 1;
                        let newb = new_iter.next().unwrap();
                        if log_2(n as u64) > newb {
                            return None;
                        }
                        new.bits = new.bits | (n << offset);
                        offset += newb;
                        self.bits = self.bits >> oldb;
                        for newb in new_iter {
                            let oldb = old_iter.next().unwrap();
                            let n = self.bits & mask(oldb as usize) as usize;
                            if log_2(n as u64) > newb {
                                return None;
                            }
                            new.bits = new.bits | (n << offset);
                            self.bits = self.bits >> oldb;
                            offset += newb;
                        }
                        return Some(new);
                    }
                    e -= n + 1;
                    new.bits = new.bits | (n << offset);
                    offset += newb;
                    self.bits = self.bits >> oldb;
                } else {
                    // the new one is last
                    if log_2(e as u64) > newb {
                        return None;
                    }
                    new.bits = new.bits | (e << offset);
                }
            }
            Some(new)
        } else {
            if self.clone().any(|x| x == e as u64) {
                Some(self)
            } else {
                None
            }
        }
    }
    fn contains(mut self, e: u64) -> bool {
        if e > usize::MAX as u64 {
            return false;
        }
        let mut e = e as usize;
        let bitsplits = BITSPLITS[self.sz as usize];
        for b in bitsplits.iter().cloned() {
            let n: usize = self.bits & mask(b as usize) as usize;
            if e == n {
                return true;
            } else if e < n {
                return false;
            }
            e -= n + 1;
            self.bits = self.bits >> b;
        }
        false
    }
}

#[cfg(test)]
fn test_vec(v: Vec<u64>) {
    println!("\ntesting {:?}", v);
    assert_eq!(Tiny::new_sorted_deduped(&v).unwrap().collect::<Vec<_>>(), v);
}

#[test]
fn test_tiny() {
    assert_eq!(Tiny::new_sorted_deduped(&[]), None);
    test_vec(vec![1]);
    test_vec(vec![1024]);
    test_vec(vec![1, 2]);
    test_vec(vec![1, 2, 3]);
    test_vec(vec![1, 2, 3, 4, 5]);
    test_vec(vec![1, 2, 3, 4, 5, 6]);
    test_vec(vec![1, 2, 3, 4, 5, 6, 7]);
}

enum Internal<'a> {
    Empty,
    /// This is the case where we store up to seven values in the pointer
    /// itself.  The `Tiny` data structure is a copy of that information, which
    /// also functions as an `Iterator`.
    Stack(Tiny),
    /// This is the normal storage for tiny sets.  We use effectively a hash
    /// table.  Each u32 element stores two things, a bitmap (in the least
    /// significant bits) giving the elements in the set, and in the most
    /// significant bits a value representing the offset of the bitmap.
    Heap {
        s: &'a Sbeginning,
        a: &'a [u64],
    },
    /// This is for the case where the maximum number in our set is so large
    /// that we can't use the bitmap approach above.  Intead we store a single
    /// value to represent any zero element.  We use zeros to represent empty
    /// bins, and actual values to represent numbers, except in the case where
    /// an actual zero is present in the set, in which case we store a chosen
    /// unique value (which we change if that unique value gets added to the
    /// set).
    Big {
        s: &'a Sbeginning,
        a: &'a [u64],
    },
    // The data is stored as a bitmap.  This is used when the number of elements
    // in the set is getting close enough to the maximum value of the set.
    Dense {
        sz: usize,
        a: &'a [u64],
    },
}
enum InternalMut<'a> {
    Empty,
    Stack(Tiny),
    Heap {
        s: &'a mut Sbeginning,
        a: &'a mut [u64],
    },
    Big {
        s: &'a mut Sbeginning,
        a: &'a mut [u64],
    },
    Dense {
        sz: &'a mut usize,
        a: &'a mut [u64],
    },
}

impl Extend<u64> for SetU64 {
    fn extend<T: IntoIterator<Item = u64>>(&mut self, iter: T) {
        for i in iter.into_iter() {
            self.insert(i);
        }
    }
}

#[cfg(feature = "compactserde")]
impl SetU64 {
    fn to_array(&self) -> Vec<u64> {
        let mut out = Vec::new();
        if self.0 as usize == 0 || self.0 as usize & 7 != 0 {
            out.push(self.0 as u64);
        } else {
            let s = unsafe { &*self.0 };
            let b = &s.b;
            let a = unsafe { std::slice::from_raw_parts(&s.array as *const u64, b.cap) };
            out.push(b.sz as u64);
            out.push(b.bits as u64);
            out.extend(a);
        }
        out
    }
    fn from_array(v: &[u64]) -> SetU64 {
        if v.len() > 1 {
            let cap = v.len() - 2;
            let mut set = SetU64::with_capacity_and_bits(cap, v[1]);
            match set.internal_mut() {
                InternalMut::Empty => unreachable!(),
                InternalMut::Stack(_) => unreachable!(),
                InternalMut::Dense { sz, a } => {
                    *sz = v[0] as usize;
                    for (i, o) in v[2..].iter().zip(a.iter_mut()) {
                        *o = *i;
                    }
                }
                InternalMut::Heap { s, a } => {
                    s.sz = v[0] as usize;
                    s.bits = v[1];
                    for (i, o) in v[2..].iter().zip(a.iter_mut()) {
                        *o = *i;
                    }
                }
                InternalMut::Big { s, a } => {
                    s.sz = v[0] as usize;
                    s.bits = v[1];
                    for (i, o) in v[2..].iter().zip(a.iter_mut()) {
                        *o = *i;
                    }
                }
            }
            set
        } else {
            SetU64(v[0] as *mut S)
        }
    }
}

#[cfg(feature = "compactserde")]
#[test]
fn to_from_array() {
    use std::iter::FromIterator;

    let set = SetU64::from_iter([0]);
    let s = set.to_array();
    assert_eq!(set, SetU64::from_array(&s));

    let set = SetU64::from_iter([]);
    let s = set.to_array();
    assert_eq!(set, SetU64::from_array(&s));

    let set = SetU64::from_iter([u64::MAX, u64::MAX - 100]);
    let s = set.to_array();
    let newset = SetU64::from_array(&s);
    for n in set.iter() {
        assert!(set.contains(n));
    }
    for n in newset.iter() {
        assert!(newset.contains(n));
    }
    println!("set is {set:?}");
    println!("newset is {newset:?}");
    assert_eq!(set.len(), newset.len());
    assert_eq!(set, SetU64::from_array(&s));

    let set = SetU64::from_iter(0..10000);
    let s = set.to_array();
    assert_eq!(set, SetU64::from_array(&s));
}

#[cfg(feature = "serde")]
mod serde {
    use crate::SetU64;
    use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
    use serde::ser::{Serialize, SerializeSeq, Serializer};

    impl Serialize for SetU64 {
        #[cfg(feature = "compactserde")]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let a = self.to_array();
            let mut seq = serializer.serialize_seq(Some(a.len()))?;
            for e in a.into_iter() {
                seq.serialize_element(&e)?;
            }
            seq.end()
        }
        #[cfg(not(feature = "compactserde"))]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.len()))?;
            for e in self.iter() {
                seq.serialize_element(&e)?;
            }
            seq.end()
        }
    }

    // A Visitor is a type that holds methods that a Deserializer can drive
    // depending on what is contained in the input data.
    //
    // In the case of a map we need generic type parameters K and V to be
    // able to set the output type correctly, but don't require any state.
    // This is an example of a "zero sized type" in Rust. The PhantomData
    // keeps the compiler from complaining about unused generic type
    // parameters.
    struct SetVisitor;

    impl SetVisitor {
        fn new() -> Self {
            SetVisitor
        }
    }

    // This is the trait that Deserializers are going to be driving. There
    // is one method for each type of data that our type knows how to
    // deserialize from. There are many other methods that are not
    // implemented here, for example deserializing from integers or strings.
    // By default those methods will return an error, which makes sense
    // because we cannot deserialize a MyMap from an integer or string.
    impl<'de> Visitor<'de> for SetVisitor {
        // The type that our Visitor is going to produce.
        type Value = SetU64;

        // Format a message stating what data this Visitor expects to receive.
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a set of usize")
        }

        #[cfg(feature = "compactserde")]
        fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: SeqAccess<'de>,
        {
            let mut v = if let Some(cap) = access.size_hint() {
                Vec::with_capacity(cap)
            } else {
                Vec::new()
            };
            // While there are entries remaining in the input, add them
            // into our map.
            while let Some(elem) = access.next_element()? {
                v.push(elem);
            }
            Ok(SetU64::from_array(&v))
        }
        #[cfg(not(feature = "compactserde"))]
        fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: SeqAccess<'de>,
        {
            let mut set = SetU64::new();

            // While there are entries remaining in the input, add them
            // into our map.
            while let Some(elem) = access.next_element()? {
                set.insert(elem);
            }

            Ok(set)
        }
    }

    // This is the trait that informs Serde how to deserialize MyMap.
    impl<'de> Deserialize<'de> for SetU64 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Instantiate our Visitor and ask the Deserializer to drive
            // it over the input data, resulting in an instance of MyMap.
            deserializer.deserialize_seq(SetVisitor::new())
        }
    }

    #[test]
    fn serialize_deserialize() {
        use std::iter::FromIterator;

        let set = SetU64::from_iter([0]);
        let s = serde_json::to_string(&set).unwrap();
        assert_eq!(set, serde_json::from_str(&s).unwrap());

        let set = SetU64::from_iter([]);
        let s = serde_json::to_string(&set).unwrap();
        assert_eq!(set, serde_json::from_str(&s).unwrap());

        let set = SetU64::from_iter([u64::MAX, u64::MAX - 100]);
        let s = serde_json::to_string(&set).unwrap();
        assert_eq!(set, serde_json::from_str(&s).unwrap());

        let set = SetU64::from_iter(0..10000);
        let s = serde_json::to_string(&set).unwrap();
        assert_eq!(set, serde_json::from_str(&s).unwrap());
    }
}

impl Clone for SetU64 {
    fn clone(&self) -> Self {
        if self.0 as usize & 7 == 0 && self.0 != std::ptr::null_mut() {
            let c = self.capacity();
            unsafe {
                let ptr = std::alloc::alloc_zeroed(layout_for_capacity(c)) as *mut S;
                if ptr.is_null() {
                    // It is safe to panic here rather than calling `alloc::handle_alloc_error`
                    // because we haven't  even started creating this data structure, so
                    // `catch_unwind` can't get us into trouble.
                    panic!("memory allocation failed");
                }
                std::ptr::copy_nonoverlapping(
                    self.0 as *const u8,
                    ptr as *mut u8,
                    bytes_for_capacity(c),
                );
                SetU64(ptr)
            }
        } else {
            SetU64(self.0)
        }
    }
}

impl SetU64 {
    /// Create an empty set with capacity to hold the provided set.
    ///
    /// ```
    /// let a: tinyset::SetU64 = (1..300).collect();
    /// let mut b = tinyset::SetU64::with_capacity_of(&a);
    ///
    /// assert_eq!(a.capacity(), b.capacity());
    /// assert_eq!(b.len(), 0);
    /// for i in a.iter() {
    ///   b.insert(i);
    /// }
    /// assert_eq!(a.capacity(), b.capacity());
    /// assert_eq!(b.len(), a.len());
    /// ```
    pub fn with_capacity_of(other: &Self) -> Self {
        if other.0 as usize & 7 == 0 && other.0 != std::ptr::null_mut() {
            let c = other.capacity();
            unsafe {
                let ptr = std::alloc::alloc_zeroed(layout_for_capacity(c)) as *mut S;
                if ptr.is_null() {
                    // It is safe to panic here rather than calling `alloc::handle_alloc_error`
                    // because we haven't  even started creating this data structure, so
                    // `catch_unwind` can't get us into trouble.
                    panic!("memory allocation failed");
                }
                (*ptr).b.cap = (*other.0).b.cap;
                (*ptr).b.bits = (*other.0).b.bits;
                SetU64(ptr)
            }
        } else {
            SetU64::new()
        }
    }
}

#[test]
fn just_clone() {
    let mut x = SetU64::with_capacity_and_max(100, 1000000);
    x.insert(100);
    x.insert(1000);
    let y = x.clone();
    assert_eq!(x.len(), y.len());
    assert_eq!(x.len(), x.into_iter().count());
    assert_eq!(y.len(), y.clone().into_iter().count());
}

impl SetU64 {
    /// The number of elements in the set
    #[inline]
    pub fn len(&self) -> usize {
        match self.internal() {
            Internal::Empty => 0,
            Internal::Stack(t) => t.sz as usize,
            Internal::Heap { s, .. } => s.sz,
            Internal::Big { s, .. } => s.sz,
            Internal::Dense { sz, .. } => sz,
        }
    }
    /// The capacity of the set
    #[inline]
    pub fn capacity(&self) -> usize {
        match self.internal() {
            Internal::Empty => 0,
            Internal::Stack(_) => 0,
            Internal::Heap { a, .. } => a.len(),
            Internal::Big { a, .. } => a.len(),
            Internal::Dense { a, .. } => a.len(),
        }
    }
    /// Print debugging information about this set.
    pub fn debug_me(&self, msg: &str) {
        match self.internal() {
            Internal::Empty => println!("empty set: {}", msg),
            Internal::Stack(t) => println!("{}: stack {:?} => {:?}", msg, t, t.collect::<Vec<_>>()),
            Internal::Heap { s, a } => {
                println!("{}: heap {:?}", msg, s);
                for (i, x) in a.iter().cloned().enumerate() {
                    println!(
                        "      {} (key {} pov {}): {:0b} ({})",
                        (x >> s.bits) * s.bits,
                        x >> s.bits,
                        p_poverty(x >> s.bits, i, a.len()),
                        x & mask(s.bits as usize),
                        x
                    );
                }
                println!("    => {:?}", self.iter().collect::<Vec<_>>());
            }
            Internal::Big { s, a } => {
                println!("{}: big {:?}\n    {:?}", msg, s, a);
                let v: Vec<_> = a.iter().cloned().map(|x| x % a.len() as u64).collect();
                println!("     >>>{:?}", v);
            }
            Internal::Dense { sz, a } => {
                println!(
                    "{}: dense {:?}\n    {:?}\n    => {:?}",
                    msg,
                    sz,
                    a,
                    self.iter().collect::<Vec<u64>>()
                );
                println!("    foo {:?}", self.iter());
            }
        }
    }
    /// Tally up how much memory is in use.
    #[inline]
    pub fn mem_used(&self) -> usize {
        match self.internal() {
            Internal::Empty => std::mem::size_of::<Self>(),
            Internal::Stack(_) => std::mem::size_of::<Self>(),
            Internal::Heap { s, .. } => std::mem::size_of::<Self>() + s.cap * 8 - 8,
            Internal::Dense { a, .. } => std::mem::size_of::<Self>() + a.len() * 8 - 8,
            Internal::Big { s, .. } => std::mem::size_of::<Self>() + s.cap * 8 - 8,
        }
    }
    /// The number of words a dense set with maximum `mx` has.
    fn dense_capacity(mx: u64) -> u64 {
        1 + mx / 64 + mx / 256
    }
    fn dense_with_max(mx: u64) -> SetU64 {
        let cap = SetU64::dense_capacity(mx);
        // This should be stored in a dense bitset.
        unsafe {
            let ptr = std::alloc::alloc_zeroed(layout_for_capacity(cap as usize)) as *mut S;
            if ptr.is_null() {
                // It is safe to panic here rather than calling `alloc::handle_alloc_error`
                // because we haven't  even started creating this data structure, so
                // `catch_unwind` can't get us into trouble.
                panic!("memory allocation failed");
            }
            let x = SetU64(ptr);
            (*x.0).b.cap = cap as usize;
            (*x.0).b.bits = 64;
            x
        }
    }

    /// Create a set with the given capacity
    pub fn with_capacity_and_max(cap: usize, mx: u64) -> SetU64 {
        if cap as u64 > mx >> 7 {
            SetU64::dense_with_max(mx)
        } else {
            SetU64::with_capacity_and_bits(cap, compute_array_bits(mx))
        }
    }
    /// Create a set with the given capacity and bits
    pub fn with_capacity_and_bits(cap: usize, bits: u64) -> SetU64 {
        if cap > 0 {
            unsafe {
                let ptr = std::alloc::alloc_zeroed(layout_for_capacity(cap)) as *mut S;
                if ptr.is_null() {
                    // It is safe to panic here rather than calling `alloc::handle_alloc_error`
                    // because we haven't  even started creating this data structure, so
                    // `catch_unwind` can't get us into trouble.
                    panic!("memory allocation failed");
                }
                let x = SetU64(ptr);
                (*x.0).b.cap = cap;
                (*x.0).b.bits = if bits == 0 {
                    let mut b = 0;
                    while b <= 64 {
                        b = crate::rand::rand64(cap, bits);
                    }
                    b
                } else {
                    bits
                };
                x
            }
        } else {
            SetU64(0 as *mut S)
        }
    }
    /// An empty set
    #[inline]
    pub const fn new() -> Self {
        SetU64(0 as *mut S)
    }

    /// Insert and return true if it was not present.
    pub fn insert(&mut self, e: u64) -> bool {
        self.insert_inner(e, false)
    }

    /// Insert, and return true if it was not present.  If `reserved`, the caller guarantees that
    /// the table, if it is one, has room for every element it is going to insert, so there is no
    /// need to check that it has, or to grow it.
    fn insert_inner(&mut self, e: u64, reserved: bool) -> bool {
        match self.internal_mut() {
            InternalMut::Empty => {
                if let Some(t) = Tiny::from_singleton(e) {
                    *self = SetU64(t.to_usize() as *mut S);
                    return true;
                }
                // println!("I could not create tiny set with singleton {}", e);
                *self = Self::with_capacity_and_max(1, e);
            }
            InternalMut::Stack(t) => {
                if let Some(newt) = t.insert(e) {
                    *self = SetU64(newt.to_usize() as *mut S);
                    return newt.sz != t.sz;
                }
                let mx = t.max().unwrap();
                let mx = if e > mx { e } else { mx };
                *self = Self::with_capacity_and_max(t.sz as usize + 1, mx);
                // self.debug_me("empty array");
                for x in t {
                    self.insert(x);
                    // self.debug_me(&format!("   ...after inserting {}", x));
                }
                self.insert(e);
                return true;
            }
            _ => (),
        }
        match self.internal_mut() {
            InternalMut::Empty => unreachable!(),
            InternalMut::Stack(_) => unreachable!(),
            InternalMut::Dense { sz, a } => {
                let key = (e >> 6) as usize;
                if let Some(bits) = a.get_mut(key) {
                    let whichbit = 1 << (e & 63);
                    let present = *bits & whichbit != 0;
                    *bits = *bits | whichbit;
                    if !present {
                        *sz = *sz + 1;
                    }
                    !present
                } else {
                    // println!("key is {}", key);
                    if key > 128 * (*sz as usize) {
                        // It is getting sparse, so let us switch back
                        // to a non-hash table.
                        let cap = 2 * (*sz + 1);
                        let mut new = SetU64::with_capacity_and_bits(cap as usize, 0);
                        for x in self.iter() {
                            new.insert(x);
                        }
                        new.insert(e);
                        *self = new;
                    } else {
                        let mut new = SetU64::with_capacity_and_bits(1 + key + key / 4, 64);
                        match new.internal_mut() {
                            InternalMut::Empty => unreachable!(),
                            InternalMut::Stack(_) => unreachable!(),
                            InternalMut::Big { .. } => unreachable!(),
                            InternalMut::Heap { .. } => unreachable!(),
                            InternalMut::Dense { sz: newsz, a: na } => {
                                for (i, v) in a.iter().cloned().enumerate() {
                                    na[i] = v;
                                }
                                na[key] = 1 << (e & 63);
                                *newsz = *sz + 1;
                            }
                        }
                        *self = new;
                    }
                    true
                }
            }
            InternalMut::Heap { s, a } => {
                if compute_array_bits(e) < s.bits {
                    let mut new = Self::with_capacity_and_bits(
                        s.cap + 1 + 2 * (crate::rand::rand_usize(s.cap, s.bits) % s.cap),
                        compute_array_bits(e),
                    );
                    // new.debug_me("\n\nnew set");
                    for d in self.iter() {
                        new.insert(d);
                        // new.debug_me(&format!("\n -- after inserting {}", d));
                    }
                    new.insert(e);
                    // new.debug_me(&format!("\n -- after inserting {}", e));
                    *self = new;
                    return true;
                }
                let (key, offset) = split_u64(e, s.bits);
                match p_lookfor(key, a, s.bits) {
                    LookedUp::KeyFound(idx) => {
                        if a[idx] & (1 << offset) != 0 {
                            return false;
                        } else {
                            a[idx] = a[idx] | (1 << offset);
                            s.sz += 1;
                            return true;
                        }
                    }
                    LookedUp::EmptySpot(idx) => {
                        a[idx] = key << s.bits | 1 << offset;
                        s.sz += 1;
                        return true;
                    }
                    LookedUp::NeedInsert => {}
                }
                // println!("looking for space in sparse... {:?}", a);
                if reserved || p_has_room(key, a) {
                    let idx = p_insert(key, a, s.bits);
                    // println!("about to insert key {} with elem {} at {}",
                    //          key, e, idx);
                    a[idx] = (key << s.bits) | (1 << offset);
                    s.sz += 1;
                    return true;
                }
                // println!("no room in the sparse set... {:?}", a);
                // We'll have to expand the set.
                let mx = a
                    .iter()
                    .cloned()
                    .map(|x| (x >> s.bits) * s.bits + s.bits)
                    .max()
                    .unwrap();
                let mx = if e > mx { e } else { mx };
                if s.cap as u64 > mx >> 6 {
                    // A dense set will save memory
                    let mut new = Self::dense_with_max(mx);
                    for x in self.iter() {
                        new.insert(x);
                    }
                    new.insert(e);
                    *self = new;
                } else {
                    // Let's keep things sparse
                    // A dense set will cost us memory
                    let newcap = grown_capacity(s.cap, s.bits);
                    let mut new = Self::with_capacity_and_bits(newcap, s.bits);
                    // new.debug_me("initial new");
                    for v in self.iter() {
                        new.insert(v);
                    }
                    new.insert(e);
                    *self = new;
                }
                true
            }
            InternalMut::Big { s, a } => {
                if e == s.bits {
                    // Pick a new number not present in the set.  We
                    // use a cryptographically secure random number
                    // generator to look for a new number not present,
                    // which feels like overkill.  But the cost of
                    // changing the "bits" is $O(N)$, so it's worth
                    // a high O(1) cost to reduce collisions.
                    let had_zero = p_remove(s.bits, a, 0);
                    loop {
                        let i: u64 = crate::rand::rand64(s.cap, s.bits);
                        if i > 64 && !a.iter().any(|&v| v == i) {
                            s.bits = i;
                            break;
                        }
                    }
                    if had_zero {
                        a[p_insert(s.bits, a, 0)] = s.bits;
                    }
                }
                let e = if e == 0 { s.bits } else { e };
                match p_lookfor(e, a, 0) {
                    LookedUp::KeyFound(_) => {
                        return false;
                    }
                    LookedUp::EmptySpot(idx) => {
                        a[idx] = e;
                        s.sz += 1;
                        return true;
                    }
                    LookedUp::NeedInsert => (),
                }
                // println!("looking for space in... {:?}", a);
                if reserved || p_has_room(e, a) {
                    // println!("about to insert at {}", p_insert(e, a, 0));
                    a[p_insert(e, a, 0)] = e;
                    s.sz += 1;
                    return true;
                }
                // println!("no room in the set... {:?}", a);
                let newcap = grown_capacity(s.cap, s.bits);
                let mut new = Self::with_capacity_and_bits(newcap, s.bits);
                // new.debug_me("initial new");
                match new.internal_mut() {
                    InternalMut::Empty => unreachable!(),
                    InternalMut::Stack(_) => unreachable!(),
                    InternalMut::Dense { .. } => unreachable!(),
                    InternalMut::Heap { .. } => unreachable!(),
                    InternalMut::Big { s: ns, a: na } => {
                        for v in a.iter().cloned().filter(|&x| x != 0) {
                            na[p_insert(v, na, 0)] = v;
                            // println!("  with {} gives {:?}", v, na);
                            // let v: Vec<_> = na.iter().cloned().map(|x| x % na.len() as u64).collect();
                            // println!("     >>>{:?}", v);
                        }
                        na[p_insert(e, na, 0)] = e;
                        // println!("  with {} gives {:?}", e, na);
                        ns.sz = s.sz + 1;
                        // println!("  size ends up as {}", ns.sz);
                        // new.debug_me("aftr growing");
                    }
                }
                *self = new;
                true
            }
        }
    }

    /// Remove
    pub fn remove(&mut self, e: u64) -> bool {
        match self.internal_mut() {
            InternalMut::Empty => false,
            InternalMut::Stack(t) => {
                if t.clone().any(|x| x == e) {
                    let sz = t.sz - 1;
                    if sz == 0 {
                        *self = SetU64(0 as *mut S);
                    } else {
                        *self = t.filter(|&x| x != e).collect();
                    }
                    true
                } else {
                    false
                }
            }
            InternalMut::Dense { sz, a } => {
                let key = e >> 6;
                if let Some(bits) = a.get_mut(key as usize) {
                    let whichbit = 1 << (e & 63);
                    let present = *bits & whichbit != 0;
                    *bits = *bits & !whichbit;
                    if present {
                        *sz = *sz - 1;
                    }
                    present
                } else {
                    false
                }
            }
            InternalMut::Heap { s, a } => {
                if compute_array_bits(e) < s.bits {
                    return false;
                }
                let (key, offset) = split_u64(e, s.bits);
                if let LookedUp::KeyFound(idx) = p_lookfor(key, a, s.bits) {
                    if a[idx] & (1 << offset) != 0 {
                        let newa = a[idx] & !(1 << offset);
                        s.sz -= 1;
                        if newa == key << s.bits {
                            // We've removed everything with this key,
                            // so remove the whole key!
                            p_remove(key, a, s.bits);
                        } else {
                            a[idx] = newa;
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            InternalMut::Big { s, a } => {
                if e == s.bits {
                    return false;
                }
                let e = if e == 0 { s.bits } else { e };
                let had_e = p_remove(e, a, 0);
                if had_e {
                    s.sz -= 1;
                }
                had_e
            }
        }
    }

    /// Contais
    pub fn contains(&self, e: u64) -> bool {
        match self.internal() {
            Internal::Empty => false,
            Internal::Stack(t) => t.contains(e), // t.clone().any(|x| x == e),
            Internal::Dense { a, .. } => {
                let key = e >> 6;
                if let Some(bits) = a.get(key as usize) {
                    bits & (1 << (e & 63)) != 0
                } else {
                    false
                }
            }
            Internal::Heap { s, a } => {
                if compute_array_bits(e) < s.bits {
                    // println!("too big a thing");
                    return false;
                }
                let (key, offset) = split_u64(e, s.bits);
                if let LookedUp::KeyFound(idx) = p_lookfor(key, a, s.bits) {
                    a[idx] & (1 << offset) != 0
                } else {
                    // self.debug_me(&format!("did not find key {} from {}", key, e));
                    false
                }
            }
            Internal::Big { s, a } => {
                if e == s.bits {
                    return false;
                }
                let e = if e == 0 { s.bits } else { e };
                p_lookfor(e, a, 0).key_found()
            }
        }
    }

    /// Clears the set, returning all elements in an iterator.
    #[inline]
    pub fn drain<'a>(&mut self) -> impl Iterator<Item = u64> + 'static {
        std::mem::replace(self, SetU64::new()).into_iter()
    }

    #[inline]
    fn internal<'a>(&'a self) -> Internal<'a> {
        if self.0 as usize == 0 {
            Internal::Empty
        } else if self.0 as usize & 7 != 0 {
            Internal::Stack(Tiny::from_usize(self.0 as usize))
        } else {
            let s = unsafe { &*self.0 };
            let b = &s.b;
            let array = unsafe {
                // Use the calculated offset to jump the pointer to where the array starts directly, keeping the permissions for the whole allocation intact.
                self.0.cast::<u64>().offset(
                    // get a raw pointer to the array field
                    (&s.array as *const u64)
                        // calculate the offset from `*mut S` to the array field
                        .offset_from(self.0.cast()),
                )
            };
            let a = unsafe { std::slice::from_raw_parts(array, b.cap as usize) };
            if b.bits == 0 || b.bits > 64 {
                Internal::Big { s: b, a }
            } else if b.bits == 64 {
                Internal::Dense { sz: b.sz, a }
            } else {
                Internal::Heap { s: b, a }
            }
        }
    }

    fn internal_mut<'a>(&'a mut self) -> InternalMut<'a> {
        if self.0 as usize == 0 {
            InternalMut::Empty
        } else if self.0 as usize & 7 != 0 {
            InternalMut::Stack(Tiny::from_usize(self.0 as usize))
        } else {
            let s = unsafe { &mut *self.0 };
            let b = &mut s.b;
            let array = unsafe {
                // Use the calculated offset to jump the pointer to where the array starts directly, keeping the permissions for the whole allocation intact.
                self.0.cast::<u64>().offset(
                    // get a raw pointer to the array field
                    (&mut s.array as *mut u64)
                        // calculate the offset from `*mut S` to the array field
                        .offset_from(self.0.cast()),
                )
            };
            let a = unsafe { std::slice::from_raw_parts_mut(array, b.cap as usize) };
            if b.bits == 0 || b.bits > 64 {
                InternalMut::Big { s: b, a }
            } else if b.bits == 64 {
                InternalMut::Dense { sz: &mut b.sz, a }
            } else {
                InternalMut::Heap { s: b, a }
            }
        }
    }
}

#[cfg(test)]
impl crate::copyset::CopySet for SetU64 {
    type Item = u64;
    type Iter = IntoIter;
    fn ins(&mut self, e: u64) -> bool {
        self.insert(e)
    }
    fn rem(&mut self, e: u64) -> bool {
        self.remove(e)
    }
    fn con(&self, e: u64) -> bool {
        self.contains(e)
    }
    fn vec(&self) -> Vec<u64> {
        self.iter().collect()
    }
    fn ln(&self) -> usize {
        self.len()
    }
    fn it(self) -> Self::Iter {
        self.into_iter()
    }
}

impl Default for SetU64 {
    fn default() -> Self {
        SetU64(0 as *mut S)
    }
}

impl std::iter::FromIterator<u64> for SetU64 {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u64>,
    {
        let mut v: Vec<_> = iter.into_iter().collect();
        let mx = if let Some(mx) = v.iter().cloned().max() {
            mx
        } else {
            return SetU64(0 as *mut S);
        };
        if v.len() < BITSPLITS.len() {
            // A few small elements might fit in a single word.
            v.sort();
            v.dedup();
            if let Some(t) = Tiny::new_sorted_deduped(&v) {
                return SetU64(t.to_usize() as *mut S);
            }
        }
        if SetU64::dense_capacity(mx) <= v.len() as u64 * 11 / 10 {
            // A bitmap takes no more memory than a table would (and a table this
            // full of keys that are this close together is slow), and needs no
            // sorting to fill.
            let mut s = SetU64::dense_with_max(mx);
            for value in v.into_iter() {
                s.insert(value);
            }
            return s;
        }
        // Sorting lets us insert in increasing order.
        v.sort();
        v.dedup();
        let bits = compute_array_bits(mx);
        if bits == 0 {
            let mut s = SetU64::with_capacity_and_bits(v.len(), bits);
            for value in v.into_iter() {
                s.insert_inner(value, true);
            }
            s
        } else {
            let mut keys: Vec<_> = v.iter().map(|&x| x / bits).collect();
            keys.sort();
            keys.dedup();
            let sz = (keys.len() + 1) * 11 / 10;
            let mut s = SetU64::with_capacity_and_bits(sz, bits);
            for value in v.into_iter() {
                s.insert_inner(value, true);
            }
            s
        }
    }
}

#[cfg(test)]
fn test_a_collect(v: Vec<u64>) {
    let s: SetU64 = v.iter().cloned().collect();
    let vv: Vec<_> = s.iter().collect();
    let ss: SetU64 = vv.iter().cloned().collect();
    let vvv: Vec<_> = ss.iter().collect();
    assert_eq!(vv, vvv);
}

#[test]
#[should_panic(expected = "too large")]
fn test_alloc_failure() {
    SetU64::with_capacity_and_bits(usize::MAX / 8 - 2, 0);
}

#[test]
fn test_last_is_last_element() {
    // `last` has its own implementation for each format of set, and it must agree
    // with the last element that iterating yields.  It once returned a value that
    // was not even in the set.
    for (n, max) in [
        (5usize, 5_000u64),
        (10, 10_000),
        (100, 2_000),
        (100, 1_000_000),
        (1000, 20_000),
        (1000, 1_000_000),
    ] {
        let mut x: u64 = 12345;
        let mut v = Vec::new();
        for _ in 0..n {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            v.push((x >> 33) % max);
        }
        let collected: SetU64 = v.iter().cloned().collect();
        let mut inserted = SetU64::new();
        for &e in &v {
            inserted.insert(e);
        }
        for s in [&collected, &inserted] {
            let all: Vec<u64> = s.iter().collect();
            assert_eq!(s.iter().last(), all.last().cloned(), "n={} max={}", n, max);
            assert_eq!(s.iter().max(), all.iter().cloned().max());
            assert_eq!(s.iter().min(), all.iter().cloned().min());
        }
    }
}

#[test]
fn test_collect() {
    test_a_collect(vec![]);
    test_a_collect(vec![0]);
    test_a_collect(vec![0, 1 << 60]);
    test_a_collect(vec![0, 1 << 30, 1 << 60]);
    test_a_collect((0..1024).collect());
}

/// Returns the number of bytes needed for a set with capacity `sz`.
///
/// Panics if that size overflows or exceeds `isize::MAX`, since in release
/// mode wrapping arithmetic would otherwise produce a bogus (possibly zero)
/// allocation size.
fn bytes_for_capacity(sz: usize) -> usize {
    match sz
        .checked_mul(8)
        .and_then(|b| b.checked_add(std::mem::size_of::<S>() - 8))
    {
        Some(size) if size <= isize::MAX as usize => size,
        _ => panic!("tinyset size is too large: {}", sz),
    }
}
fn layout_for_capacity(sz: usize) -> std::alloc::Layout {
    std::alloc::Layout::from_size_align(bytes_for_capacity(sz), 8)
        .unwrap_or_else(|_| panic!("tinyset size is too large: {}", sz))
}

impl Drop for SetU64 {
    fn drop(&mut self) {
        if self.0 as usize > 0 {
            // make it drop by moving it out
            let c = self.capacity();
            if c == 0 {
            } else {
                unsafe {
                    std::alloc::dealloc(self.0 as *mut u8, layout_for_capacity(c));
                }
            }
        }
    }
}

#[cfg(test)]
impl heapsize::HeapSizeOf for SetU64 {
    fn heap_size_of_children(&self) -> usize {
        match self.internal() {
            Internal::Empty => 0,
            Internal::Stack(_) => 0,
            Internal::Heap { a, .. } => std::mem::size_of::<S>() - 8 + a.len() * 8,
            Internal::Big { a, .. } => std::mem::size_of::<S>() - 8 + a.len() * 8,
            Internal::Dense { a, .. } => std::mem::size_of::<S>() - 8 + a.len() * 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapsize::HeapSizeOf;

    fn check_set(elems: &[u64]) {
        println!("\n\ncheck_set {:?}\n", elems);
        let mut s = SetU64::default();
        s.debug_me("default set");
        let mut count = 0;
        for x in elems.iter().cloned() {
            let was_here = s.contains(x);
            s.debug_me(&format!("\n\n\nabout to insert {}", x));
            let changed_something = s.insert(x);
            s.debug_me(&format!("\n\nafter inserting {}", x));
            if changed_something {
                count += 1;
                println!("    {} is new now count {}", x, count);
            }
            assert_eq!(!was_here, changed_something);
            s.debug_me(&format!("after inserting {} length is {}", x, s.len()));
            println!("what is this? count {} does it have {}?", count, x);
            assert!(s.contains(x));
            assert_eq!(s.len(), count);
            assert_eq!(s.iter().count(), count);
        }
        assert!(elems.len() >= s.len());
        assert_eq!(elems.iter().cloned().min(), s.iter().min());
        println!("set {:?} with length {}", elems, s.len());
        for x in s.iter() {
            println!("    {}", x);
        }
        s.debug_me("finally");
        assert_eq!(s.iter().count(), s.len());
        for x in s.iter() {
            println!("looking for {}", x);
            assert!(elems.contains(&x));
        }
        for x in s.iter() {
            println!("found {}", x);
            assert!(elems.contains(&x));
        }
        for x in elems.iter().cloned() {
            println!("YYYY looking for {}", x);
            assert!(s.contains(x));
        }
        for x in elems.iter().cloned() {
            println!("removing {}", x);
            s.remove(x);
            s.debug_me("  after remove");
        }
        for x in elems.iter().cloned() {
            println!("XXXX looking for {}", x);
            assert!(!s.contains(x));
        }
        s.debug_me("after everything was removed");
        assert_eq!(s.len(), 0);
        check_size(elems);
    }

    #[test]
    fn check_sets() {
        check_set(&[
            2020521336280004635,
            17919264261434241137,
            2238430514865874295,
            6993057942733118921,
            151320868361192487,
        ]);

        check_set(&[
            1121917459475225854,
            2080724155257001326,
            4615424731220156355,
            0,
        ]);

        check_set(&[
            12891372448885225674,
            7003397808690412416,
            129282323776365774,
            9248739364838008708,
        ]);

        check_set(&[]);
        check_set(&[10]);
        check_set(&[1024]);
        check_set(&[1]);
        check_set(&[0]);
        check_set(&[0, 1]);
        check_set(&[0, 1]);

        check_set(&[0, 1, 2, 3]);
        check_set(&[0, 1, 0, 2, 3]);

        check_set(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        check_set(&[0, 1, 2, 3, 4, 5, 6, 7 << 30, 8, 9, 10]);

        check_set(&[0, 2097153, 2, 2305843009213693952]);

        check_set(&[0, 1024]);
        check_set(&[0, 1024, 1 << 61]);
        check_set(&[0, 1024, 1 << 63]);
    }

    fn check_tiny_from_vec(slice: Vec<u64>) {
        println!("\n\n\ncheck_tiny_from_vec({:?})", slice);
        let mut t = if let Some(t) = Tiny::from_singleton(slice[0]) {
            t
        } else {
            return; // hokey, but avoids a crash
        };
        t.debug_me("to start with");
        let mut count = 1;
        let mut included = Vec::new();
        for x in slice.iter().cloned() {
            t.debug_me(&format!("starting in on {}", x));
            assert_eq!(t.clone().any(|e| e == x), t.contains(x));
            let already = t.clone().any(|e| e == x);
            let next = t.insert(x);
            if let Some(tt) = next {
                tt.debug_me("    inserting gives");
                assert_eq!(tt.sz == t.sz, already);
                if tt.sz != t.sz {
                    count += 1;
                    included.push(x);
                }
                t = tt;
                t.debug_me("hello");
                assert!(t.clone().any(|e| e == x));
                assert!(t.insert(x).is_some()); // inserting a second time must succeeed
                t.debug_me("hello again");
                assert_eq!(t.sz, count);
                for xx in t.clone() {
                    assert!(t.contains(xx));
                    assert!(t.clone().any(|x| x == xx));
                }
            } else {
                assert!(!already);
            }
            t.debug_me("at end of loop");
        }
        for x in included.iter().cloned() {
            assert!(t.contains(x));
        }
    }
    #[test]
    fn check_specific_tinies() {
        check_tiny_from_vec(vec![49, 50, 1]);
        check_tiny_from_vec(vec![1]);
        check_tiny_from_vec(vec![1, 2]);
        check_tiny_from_vec(vec![1000000, 1000030]);
        check_tiny_from_vec(vec![3000027656, 3000030504]);
        check_tiny_from_vec(vec![3127827656, 3125730504]);
        check_tiny_from_vec(vec![3127827656, 3125730504, 1]);
        check_tiny_from_vec(vec![2, 3, 143, 251, 1, 130, 251]);
        check_tiny_from_vec(vec![1, 130, 131, 132, 133, 251]);
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn copycheck_random_sets(slice in prop::collection::vec(1u64..5, 1usize..10)) {
            crate::copyset::check_set::<SetU64>(&slice);
        }
        #[test]
        fn copycheck_medium_sets(slice in prop::collection::vec(1u64..255, 1usize..100)) {
            crate::copyset::check_set::<SetU64>(&slice);
        }
        #[test]
        fn copycheck_big_sets(slice: Vec<u64>) {
            crate::copyset::check_set::<SetU64>(&slice);
        }
    }
    proptest! {
        #[test]
        fn check_random_sets(slice in prop::collection::vec(1u64..5, 1usize..10)) {
            check_set(&slice);
        }
        #[test]
        fn check_medium_sets(slice in prop::collection::vec(1u64..255, 1usize..100)) {
            check_set(&slice);
        }
        #[test]
        fn check_big_sets(slice: Vec<u64>) {
            check_set(&slice);
        }
        #[test]
        fn check_tiny(slice in prop::collection::vec(1u64..0xffffffff, 1usize..9)) {
            check_tiny_from_vec(slice);
        }
    }
    #[test]
    fn check_specific_sets() {
        check_set(&[
            2847318633310315892,
            63058418965769059,
            2042910419467651321,
            1999840121589118486,
            16041957357413958548,
            3644150915196633528,
            11391567487966916668,
            2789376712080388913,
            8889702475440805467,
            16888113214698725429,
            1249634136756270040,
            15461625332902556004,
            8159161795026448273,
            12009229139422836646,
            15912096166435473807,
        ]);
    }

    fn total_size_of<T: HeapSizeOf>(x: &T) -> usize {
        std::mem::size_of::<T>() + x.heap_size_of_children()
    }

    fn check_size(v: &[u64]) {
        let s: SetU64 = v.iter().cloned().collect();
        let hs: std::collections::HashSet<_> = v.iter().cloned().collect();
        // let bs: std::collections::BTreeSet<_> = v.iter().cloned().collect();
        println!("setu64: {}", total_size_of(&s));
        println!("hashst: {}", total_size_of(&hs));
        // println!("btrees: {}", total_size_of(&bs));
        assert!(total_size_of(&s) < total_size_of(&hs));
        // assert!(total_size_of(&s) < total_size_of(&bs));
    }

    #[cfg(test)]
    fn collect_size_is(v: &[u64], sz: usize) {
        let s: SetU64 = v.iter().cloned().collect();
        println!("collect_size_is {:?} == {} =? {}", v, total_size_of(&s), sz);
        println!("    num elements = {}", s.len());
        assert_eq!(total_size_of(&s), sz);
    }

    #[cfg(test)]
    fn incremental_size_le(v: &[u64], sz: usize) {
        // repeat the size tests because our expansion is random in a
        // lame attempt to foil DOS attacks.
        for _ in 1..1000 {
            let sc: SetU64 = v.iter().cloned().collect();
            if total_size_of(&sc) > sz {
                println!("collect_size {:?} = {} > {}", v, total_size_of(&sc), sz);
            }
            assert!(total_size_of(&sc) <= sz);

            let mut s = SetU64::new();
            let mut vv = Vec::new();
            for x in v.iter().cloned() {
                s.insert(x);
                vv.push(x);
                if total_size_of(&s) > sz {
                    println!("incremental_size {:?} = {} > {}", vv, total_size_of(&s), sz);
                }
                assert!(total_size_of(&s) <= sz);
            }
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<S>(), 32);
        collect_size_is(&[0], 8);
        incremental_size_le(&[0], 8);
        collect_size_is(&[], 8);
        incremental_size_le(&[], 8);
        collect_size_is(&[3000], 8);
        collect_size_is(&[1, 2, 3, 4, 5, 6], 8);
        incremental_size_le(&[1, 2, 3, 4, 5, 6], 8);
        collect_size_is(&[1000, 1002, 1004, 1006, 1008], 8);
        incremental_size_le(&[1000, 1002, 1004, 1006, 1008], 8);
        collect_size_is(&[255, 260, 265, 270, 275, 280, 285], 8);
        collect_size_is(&[1000, 1002, 1004, 1006, 1008, 1009, 1010], 8);

        incremental_size_le(&(0..7).collect::<Vec<_>>(), 8);
        incremental_size_le(&(10..10 + 7).collect::<Vec<_>>(), 8);
        incremental_size_le(&(100..100 + 7).collect::<Vec<_>>(), 8);
        incremental_size_le(&(1000..1000 + 7).collect::<Vec<_>>(), 8);
        incremental_size_le(&(10000..10000 + 7).collect::<Vec<_>>(), 8);
        incremental_size_le(&(100000..100000 + 7).collect::<Vec<_>>(), 8);

        incremental_size_le(&(1..30).collect::<Vec<_>>(), 40);
        incremental_size_le(&(1..60).collect::<Vec<_>>(), 40);

        incremental_size_le(&(1..160).collect::<Vec<_>>(), 88);

        incremental_size_le(&(0..100).map(|x| x * 10).collect::<Vec<_>>(), 400);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<S>(), 24);
        collect_size_is(&[0], 4);
        incremental_size_le(&[0], 4);
        collect_size_is(&[], 4);
        incremental_size_le(&[], 4);
        collect_size_is(&[3000], 4);
        collect_size_is(&[1, 2, 3, 4, 5, 6], 4);
        incremental_size_le(&[1, 2, 3, 4, 5, 6], 4);
        collect_size_is(&[1000, 1002, 1004, 1006, 1008], 4);
        incremental_size_le(&[1000, 1002, 1004, 1006, 1008], 4);

        collect_size_is(&[255, 260, 265, 270, 275, 280, 285], 4);

        incremental_size_le(&(1..30).collect::<Vec<_>>(), 32);
        incremental_size_le(&(1..60).collect::<Vec<_>>(), 32);

        incremental_size_le(&(1..160).collect::<Vec<_>>(), 84);

        incremental_size_le(&(0..100).map(|x| x * 10).collect::<Vec<_>>(), 400);
    }

    fn check_set_primitives(elems: &[u64]) {
        let sz = elems.len();
        println!("\n\nprimitives: {:?}\n", elems);
        let mut a = vec![0; sz];
        for x in elems.iter().cloned() {
            if p_lookfor(x, &a, 0).key_found() {
                println!("we already have {}", x);
            } else {
                let i = p_insert(x, &mut a, 0);
                a[i] = x;
            }
            println!("    after inserting {} we have {:?}", x, a);
            assert!(p_lookfor(x, &a, 0).key_found());
        }
        for x in elems.iter().cloned() {
            println!("looking for {}", x);
            assert!(p_lookfor(x, &a, 0).key_found());
        }
        for x in elems.iter().cloned() {
            println!("removing {}", x);
            p_remove(x, &mut a, 0);
            println!("    after removing {} we have {:?}", x, a);
        }
        for x in elems.iter().cloned() {
            println!("XXXX looking for {}", x);
            assert!(p_lookfor(x, &a, 0).empty_spot());
        }
        println!("after everything was removed: {:?}", a);
        for x in a.iter().cloned() {
            assert_eq!(x, 0);
        }
    }

    proptest! {
        #[test]
        fn check_random_primitives(slice in prop::collection::vec(1u64..50, 1usize..100)) {
            check_set_primitives(&slice);
        }
    }
}

/// The slot of a table of `n` slots where the key `k` belongs.
///
/// The key is scrambled by a multiplication before it is scaled to the size of the
/// table, so that keys that are close together, or that differ by a multiple of the
/// size of the table, do not pile up in the same place.  Without that, a table with
/// a few times as many possible keys as slots (a set at about 1% density, say) can
/// get long runs of keys that every lookup has to walk along.  Scaling is by another
/// multiplication rather than the division that `%` costs.  The scrambling depends
/// on `n`, which is random.
#[inline]
fn home(k: u64, n: usize) -> usize {
    let h = (k ^ (n as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    ((h as u128 * n as u128) >> 64) as usize
}

/// The most slots we will push entries along to make room for another.  A table that is
/// so full that making room takes more is grown instead.  Without a limit a table is only
/// grown when it is completely full (or, for a `SetU32`, nearly so, but finding that out
/// meant counting its empty slots on every insertion), and the last insertions before
/// that each have to push a very long run of entries along.
const MAX_CHAIN: usize = 256;

/// The most slots to push entries along in a table of `n` slots: a small table must not
/// be allowed to get nearly full either.
fn max_chain(n: usize) -> usize {
    (n / 16 + 16).min(MAX_CHAIN)
}

/// Whether there is an empty slot within `MAX_CHAIN` slots after where the key `k` belongs,
/// which is as far as we would have to push entries to make room for it.
fn p_has_room(k: u64, a: &[u64]) -> bool {
    let n = a.len();
    let start = home(k, n);
    let end = start + max_chain(n).min(n);
    if end <= n {
        a[start..end].iter().any(|&x| x == 0)
    } else {
        a[start..].iter().any(|&x| x == 0) || a[..end - n].iter().any(|&x| x == 0)
    }
}

/// How much bigger, in percent, a table that has run out of room becomes: at least `GROW_MIN`
/// and less than `GROW_MAX`.  The choice is random, which makes it harder to arrange for many
/// keys to collide.  Growing by a lot leaves a table with a lot of room to spare, which costs
/// memory, and growing by only a little means growing again soon, which costs time.
const GROW_MIN: usize = 20;
const GROW_MAX: usize = 50;

/// A table of up to this many slots grows by between 1 and 2 times its size instead, which
/// costs little memory and saves rebuilding it many times on the way up.
const GROW_SMALL: usize = 128;

/// The capacity to grow a table of capacity `cap` to.
fn grown_capacity(cap: usize, bits: u64) -> usize {
    if cap <= GROW_SMALL {
        return cap + 1 + crate::rand::rand_usize(cap, bits) % cap;
    }
    let span = cap * (GROW_MAX - GROW_MIN) / 100 + 1;
    cap + cap * GROW_MIN / 100 + 1 + crate::rand::rand_usize(cap, bits) % span
}

fn p_poverty(k: u64, idx: usize, n: usize) -> usize {
    // How far `idx` is beyond the slot where `k` would ideally be, going round the table.
    let h = home(k, n);
    if idx >= h {
        idx - h
    } else {
        idx + n - h
    }
}

/// This inserts k into the array, and requires that there be room for
/// one more element.  Otherwise, things will be sad.
fn p_insert(k: u64, a: &mut [u64], offset: u64) -> usize {
    let n = a.len();
    if n == 0 {
        unreachable!()
    }
    // `ii` is the slot `pov` beyond where `k` ought to be.
    let mut ii = home(k, n);
    for pov in 0..n {
        if a[ii] == 0 || a[ii] >> offset == k {
            // println!("already got a spot");
            return ii;
        }
        let ki = a[ii] >> offset;
        let pov_ki = p_poverty(ki, ii, n);
        if pov_ki < pov {
            // println!("need to steal from {} < {} at spot {}", pov_ki, pov, ii);
            // need to steal
            let stolen = ii;
            let mut displaced = a[ii];
            // println!("displaced value is {}", displaced);
            let mut pov_displaced = pov_ki;
            a[stolen] = 0;

            let mut jj = stolen;
            for _ in 1..n {
                pov_displaced += 1;
                jj += 1;
                if jj == n {
                    jj = 0;
                }
                if a[jj] == 0 {
                    // We finally found an unoccupied spot!
                    // println!("put the displaced at {}", jj);
                    a[jj] = displaced;
                    return stolen;
                }
                let kj = a[jj] >> offset;
                let pov_kj = p_poverty(kj, jj, n);
                if pov_kj < pov_displaced {
                    // need to steal again!
                    std::mem::swap(&mut a[jj], &mut displaced);
                    pov_displaced = pov_kj;
                }
            }
            panic!("p_insert was called when there was no room!")
        }
        ii += 1;
        if ii == n {
            ii = 0;
        }
    }
    unreachable!()
}

/// Checks that `a` is a valid Robin Hood table: each entry can be reached from the slot it
/// belongs in without crossing an empty slot, and sits no further from it than the entry
/// before it, plus one.
#[cfg(test)]
fn check_table(a: &[u64]) {
    let n = a.len();
    for (i, &x) in a.iter().enumerate() {
        if x == 0 {
            continue;
        }
        let pov = p_poverty(x, i, n);
        for d in 1..=pov {
            assert_ne!(a[(i + n - d) % n], 0, "a gap before {} in {:?}", x, a);
        }
        let prev = (i + n - 1) % n;
        if a[prev] != 0 && pov > 0 {
            assert!(pov <= p_poverty(a[prev], prev, n) + 1, "{:?}", a);
        }
    }
}

#[test]
fn test_insert() {
    // In an empty table the key goes where it belongs, and nothing moves.
    let mut a = [0 as u64; 4];
    assert_eq!(home(2, 4), p_insert(2, &mut a, 0));
    assert_eq!(&a, &[0, 0, 0, 0]);
    for i in 0..10 {
        assert_eq!(0, a[p_insert(i, &mut a, 0)]);
    }

    // A key that belongs where another one already is goes in the next slot.
    let other = (7 as u64..).find(|&k| home(k, 4) == home(6, 4)).unwrap();
    let mut a = [0 as u64; 4];
    a[home(6, 4)] = 6;
    assert_eq!((home(6, 4) + 1) % 4, p_insert(other, &mut a, 0));
    assert_eq!(a[home(6, 4)], 6);
    for i in 0..10 {
        assert!([0, i].contains(&a[p_insert(i, &mut a, 0)]));
    }

    // Tables of every small size can be filled, and each key can then be found.
    for n in 1..12usize {
        let mut a = vec![0 as u64; n];
        for k in 1..=n as u64 {
            let i = p_insert(k * 7, &mut a, 0);
            a[i] = k * 7;
            check_table(&a);
        }
        for k in 1..=n as u64 {
            assert!(p_lookfor(k * 7, &a, 0).key_found());
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum LookedUp {
    EmptySpot(usize),
    KeyFound(usize),
    NeedInsert,
}
impl LookedUp {
    fn key_found(self) -> bool {
        if let LookedUp::KeyFound(_) = self {
            true
        } else {
            false
        }
    }
    #[cfg(test)]
    fn empty_spot(self) -> bool {
        if let LookedUp::EmptySpot(_) = self {
            true
        } else {
            false
        }
    }
    #[cfg(test)]
    fn unwrap(self) -> usize {
        if let LookedUp::KeyFound(idx) = self {
            idx
        } else {
            panic!("unwrap called on {:?}", self)
        }
    }
}

fn p_lookfor(k: u64, a: &[u64], offset: u64) -> LookedUp {
    let n = a.len();
    if n == 0 {
        return LookedUp::NeedInsert;
    }
    // `ii` is the slot `pov` beyond where `k` ought to be.
    let mut ii = home(k, n);
    for pov in 0..n {
        // println!("looking in spot ii = {} with pov={}", ii, pov);
        if a[ii] == 0 {
            // println!("got empty spot at {} for key {}", ii, k);
            return LookedUp::EmptySpot(ii);
        }
        let ki = a[ii] >> offset;
        if ki == k {
            // println!("lookfor already got a spot");
            return LookedUp::KeyFound(ii);
        } else if p_poverty(ki, ii, n) < pov {
            // println!("at index {} we have {} > {}", ii, pov, pov_ki);
            return LookedUp::NeedInsert;
        }
        ii += 1;
        if ii == n {
            ii = 0;
        }
    }
    LookedUp::NeedInsert
}

#[test]
fn test_lookfor() {
    // A full table without the key says to make room.
    assert_eq!(LookedUp::NeedInsert, p_lookfor(5, &[3, 1, 2], 0));
    // A key that is not there is never reported as found.
    assert!(!p_lookfor(5, &[3, 0, 2], 0).key_found());
    let mut a = [0 as u64; 4];
    a[home(7, 4)] = 7;
    assert_eq!(LookedUp::KeyFound(home(7, 4)), p_lookfor(7, &a, 0));
}

/// Remove value `k` from hashmap `a`, where we bit shift by `offset`
///
/// Returns true if the value was found.
fn p_remove(k: u64, a: &mut [u64], offset: u64) -> bool {
    let n = a.len();
    if n == 0 {
        return false;
    }
    // `ii` is the slot `i` beyond the bucket where the value ought to be, so we
    // start with where it ought to be in the hashmap.
    let mut ii = home(k, n);
    for i in 0..n {
        // println!("    looking to remove at distance {} slot {}", i, ii);
        if a[ii] == 0 {
            // We use a `0` value to indicate an empty bucket.
            return false;
        }
        let ki = a[ii] >> offset;
        if ki == k {
            // println!("found {} at location {}", k, ii);
            a[ii] = 0;
            // Now we need to return anything that might have been
            // stolen from... to massacre my grammar.
            let mut previous = ii;
            let mut jj = ii;
            for _ in 1..n {
                jj += 1;
                if jj == n {
                    jj = 0;
                }
                // println!("looking at removing location {}", jj);
                if a[jj] == 0 || p_poverty(a[jj] >> offset, jj, n) == 0 {
                    // We found an unoccupied spot or a perfectly
                    // happy customer, so nothing else could have been
                    // bumped.
                    return true;
                }
                // need to undo some stealing!
                a[previous] = a[jj];
                a[jj] = 0;
                previous = jj;
            }
            return true;
        } else if i > p_poverty(ki, ii, n) {
            // The value we want would have displaced this one.
            return false;
        }
        ii += 1;
        if ii == n {
            ii = 0;
        }
    }
    // The array was entirely full and we didn't find the value, so it wasn't present.
    false
}

#[cfg(test)]
fn test_insert_remove(x: u64, a: &mut [u64]) {
    println!("test_insert_remove({}, {:?})", x, a);
    let v: Vec<u64> = a.iter().cloned().collect();
    assert!(!p_remove(x, a, 0));
    assert!(!a.contains(&x)); // otherwise the test won't work right.
    assert!(!p_lookfor(x, a, 0).key_found());
    a[p_insert(x, a, 0)] = x;
    assert!(a.contains(&x));
    println!("  after insertion of {} a is {:?}", x, a);
    assert!(p_lookfor(x, a, 0).key_found());
    assert_eq!(x, a[p_lookfor(x, a, 0).unwrap()]);
    assert!(p_remove(x, a, 0));
    println!("  after remove of {} a is {:?}", x, a);
    // The entries may have been rearranged, but they are the same ones, and the table is still valid.
    check_table(a);
    let mut now: Vec<_> = a.iter().cloned().collect();
    let mut before = v.clone();
    now.sort();
    before.sort();
    assert_eq!(now, before);
}

#[test]
fn test_remove() {
    for n in 3..9usize {
        for count in 0..n {
            let mut a = vec![0 as u64; n];
            for k in 1..=count as u64 {
                let i = p_insert(k * 11, &mut a, 0);
                a[i] = k * 11;
            }
            check_table(&a);
            // Inserting a key that is not there and removing it again changes nothing.
            test_insert_remove(1000, &mut a);
            // Removing any key leaves the others to be found.
            for k in 1..=count as u64 {
                let mut b = a.clone();
                assert!(p_remove(k * 11, &mut b, 0));
                check_table(&b);
                assert!(!p_lookfor(k * 11, &b, 0).key_found());
                for j in 1..=count as u64 {
                    if j != k {
                        assert!(p_lookfor(j * 11, &b, 0).key_found());
                    }
                }
            }
        }
    }
}

#[test]
fn p_remove_from_small_full() {
    let (x, y) = (258 as u64, 260 as u64);
    let mut a = [0 as u64; 2];
    let i = p_insert(x, &mut a, 0);
    a[i] = x;
    let i = p_insert(y, &mut a, 0);
    a[i] = y;
    assert!(!p_remove(2, &mut a, 0));
    assert_eq!(2, a.iter().filter(|&&v| v != 0).count());
    assert!(p_remove(x, &mut a, 0));
    assert!(p_lookfor(y, &a, 0).key_found());
    assert_eq!(1, a.iter().filter(|&&v| v != 0).count());
}

// Too slow to run under Miri.
#[test]
#[cfg_attr(miri, ignore)]
fn test_keys_do_not_cluster() {
    // About 1% of the numbers below two million: the table has a few times as many
    // possible keys as slots.  When a key's slot was its remainder modulo the number of
    // slots, keys wrapped round onto one another and entries were about 30 slots from
    // where they belong on average; scrambled, it is about 3.
    let mut x: u64 = 1;
    let mut s = SetU64::new();
    while s.len() < 20_000 {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        s.insert(((x >> 33) % 2_000_000) as u64);
    }
    match s.internal() {
        Internal::Heap { s: header, a } => {
            let n = a.len();
            let occupied = a.iter().filter(|&&v| v != 0).count();
            let total: usize = a
                .iter()
                .enumerate()
                .filter(|(_, &v)| v != 0)
                .map(|(i, &v)| p_poverty(v >> header.bits, i, n))
                .sum();
            let mean = total as f64 / occupied as f64;
            assert!(mean < 8.0, "mean distance from home is {}", mean);
        }
        _ => panic!("this should be a set stored as a hash table"),
    }
}

/// The mean distance of the entries of a set stored as a hash table from the slot they belong in.
#[cfg(test)]
fn mean_distance_from_home(s: &SetU64) -> Option<f64> {
    let entries: Vec<(u64, usize)>;
    let n;
    match s.internal() {
        Internal::Heap { s: header, a } => {
            n = a.len();
            entries = a
                .iter()
                .enumerate()
                .filter(|(_, &x)| x != 0)
                .map(|(i, &x)| (x >> header.bits, i))
                .collect();
        }
        Internal::Big { a, .. } => {
            n = a.len();
            entries = a
                .iter()
                .enumerate()
                .filter(|(_, &x)| x != 0)
                .map(|(i, &x)| (x, i))
                .collect();
        }
        _ => return None,
    }
    let total: usize = entries.iter().map(|&(k, i)| p_poverty(k, i, n)).sum();
    Some(total as f64 / entries.len().max(1) as f64)
}

// Too slow to run under Miri.
#[test]
#[cfg_attr(miri, ignore)]
fn test_tables_do_not_get_crowded() {
    // Insert one element at a time.  A table used to be grown only when it was completely
    // full, so much of the time it was nearly full, and the entries were pushed a long way
    // from the slots they belong in, which made each insertion slower than the one before.
    let mut x: u64 = 1;
    let mut s = SetU64::new();
    let mut worst = 0.0f64;
    for i in 0..30_000usize {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        s.insert(((x >> 33) % 50_000_000) as u64);
        if i % 97 == 0 {
            if let Some(mean) = mean_distance_from_home(&s) {
                worst = worst.max(mean);
            }
        }
    }
    assert!(worst < 12.0, "worst mean distance from home was {}", worst);
}
