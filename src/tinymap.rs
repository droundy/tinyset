//! A set that is compact in size.

use std;

use std::hash::Hash;
use fnv::FnvHashMap;

/// Small set?
#[derive(Debug, Clone)]
pub enum TinyMap<K: Hash+Eq,T> {
    /// should be private
    Sm(Vec<(K,T)>),
    /// should be private
    V(FnvHashMap<K,T>)
}

const SMALL: usize = 16;

impl<K: Hash+Eq, T> TinyMap<K,T> {
    /// new
    pub fn new() -> TinyMap<K,T> {
        TinyMap::Sm(Vec::new())
    }
    /// len
    pub fn len(&self) -> usize {
        match self {
            &TinyMap::Sm(ref v) => v.len(),
            &TinyMap::V(ref m) => m.len(),
        }
    }
    /// insert
    pub fn insert(&mut self, k: K, elem: T) -> Option<T> {
        let oldv = match self {
            &mut TinyMap::Sm(ref mut v) => {
                if v.len() < SMALL {
                    let mut e = None;
                    for (i, &(ref kk, _)) in v.iter().enumerate() {
                        if *kk == k {
                            e = Some(i);
                            break;
                        }
                    }
                    return if let Some(i) = e {
                        Some(std::mem::replace(&mut v[i], (k,elem)).1)
                    } else {
                        if v.len() == v.capacity() {
                            v.reserve_exact(1);
                        }
                        v.push((k,elem));
                        None
                    };
                }
                std::mem::replace(v, Vec::new())
            },
            &mut TinyMap::V(ref mut m) => return m.insert(k,elem),
        };
        let mut m = FnvHashMap::default();
        let mut e = None;
        for x in oldv {
            if x.0 == k {
                e = Some(x.1);
            } else {
                m.insert(x.0, x.1);
            }
        }
        m.insert(k, elem);
        *self = TinyMap::V(m);
        e
    }
    /// has key
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<T>
        where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            &mut TinyMap::Sm(ref mut v) => {
                for i in 0..v.len() {
                    if v[i].0.borrow().eq(k) {
                        return Some(v.swap_remove(i).1)
                    }
                }
                None
            },
            &mut TinyMap::V(ref mut m) => m.remove(k),
        }
    }
    /// has key
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
        where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            &TinyMap::Sm(ref v) => v.iter().any(|x| x.0.borrow().eq(k)),
            &TinyMap::V(ref m) => m.contains_key(k),
        }
    }
    /// get
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&T>
        where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            &TinyMap::Sm(ref v) => {
                for i in 0..v.len() {
                    if v[i].0.borrow().eq(k) {
                        return Some(&v[i].1)
                    }
                }
                None
            },
            &TinyMap::V(ref m) => m.get(k),
        }
    }
    /// get
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut T>
        where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            &mut TinyMap::Sm(ref mut v) => {
                for i in 0..v.len() {
                    if v[i].0.borrow().eq(k) {
                        return Some(&mut v[i].1)
                    }
                }
                None
            },
            &mut TinyMap::V(ref mut m) => m.get_mut(k),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn it_works() {
        let mut ss: TinyMap<usize,u8> = TinyMap::new();
        println!("inserting 5");
        ss.insert(5, 5);
        println!("ss: {:?}", &ss);
        println!("contains 5");
        assert!(ss.contains_key(&5));
        assert_eq!(ss.get(&5), Some(&5));
        println!("contains 4");
        assert!(!ss.contains_key(&4));
        println!("inserting 3");
        ss.insert(3, 30);
        println!("now {:?}", &ss);
        assert!(ss.contains_key(&3));
        assert!(ss.contains_key(&5));
        assert_eq!(ss.len(), 2);
        // for num in ss.iter() {
        //     println!("num is {}", num);
        //     assert!(ss.contains(&num));
        // }
        // assert!(!ss.remove(&2));
        // assert!(ss.remove(&3));
        // assert!(!ss.contains(&3));
        // assert_eq!(ss.len(), 1);
    }
    #[test]
    fn size_unwasted() {
        println!("small size: {}", std::mem::size_of::<TinyMap<u64,u64>>());
        println!(" hash size: {}", std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<TinyMap<u64,u64>>() <=
                2*std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<TinyMap<u64,u64>>() <= 32);
    }

    // #[cfg(test)]
    // quickcheck! {
    //     fn prop_bigint(steps: Vec<Result<(u64,u8),(u64,u8)>>) -> bool {
    //         let mut steps = steps;
    //         let mut set = U64Set::new();
    //         let mut refset = HashSet::<u64>::new();
    //         loop {
    //             match steps.pop() {
    //                 Some(Ok((v,shift))) => {
    //                     let v = v << (shift & 31);
    //                     set.insert(v); refset.insert(v);
    //                 },
    //                 Some(Err((v,shift))) => {
    //                     let v = v << (shift & 31);
    //                     set.remove(&v); refset.remove(&v);
    //                 },
    //                 None => return true,
    //             }
    //             if set.len() != refset.len() { return false; }
    //             for i in 0..2550 {
    //                 if set.contains(&i) != refset.contains(&i) {
    //                     println!("refset: {:?}", &refset);
    //                     println!("set: {:?}", &set);
    //                     for x in set.iter() {
    //                         print!(" {}", x);
    //                     }
    //                     println!();
    //                     assert_eq!(set.contains(&i), refset.contains(&i));
    //                     return false;
    //                 }
    //             }
    //         }
    //     }
    // }
}
