use std;

use castset::Cast;

enum SearchResult {
    Present(usize),
    Empty(usize),
    /// The element is not present, but there is someone richer than
    /// us we could steal from!
    Richer(usize),
}

/// A set implemented for types that can be cast to usize
#[derive(Debug,Clone)]
pub struct OptCastSet<T: Cast> {
    inner: OCS<T>
}

#[derive(Debug,Clone)]
enum OCS<T: Cast> {
    Pow1 { sz: u8, v: Box<[T; 2]> },
    Pow2 { sz: u8, v: Box<[T; 4]> },
    Pow3 { sz: u8, v: Box<[T; 8]> },
    Pow4 { sz: u8, v: Box<[T; 16]> },
    Vec {
        v: Vec<T>,
        poweroftwo: u8,
        sz: usize,
    },
}

fn capacity_to_power(cap: usize) -> u8 {
    let bits = std::mem::size_of::<usize>() as u8 *8;
    let cap = cap*11/10; // give a bit of space
    bits - cap.leading_zeros() as u8
}

#[inline]
fn search_from<T: Cast>(v: &[T], i_start: usize, elem: T) -> SearchResult {
    let h = elem.cast();
    let mask = v.len()-1;
    let invalid = T::invalid();
    let mut dist = i_start.wrapping_sub(h.cast()) & mask;
    loop {
        let i = h+dist & mask;
        if v[i] == invalid {
            return SearchResult::Empty(i);
        } else if v[i] == elem {
            return SearchResult::Present(i);
        }
        // the following is a bit contorted, to compute distance
        // when wrapped.
        let his_dist = i.wrapping_sub(v[i].cast()) & mask;
        if his_dist < dist {
            return SearchResult::Richer(i);
        }
        dist += 1;
        assert!(dist < v.len());
    }
}

#[inline]
fn search<T: Cast>(v: &[T], elem: T) -> SearchResult {
    let h = elem.cast();
    let mask = v.len() - 1;
    let invalid = T::invalid();
    let mut dist = 0;
    loop {
        let i = h+dist & mask;
        if v[i] == invalid {
            return SearchResult::Empty(i);
        } else if v[i] == elem {
            return SearchResult::Present(i);
        }
        // the following is a bit contorted, to compute distance
        // when wrapped.
        let his_dist = i.wrapping_sub(v[i].cast()) & mask;
        if his_dist < dist {
            return SearchResult::Richer(i);
        }
        dist += 1;
        assert!(dist < v.len());
    }
}

fn remove<T: Cast>(v: &mut [T], elem: T) -> bool {
    match search(v, elem) {
        SearchResult::Present(mut i) => {
            let mask = v.len()-1;
            let invalid = T::invalid();
            loop {
                let iplus1 = (i+1) & mask;
                if v[iplus1] == invalid ||
                    (v[iplus1].cast().wrapping_sub(iplus1) & mask) == 0
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

#[inline]
fn insert<T: Cast>(v: &mut [T], mut elem: T) -> bool {
    match search(v, elem) {
        SearchResult::Present(_) => false,
        SearchResult::Empty(i) => {
            v[i] = elem;
            true
        },
        SearchResult::Richer(i) => {
            std::mem::swap(&mut elem, &mut v[i]);
            steal(v, i, elem);
            true
        },
    }
}

#[inline]
fn steal<T: Cast>(v: &mut [T], mut i: usize, mut elem: T) {
    loop {
        match search_from(v, i, elem) {
            SearchResult::Present(_) => return,
            SearchResult::Empty(i) => {
                v[i] = elem;
                return;
            },
            SearchResult::Richer(inew) => {
                std::mem::swap(&mut elem, &mut v[inew]);
                i = inew;
            },
        }
    }
}

impl<T: Cast> OptCastSet<T> {
    /// Creates an empty set..
    pub fn new() -> OptCastSet<T> {
        OptCastSet::with_capacity(0)
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_capacity(cap: usize) -> OptCastSet<T> {
        OptCastSet::with_pow(capacity_to_power(cap))
    }
    /// Creates an empty set with the specified power of two size.
    fn with_pow(pow: u8) -> OptCastSet<T> {
        let ocs = match pow {
            1 | 0 => OCS::Pow1 { v: Box::new([T::invalid(); 2]), sz: 0 },
            2 => OCS::Pow2 { v: Box::new([T::invalid(); 4]), sz: 0 },
            3 => OCS::Pow3 { v: Box::new([T::invalid(); 8]), sz: 0 },
            4 => OCS::Pow4 { v: Box::new([T::invalid(); 16]), sz: 0 },
            pow => {
                let cap: usize = 1 << pow;
                OCS::Vec {
                    poweroftwo: pow,
                    v: vec![T::invalid(); cap],
                    sz: 0,
                }
            },
        };
        OptCastSet { inner: ocs }
    }
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        match self.inner {
            OCS::Vec { v: _, sz, poweroftwo: _ } => sz,
            OCS::Pow1 { v: _, sz } => sz as usize,
            OCS::Pow2 { v: _, sz } => sz as usize,
            OCS::Pow3 { v: _, sz } => sz as usize,
            OCS::Pow4 { v: _, sz } => sz as usize,
        }
    }
    fn my_power(&self) -> u8 {
        match self.inner {
            OCS::Vec { v: _, sz: _, poweroftwo } => poweroftwo,
            OCS::Pow1 { v: _, sz: _ } => 1,
            OCS::Pow2 { v: _, sz: _ } => 2,
            OCS::Pow3 { v: _, sz: _ } => 3,
            OCS::Pow4 { v: _, sz: _ } => 3,
        }
    }
    /// Reserves capacity for at least `additional` more elements to be
    /// inserted in the set. The collection may reserve more space
    /// to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        let pow = capacity_to_power(self.len() + additional);
        if pow > self.my_power() {
            let mut set = OptCastSet::with_pow(pow);
            for &v in self.iter() {
                set.insert_unchecked(v);
            }
            *self = set;
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
    fn insert_unchecked(&mut self, elem: T) -> bool {
        match self.inner {
            OCS::Vec { ref mut v, ref mut sz, poweroftwo: _ } => {
                if insert(v, elem) {
                    *sz += 1;
                    true
                } else { false }
            },
            OCS::Pow1 { ref mut v, ref mut sz } => {
                if insert(v.as_mut(), elem) {
                    *sz += 1;
                    true
                } else { false }
            },
            OCS::Pow2 { ref mut v, ref mut sz } => {
                if insert(v.as_mut(), elem) {
                    *sz += 1;
                    true
                } else { false }
            },
            OCS::Pow3 { ref mut v, ref mut sz } => {
                if insert(v.as_mut(), elem) {
                    *sz += 1;
                    true
                } else { false }
            },
            OCS::Pow4 { ref mut v, ref mut sz } => {
                if insert(v.as_mut(), elem) {
                    *sz += 1;
                    true
                } else { false }
            },
        }
    }
    /// Returns true if the set contains a value.
    pub fn contains(&self, value: &T) -> bool {
        match search(self.slice(), *value) {
            SearchResult::Present(_) => true,
            SearchResult::Empty(_) => false,
            SearchResult::Richer(_) => false,
        }
    }
    /// Removes an element, and returns true if that element was present.
    pub fn remove(&mut self, value: &T) -> bool {
        if remove(self.mut_slice(), *value) {
            match self.inner {
                OCS::Vec { v: _, ref mut sz, poweroftwo: _ } => { *sz -= 1; },
                OCS::Pow1 { v: _, ref mut sz } => { *sz -= 1; },
                OCS::Pow2 { v: _, ref mut sz } => { *sz -= 1; },
                OCS::Pow3 { v: _, ref mut sz } => { *sz -= 1; },
                OCS::Pow4 { v: _, ref mut sz } => { *sz -= 1; },
            }
            true
        } else {
            false
        }
    }
    fn slice(&self) -> &[T] {
        match self.inner {
            OCS::Vec { ref v, sz: _, poweroftwo: _ } => v,
            OCS::Pow1 { ref v, sz: _ } => v.as_ref(),
            OCS::Pow2 { ref v, sz: _ } => v.as_ref(),
            OCS::Pow3 { ref v, sz: _ } => v.as_ref(),
            OCS::Pow4 { ref v, sz: _ } => v.as_ref(),
        }
    }
    fn mut_slice(&mut self) -> &mut [T] {
        match self.inner {
            OCS::Vec { ref mut v, sz: _, poweroftwo: _ } => v,
            OCS::Pow1 { ref mut v, sz: _ } => v.as_mut(),
            OCS::Pow2 { ref mut v, sz: _ } => v.as_mut(),
            OCS::Pow3 { ref mut v, sz: _ } => v.as_mut(),
            OCS::Pow4 { ref mut v, sz: _ } => v.as_mut(),
        }
    }
    /// Returns an iterator over the set.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            slice: self.slice(),
            nleft: self.len(),
        }
    }
    // /// Clears the set, returning all elements in an iterator.
    // pub fn drain(&mut self) -> IntoIter<T> {
    //     let set = std::mem::replace(self, OptCastSet::new());
    //     IntoIter { set: set }
    // }
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

// impl<'a, T: Cast> IntoIterator for &'a OptCastSet<T> {
//     type Item = &'a T;
//     type IntoIter = Iter<'a, T>;

//     fn into_iter(self) -> Iter<'a, T> {
//         self.iter()
//     }
// }

// pub struct IntoIter<T: Cast> {
//     set: OptCastSet<T>
// }

// impl<T: Cast> Iterator for IntoIter<T> {
//     type Item = T;
//     fn next(&mut self) -> Option<T> {
//         if self.set.sz == 0 {
//             None
//         } else {
//             self.set.sz -= 1;
//             loop {
//                 let val = self.set.v.pop();
//                 if val != Some(T::invalid()) {
//                     return val;
//                 }
//             }
//         }
//     }
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.set.sz, Some(self.set.sz))
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn it_works() {
        let mut ss: OptCastSet<usize> = OptCastSet::new();
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
        println!("small size: {}", std::mem::size_of::<OptCastSet<usize>>());
        println!(" hash size: {}", std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<OptCastSet<usize>>() <=
                2*std::mem::size_of::<HashSet<usize>>());
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches_u8(steps: Vec<Result<u8,u8>>) -> bool {
            let mut steps = steps;
            let mut set = OptCastSet::<u8>::new();
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
            let mut set = OptCastSet::<usize>::new();
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

