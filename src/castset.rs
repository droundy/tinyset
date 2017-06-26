use std;

/// Trait for any type that can be converted to a `usize`.  This could
/// actually be a hash function, but we will assume that it is *fast*,
/// so I'm not calling it `Hash`.
pub trait Cast : Copy+Eq {
    /// Convert to a `usize`.
    fn cast(self) -> usize;
    /// A unique invalid value for this type.  If you cannot identify
    /// an invalid value, then you don't get to use CastSet, since we
    /// would need to store an `Option<T>` which would probably double
    /// the size of the set.
    fn invalid() -> Self;
}

impl Cast for usize {
    fn cast(self) -> usize { self }
    fn invalid() -> Self { (-1 as i64) as Self }
}
impl Cast for u64 {
    fn cast(self) -> usize { self as usize }
    fn invalid() -> Self { (-1 as i64) as Self }
}
impl Cast for u32 {
    fn cast(self) -> usize { self as usize }
    fn invalid() -> Self { (-1 as i32) as Self }
}
impl Cast for u16 {
    fn cast(self) -> usize { self as usize }
    fn invalid() -> Self { (-1 as i16) as Self }
}
impl Cast for u8 {
    fn cast(self) -> usize { self as usize }
    fn invalid() -> Self { (-1 as i8) as Self }
}

enum SearchResult {
    Present(usize),
    Empty(usize),
    /// The element is not present, but there is someone richer than
    /// us we could steal from!
    Richer(usize),
}

/// A set implemented for types that can be cast to usize
#[derive(Debug,Clone)]
pub struct CastSet<T: Cast> {
    v: Vec<T>,
    poweroftwo: u8,
    sz: usize,
}

fn capacity_to_power(cap: usize) -> u8 {
    let bits = std::mem::size_of::<usize>() as u8 *8;
    let cap = cap*11/10; // give a bit of space
    bits - cap.leading_zeros() as u8
}

impl<T: Cast> CastSet<T> {
    /// Creates an empty set..
    pub fn new() -> CastSet<T> {
        CastSet::with_capacity(0)
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_capacity(cap: usize) -> CastSet<T> {
        let pow = capacity_to_power(cap);
        let cap: usize = 1 << pow;
        CastSet {
            poweroftwo: pow,
            v: vec![T::invalid(); cap],
            sz: 0,
        }
    }
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.sz
    }
    /// Reserves capacity for at least `additional` more elements to be
    /// inserted in the set. The collection may reserve more space
    /// to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        let pow = capacity_to_power(self.sz + additional);
        if pow > self.poweroftwo {
            let cap: usize = 1 << pow;
            let oldv = std::mem::replace(&mut self.v, vec![T::invalid(); cap]);
            self.poweroftwo = pow;
            self.sz = 0;
            let invalid = T::invalid();
            for e in oldv {
                if e != invalid {
                    self.insert(e);
                }
            }
        }
    }
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn insert(&mut self, mut elem: T) -> bool {
        self.reserve(1);
        match self.search(elem) {
            SearchResult::Present(_) => false,
            SearchResult::Empty(i) => {
                self.v[i] = elem;
                self.sz += 1;
                true
            },
            SearchResult::Richer(i) => {
                std::mem::swap(&mut elem, &mut self.v[i]);
                self.steal(i, elem);
                self.sz += 1;
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
                self.sz -= 1;
                let mask = (1 << self.poweroftwo) - 1;
                let invalid = T::invalid();
                loop {
                    let iplus1 = (i+1) & mask;
                    if self.v[iplus1] == invalid ||
                        (self.v[iplus1].cast().wrapping_sub(iplus1) & mask) == 0
                    {
                        self.v[i] = invalid;
                        return true;
                    }
                    self.v[i] = self.v[iplus1];
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
                    self.v[i] = elem;
                    return;
                },
                SearchResult::Richer(inew) => {
                    std::mem::swap(&mut elem, &mut self.v[inew]);
                    i = inew;
                },
        }
        }
    }
    fn search(&self, elem: T) -> SearchResult {
        let h = elem.cast();
        let mask = (1 << self.poweroftwo) - 1;
        let invalid = T::invalid();
        let mut dist = 0;
        loop {
            let i = h+dist & mask;
            if self.v[i] == invalid {
                return SearchResult::Empty(i);
            } else if self.v[i] == elem {
                return SearchResult::Present(i);
            }
            // the following is a bit contorted, to compute distance
            // when wrapped.
            let his_dist = self.v[i].cast().wrapping_sub(i) & mask;
            if his_dist < dist {
                return SearchResult::Richer(i);
            }
            dist += 1;
            assert!(dist < self.v.capacity());
        }
    }
    fn search_from(&self, i_start: usize, elem: T) -> SearchResult {
        let h = elem.cast();
        let mask = (1 << self.poweroftwo) - 1;
        let invalid = T::invalid();
        let mut dist = h.cast().wrapping_sub(i_start) & mask;
        loop {
            let i = h+dist & mask;
            if self.v[i] == invalid {
                return SearchResult::Empty(i);
            } else if self.v[i] == elem {
                return SearchResult::Present(i);
            }
            // the following is a bit contorted, to compute distance
            // when wrapped.
            let his_dist = self.v[i].cast().wrapping_sub(i) & mask;
            if his_dist < dist {
                return SearchResult::Richer(i);
            }
            dist += 1;
            assert!(dist < self.v.capacity());
        }
    }
    /// Returns an iterator over the set.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            slice: &self.v,
            nleft: self.sz,
        }
    }
    /// Clears the set, returning all elements in an iterator.
    pub fn drain(&mut self) -> IntoIter<T> {
        let set = std::mem::replace(self, CastSet::new());
        IntoIter { set: set }
    }
}

pub struct Iter<'a, T: 'a+Cast> {
    slice: &'a [T],
    nleft: usize,
}

impl<'a, T: 'a+Cast> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.nleft == 0 {
            None
        } else {
            assert!(self.slice.len() >= self.nleft);
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

impl<'a, T: Cast> IntoIterator for &'a CastSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

pub struct IntoIter<T: Cast> {
    set: CastSet<T>
}

impl<T: Cast> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.set.sz == 0 {
            None
        } else {
            self.set.sz -= 1;
            loop {
                let val = self.set.v.pop();
                if val != Some(T::invalid()) {
                    return val;
                }
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.set.sz, Some(self.set.sz))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn it_works() {
        let mut ss: CastSet<usize> = CastSet::new();
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
        println!("small size: {}", std::mem::size_of::<CastSet<usize>>());
        println!(" hash size: {}", std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<CastSet<usize>>() <=
                2*std::mem::size_of::<HashSet<usize>>());
    }
}

