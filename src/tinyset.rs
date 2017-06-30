//! A set that is compact in size.

use std;

use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

/// Trait for any type that can be converted to a `usize`.  This could
/// actually be a hash function, but we will assume that it is *fast*,
/// so I'm not calling it `Hash`.
pub trait HasInvalid : Copy+Eq+Hash {
    /// Hash to a `usize`.
    fn hash_usize(self) -> usize {
        let mut h: FnvHasher = Default::default();
        self.hash(&mut h);
        h.finish() as usize
    }
    /// A unique invalid value for this type.  If you cannot identify
    /// an invalid value, then you don't get to use TinySet, since we
    /// would need to store an `Option<T>` which would probably double
    /// the size of the set.
    fn invalid() -> Self;
}

impl HasInvalid for usize {
    fn invalid() -> Self { (-1 as i64) as Self }
}
impl HasInvalid for u64 {
    fn invalid() -> Self { (-1 as i64) as Self }
}
impl HasInvalid for u32 {
    fn invalid() -> Self { (-1 as i32) as Self }
}
impl HasInvalid for u16 {
    fn invalid() -> Self { (-1 as i16) as Self }
}
impl HasInvalid for u8 {
    fn invalid() -> Self { (-1 as i8) as Self }
}

enum SearchResult {
    Present(usize),
    Empty(usize),
    /// The element is not present, but there is someone richer than
    /// us we could steal from!
    Richer(usize),
}

/// A set implemented for types that have an invalid value
#[derive(Debug,Clone)]
pub struct TinySet<T: HasInvalid> {
    v: Data<T>,
}

#[derive(Debug, Clone)]
enum Data<T: HasInvalid> {
    Sm(u32, [usize; 2]),
    V(u32, Box<[T]>)
}
impl<T: HasInvalid> Data<T> {
    fn cutoff() -> usize {
        std::mem::size_of::<[usize;2]>()/std::mem::size_of::<T>()
    }
    fn new() -> Data<T> {
        let num = Data::<T>::cutoff();
        let mut v = Data::Sm(0,[0;2]);
        for i in 0..num {
            v.mu()[i] = T::invalid();
        }
        v
    }
    fn len(&self) -> usize {
        match self {
            &Data::Sm(_,_) => {
                Data::<T>::cutoff()
            },
            &Data::V(_,ref v) => v.len(),
        }
    }
    fn sl(&self) -> &[T] {
        match self {
            &Data::Sm(_,ref v) => {
                let num = Data::<T>::cutoff();
                match num {
                    1 => unsafe { std::mem::transmute::<&[usize;2],&[T;1]>(v) },
                    2 => unsafe { std::mem::transmute::<&[usize;2],&[T;2]>(v) },
                    4 => unsafe { std::mem::transmute::<&[usize;2],&[T;4]>(v) },
                    8 => unsafe { std::mem::transmute::<&[usize;2],&[T;8]>(v) },
                    16 => unsafe { std::mem::transmute::<&[usize;2],&[T;16]>(v) },
                    _ => unreachable!(),
                }
            },
            &Data::V(_,ref v) => v,
        }
    }
    fn mu(&mut self) -> &mut [T] {
        match self {
            &mut Data::Sm(_,ref mut v) => {
                let num = Data::<T>::cutoff();
                match num {
                    1 => unsafe { std::mem::transmute::<&mut [usize;2],&mut [T;1]>(v) },
                    2 => unsafe { std::mem::transmute::<&mut [usize;2],&mut [T;2]>(v) },
                    4 => unsafe { std::mem::transmute::<&mut [usize;2],&mut [T;4]>(v) },
                    8 => unsafe { std::mem::transmute::<&mut [usize;2],&mut [T;8]>(v) },
                    16 => unsafe { std::mem::transmute::<&mut [usize;2],&mut [T;16]>(v) },
                    _ => unreachable!(),
                }
            },
            &mut Data::V(_,ref mut v) => v,
        }
    }
}

fn capacity_to_rawcapacity(cap: usize) -> usize {
    if cap <= 4 {
        cap.next_power_of_two()
    } else {
        (cap*22/10).next_power_of_two()
    }
}

impl<T: HasInvalid> TinySet<T> {
    fn mut_sz(&mut self) -> &mut u32 {
        match &mut self.v {
            &mut Data::Sm(ref mut sz,_) => sz,
            &mut Data::V(ref mut sz,_) => sz,
        }
    }
    /// Creates an empty set..
    pub fn default() -> TinySet<T> {
        TinySet::with_capacity(0)
    }
    /// Creates an empty set..
    pub fn new() -> TinySet<T> {
        TinySet::with_capacity(0)
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_capacity(cap: usize) -> TinySet<T> {
        if cap <= Data::<T>::cutoff() {
            TinySet { v: Data::new() }
        } else {
            let cap = capacity_to_rawcapacity(cap);
            TinySet {
                v: Data::V(0, vec![T::invalid(); cap].into_boxed_slice()),
            }
        }
    }
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        match &self.v {
            &Data::Sm(sz,_) => sz as usize,
            &Data::V(sz,_) => sz as usize,
        }
    }
    /// Reserves capacity for at least `additional` more elements to be
    /// inserted in the set. The collection may reserve more space
    /// to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        let cap = capacity_to_rawcapacity(self.len() + additional);
        if cap > self.v.sl().len() {
            let oldv = std::mem::replace(&mut self.v,
                                         Data::V(0,vec![T::invalid(); cap]
                                                     .into_boxed_slice()));
            let invalid = T::invalid();
            for &e in oldv.sl().iter() {
                if e != invalid {
                    self.insert_unchecked(e);
                }
            }
        }
    }
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn insert(&mut self, elem: T) -> bool {
        self.reserve(1);
        self.insert_unchecked(elem)
    }
    fn insert_unchecked(&mut self, mut elem: T) -> bool {
        match self.search(elem) {
            SearchResult::Present(_) => false,
            SearchResult::Empty(i) => {
                self.v.mu()[i] = elem;
                *self.mut_sz() += 1;
                true
            },
            SearchResult::Richer(i) => {
                std::mem::swap(&mut elem, &mut self.v.mu()[i]);
                self.steal(i, elem);
                *self.mut_sz() += 1;
                true
            },
        }
    }
    /// Returns true if the set contains a value.
    pub fn contains(&self, value: &T) -> bool {
        match self.search(*value) {
            SearchResult::Present(_) => true,
            SearchResult::Empty(_) => false,
            SearchResult::Richer(_) => false,
        }
    }
    /// Removes an element, and returns true if that element was present.
    pub fn remove(&mut self, value: &T) -> bool {
        match self.search(*value) {
            SearchResult::Present(mut i) => {
                *self.mut_sz() -= 1;
                let mut v = self.v.mu();
                let mask = v.len() - 1;
                let invalid = T::invalid();
                loop {
                    let iplus1 = (i+1) & mask;
                    if v[iplus1] == invalid ||
                        (v[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                    {
                        v[i] = invalid;
                        return true;
                    }
                    v[i] = v[iplus1];
                    i = iplus1;
                }
            },
            SearchResult::Empty(_) => false,
            SearchResult::Richer(_) => false,
        }
    }
    fn steal(&mut self, mut i: usize, mut elem: T) {
        loop {
            match self.search_from(i, elem) {
                SearchResult::Present(_) => return,
                SearchResult::Empty(i) => {
                    self.v.mu()[i] = elem;
                    return;
                },
                SearchResult::Richer(inew) => {
                    std::mem::swap(&mut elem, &mut self.v.mu()[inew]);
                    i = inew;
                },
        }
        }
    }
    fn search(&self, elem: T) -> SearchResult {
        let h = elem.hash_usize();
        let invalid = T::invalid();
        let mut dist = 0;
        let v = self.v.sl();
        let mask = v.len() - 1;
        loop {
            let i = h+dist & mask;
            if v[i] == invalid {
                return SearchResult::Empty(i);
            } else if v[i] == elem {
                return SearchResult::Present(i);
            }
            // the following is a bit contorted, to compute distance
            // when wrapped.
            let his_dist = i.wrapping_sub(v[i].hash_usize()) & mask;
            if his_dist < dist {
                return SearchResult::Richer(i);
            }
            dist += 1;
            assert!(dist <= v.len());
        }
    }
    fn search_from(&self, i_start: usize, elem: T) -> SearchResult {
        let h = elem.hash_usize();
        let mask = self.v.len() - 1;
        let invalid = T::invalid();
        let mut dist = i_start.wrapping_sub(h) & mask;
        let v = self.v.sl();
        loop {
            let i = h+dist & mask;
            if v[i] == invalid {
                return SearchResult::Empty(i);
            } else if v[i] == elem {
                return SearchResult::Present(i);
            }
            // the following is a bit contorted, to compute distance
            // when wrapped.
            let his_dist = i.wrapping_sub(v[i].hash_usize()) & mask;
            if his_dist < dist {
                return SearchResult::Richer(i);
            }
            dist += 1;
            assert!(dist <= v.len());
        }
    }
    /// Returns an iterator over the set.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            slice: self.v.sl(),
            nleft: self.len(),
        }
    }
    /// Clears the set, returning all elements in an iterator.
    pub fn drain(&mut self) -> IntoIter<T> {
        let set = std::mem::replace(self, TinySet::new());
        let sz = set.len();
        IntoIter { set: set, nleft: sz }
    }
}

/// An iterator for `TinySet`.
pub struct Iter<'a, T: 'a+HasInvalid> {
    slice: &'a [T],
    nleft: usize,
}

impl<'a, T: 'a+HasInvalid> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.nleft == 0 {
            None
        } else {
            assert!(self.slice.len() >= self.nleft as usize);
            while self.slice[0] == T::invalid() {
                self.slice = self.slice.split_first().unwrap().1;
            }
            let val = &self.slice[0];
            self.slice = self.slice.split_first().unwrap().1;
            self.nleft -= 1;
            Some(val)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.nleft, Some(self.nleft))
    }
}

impl<'a, T: HasInvalid> IntoIterator for &'a TinySet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

/// An iterator for `TinySet`.
pub struct IntoIter<T: HasInvalid> {
    set: TinySet<T>,
    nleft: usize,
}

impl<T: HasInvalid> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.nleft == 0 {
            None
        } else {
            self.nleft -= 1;
            let mut i = self.nleft;
            loop {
                let val = std::mem::replace(&mut self.set.v.mu()[i], T::invalid());
                if val != T::invalid() {
                    return Some(val);
                }
                i -= 1;
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.nleft, Some(self.nleft))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use rand::{XorShiftRng, SeedableRng, Rand};
    #[test]
    fn it_works() {
        let mut ss: TinySet<usize> = TinySet::new();
        println!("inserting 5");
        ss.insert(5);
        println!("contains 5");
        assert!(ss.contains(&5));
        println!("contains 4");
        assert!(!ss.contains(&4));
        println!("inserting 3");
        ss.insert(3);
        println!("now {:?}", &ss);
        assert!(ss.contains(&3));
        assert!(ss.contains(&5));
        assert_eq!(ss.len(), 2);
        for num in ss.iter() {
            println!("num is {}", num);
            assert!(ss.contains(num));
        }
        assert!(!ss.remove(&2));
        assert!(ss.remove(&3));
        assert!(!ss.contains(&3));
        assert_eq!(ss.len(), 1);
    }
    #[test]
    fn size_unwasted() {
        println!("small size: {}", std::mem::size_of::<TinySet<usize>>());
        println!(" hash size: {}", std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<TinySet<usize>>() <=
                2*std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<TinySet<usize>>() <= 24);
    }

    macro_rules! initialize {
        ($set: ident, $item: ident, $num: expr) => {{
            let mut rng = XorShiftRng::from_seed([$num as u32,$num as u32,3,4]);
            let mut set = $set::<$item>::new();
            let mut refset = HashSet::<$item>::new();
            if $num > 0 {
                while set.len() < $num {
                    let ins = $item::rand(&mut rng) % (2*$num as $item);
                    let rem = $item::rand(&mut rng) % (2*$num as $item);
                    set.insert(ins);
                    if !set.contains(&ins) {
                        println!("oops insert");
                    }
                    set.remove(&rem);
                    if set.contains(&rem) {
                        println!("oops remove");
                    }
                    refset.insert(ins);
                    refset.remove(&rem);
                    println!("inserting {}, removing {} => {}", ins, rem, set.len());
                    println!("set: {:?}", set);
                    println!("refset: {:?}", refset);
                    let mut fails = false;
                    for i in 0..255 {
                        fails = fails || set.contains(&i) != refset.contains(&i);
                    }
                    if fails {
                        for i in 0..255 {
                            println!("i {}", i);
                            assert_eq!(set.contains(&i), refset.contains(&i));
                        }
                    }
                }
            }
            set
        }};
    }

    #[test]
    fn random_inserts_and_removals_u8() {
        for sz in 0..50 {
            println!("\nTinySet {}\n", sz);
            let myset = initialize!(TinySet, u8, sz);
            println!("\nHashSet {}\n", sz);
            let refset = initialize!(HashSet, u8, sz);
            for i in 0..255 {
                assert_eq!(myset.contains(&i), refset.contains(&i));
            }
        }
    }

    #[test]
    fn random_inserts_and_removals_u16() {
        for sz in 0..20 {
            println!("\nTinySet {}\n", sz);
            let myset = initialize!(TinySet, u16, sz);
            println!("\nHashSet {}\n", sz);
            let refset = initialize!(HashSet, u16, sz);
            for i in 0..50 {
                assert_eq!(myset.contains(&i), refset.contains(&i));
            }
        }
    }

    #[test]
    fn test_matches_u8() {
        let mut steps: Vec<Result<u8,u8>> = vec![Err(8), Ok(0), Ok(16), Ok(1), Ok(8)];
        let mut set = TinySet::<u8>::new();
        let mut refset = HashSet::<u8>::new();
        loop {
            match steps.pop() {
                Some(Ok(v)) => {
                    println!("\ninserting {}", v);
                    set.insert(v); refset.insert(v);
                },
                Some(Err(v)) => {
                    println!("\nremoving {}", v);
                    set.remove(&v); refset.remove(&v);
                },
                None => return,
            }
            println!("set: {:?}", set);
            println!("refset: {:?}", refset);
            assert_eq!(set.len(), refset.len());
            for i in 0..255 {
                if set.contains(&i) != refset.contains(&i) {
                    println!("trouble at {}", i);
                    assert_eq!(set.contains(&i), refset.contains(&i));
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches_u8(steps: Vec<Result<u8,u8>>) -> bool {
            let mut steps = steps;
            let mut set = TinySet::<u8>::new();
            let mut refset = HashSet::<u8>::new();
            loop {
                match steps.pop() {
                    Some(Ok(v)) => {
                        set.insert(v); refset.insert(v);
                    },
                    Some(Err(v)) => {
                        set.remove(&v); refset.remove(&v);
                    },
                    None => return true,
                }
                if set.len() != refset.len() { return false; }
                for i in 0..255 {
                    if set.contains(&i) != refset.contains(&i) { return false; }
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches_usize(steps: Vec<Result<usize,usize>>) -> bool {
            let mut steps = steps;
            let mut set = TinySet::<usize>::new();
            let mut refset = HashSet::<usize>::new();
            loop {
                match steps.pop() {
                    Some(Ok(v)) => {
                        set.insert(v); refset.insert(v);
                    },
                    Some(Err(v)) => {
                        set.remove(&v); refset.remove(&v);
                    },
                    None => return true,
                }
                if set.len() != refset.len() { return false; }
                for i in 0..2550 {
                    if set.contains(&i) != refset.contains(&i) { return false; }
                }
            }
        }
    }
}

