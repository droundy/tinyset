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
pub struct OptCastSet<T: Cast> {
    sz: usize,
    v: OCS<T>,
}

enum OCS<T: Cast> {
    Pow1  (Box<[T; 2]>),
    Pow2  (Box<[T; 4]>),
    Pow3  (Box<[T; 8]>),
    Pow4  (Box<[T; 16]>),
    Pow5  (Box<[T; 32]>),
    Pow6  (Box<[T; 64]>),
    Pow7  (Box<[T; 128]>),
    Pow8  (Box<[T; 256]>),
    Pow9  (Box<[T; 512]>),
    Pow10 (Box<[T; 1024]>),
    Pow11 (Box<[T; 2048]>),
    Pow12 (Box<[T; 4096]>),
    Pow13 (Box<[T; 1<<13]>),
    Pow14 (Box<[T; 1<<14]>),
    Pow15 (Box<[T; 1<<15]>),
    Pow16 (Box<[T; 1<<16]>),
    Pow17 (Box<[T; 1<<17]>),
    Pow18 (Box<[T; 1<<18]>),
    Pow19 (Box<[T; 1<<19]>),
    Pow20 (Box<[T; 1<<20]>),
    Pow21 (Box<[T; 1<<21]>),
    Pow22 (Box<[T; 1<<22]>),
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
            1 | 0 => OCS::Pow1 (Box::new([T::invalid(); 2])),
            2 =>  OCS::Pow2 (Box::new([T::invalid(); 4])),
            3 =>  OCS::Pow3 (Box::new([T::invalid(); 8])),
            4 =>  OCS::Pow4 (Box::new([T::invalid(); 16])),
            5 =>  OCS::Pow5  (Box::new([T::invalid(); 32])),
            6 =>  OCS::Pow6  (Box::new([T::invalid(); 64])),
            7 =>  OCS::Pow7  (Box::new([T::invalid(); 128])),
            8 =>  OCS::Pow8  (Box::new([T::invalid(); 256])),
            9 =>  OCS::Pow9  (Box::new([T::invalid(); 512])),
            10 => OCS::Pow10 (Box::new([T::invalid(); 1024])),
            11 => OCS::Pow11 (Box::new([T::invalid(); 2048])),
            12 => OCS::Pow12 (Box::new([T::invalid(); 4096])),
            13 => OCS::Pow13 (Box::new([T::invalid(); 1<<13])),
            14 => OCS::Pow14 (Box::new([T::invalid(); 1<<14])),
            15 => OCS::Pow15 (Box::new([T::invalid(); 1<<15])),
            16 => OCS::Pow16 (Box::new([T::invalid(); 1<<16])),
            17 => OCS::Pow17 (Box::new([T::invalid(); 1<<17])),
            18 => OCS::Pow18 (Box::new([T::invalid(); 1<<18])),
            19 => OCS::Pow19 (Box::new([T::invalid(); 1<<19])),
            20 => OCS::Pow20 (Box::new([T::invalid(); 1<<20])),
            21 => OCS::Pow21 (Box::new([T::invalid(); 1<<21])),
            22 => OCS::Pow22 (Box::new([T::invalid(); 1<<22])),
            _ => unimplemented!(),
        };
        OptCastSet { sz: 0, v: ocs }
    }
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.sz
    }
    fn my_power(&self) -> u8 {
        match self.v {
            OCS::Pow1  (_) => 1,
            OCS::Pow2  (_) => 2,
            OCS::Pow3  (_) => 3,
            OCS::Pow4  (_) => 4,
            OCS::Pow5  (_) => 5,
            OCS::Pow6  (_) => 6,
            OCS::Pow7  (_) => 7,
            OCS::Pow8  (_) => 8,
            OCS::Pow9  (_) => 9,
            OCS::Pow10 (_) => 10,
            OCS::Pow11 (_) => 11,
            OCS::Pow12 (_) => 12,
            OCS::Pow13 (_) => 13,
            OCS::Pow14 (_) => 14,
            OCS::Pow15 (_) => 15,
            OCS::Pow16 (_) => 16,
            OCS::Pow17 (_) => 17,
            OCS::Pow18 (_) => 18,
            OCS::Pow19 (_) => 19,
            OCS::Pow20 (_) => 20,
            OCS::Pow21 (_) => 21,
            OCS::Pow22 (_) => 22,
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
        let res = match self.v {
            OCS::Pow1  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow2  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow3  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow4  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow5  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow6  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow7  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow8  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow9  (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow10 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow11 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow12 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow13 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow14 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow15 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow16 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow17 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow18 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow19 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow20 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow21 (ref mut v) => insert(v.as_mut(), elem),
            OCS::Pow22 (ref mut v) => insert(v.as_mut(), elem),
        };
        if res { self.sz += 1; }
        res
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
            self.sz -= 1;
            true
        } else {
            false
        }
    }
    fn slice(&self) -> &[T] {
        match self.v {
            OCS::Pow1  (ref v) => v.as_ref(),
            OCS::Pow2  (ref v) => v.as_ref(),
            OCS::Pow3  (ref v) => v.as_ref(),
            OCS::Pow4  (ref v) => v.as_ref(),
            OCS::Pow5  (ref v) => v.as_ref(),
            OCS::Pow6  (ref v) => v.as_ref(),
            OCS::Pow7  (ref v) => v.as_ref(),
            OCS::Pow8  (ref v) => v.as_ref(),
            OCS::Pow9  (ref v) => v.as_ref(),
            OCS::Pow10 (ref v) => v.as_ref(),
            OCS::Pow11 (ref v) => v.as_ref(),
            OCS::Pow12 (ref v) => v.as_ref(),
            OCS::Pow13 (ref v) => v.as_ref(),
            OCS::Pow14 (ref v) => v.as_ref(),
            OCS::Pow15 (ref v) => v.as_ref(),
            OCS::Pow16 (ref v) => v.as_ref(),
            OCS::Pow17 (ref v) => v.as_ref(),
            OCS::Pow18 (ref v) => v.as_ref(),
            OCS::Pow19 (ref v) => v.as_ref(),
            OCS::Pow20 (ref v) => v.as_ref(),
            OCS::Pow21 (ref v) => v.as_ref(),
            OCS::Pow22 (ref v) => v.as_ref(),
        }
    }
    fn mut_slice(&mut self) -> &mut [T] {
        match self.v {
            OCS::Pow1  (ref mut v) => v.as_mut(),
            OCS::Pow2  (ref mut v) => v.as_mut(),
            OCS::Pow3  (ref mut v) => v.as_mut(),
            OCS::Pow4  (ref mut v) => v.as_mut(),
            OCS::Pow5  (ref mut v) => v.as_mut(),
            OCS::Pow6  (ref mut v) => v.as_mut(),
            OCS::Pow7  (ref mut v) => v.as_mut(),
            OCS::Pow8  (ref mut v) => v.as_mut(),
            OCS::Pow9  (ref mut v) => v.as_mut(),
            OCS::Pow10 (ref mut v) => v.as_mut(),
            OCS::Pow11 (ref mut v) => v.as_mut(),
            OCS::Pow12 (ref mut v) => v.as_mut(),
            OCS::Pow13 (ref mut v) => v.as_mut(),
            OCS::Pow14 (ref mut v) => v.as_mut(),
            OCS::Pow15 (ref mut v) => v.as_mut(),
            OCS::Pow16 (ref mut v) => v.as_mut(),
            OCS::Pow17 (ref mut v) => v.as_mut(),
            OCS::Pow18 (ref mut v) => v.as_mut(),
            OCS::Pow19 (ref mut v) => v.as_mut(),
            OCS::Pow20 (ref mut v) => v.as_mut(),
            OCS::Pow21 (ref mut v) => v.as_mut(),
            OCS::Pow22 (ref mut v) => v.as_mut(),
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
        // println!("now {:?}", &ss);
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
        assert!(std::mem::size_of::<OptCastSet<usize>>() <= 24);
        assert_eq!(std::mem::size_of::<OptCastSet<usize>>(),
                   std::mem::size_of::<OptCastSet<u8>>());
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

