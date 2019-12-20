// Copyright 2019 David Roundy
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A map from [Fits64] types to any other type.

/// A map from a [Fits64] key to another type.
#[derive(Clone)]
pub struct Map64<K,V> {
    map: Map64U,
    elems: Vec<V>,
    phantom: std::marker::PhantomData<K>,
}

impl<K: crate::Fits64, V> Default for Map64<K,V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: crate::Fits64, V> Map64<K,V> {
    /// Create an empty map
    pub fn new() -> Self {
        Map64 {
            map: Map64U::new(),
            elems: Vec::new(),
            phantom: std::marker::PhantomData
        }
    }
    /// How many elements
    pub fn len(&self) -> usize {
        self.elems.len()
    }
    /// Insert a value.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if let Some(i) = self.map.insert(k.to_u64(), self.elems.len()) {
            Some(std::mem::replace(&mut self.elems[i], v))
        } else {
            self.elems.push(v);
            None
        }
    }
    /// Lookup a value
    pub fn get(&self, k: K) -> Option<&V> {
        self.map.get(k.to_u64()).map(|i| &self.elems[i])
    }
    /// Does key exist
    pub fn contains_key(&self, k: K) -> bool {
        self.map.get(k.to_u64()).is_some()
    }
    /// remove element
    pub fn remove(&mut self, k: K) -> Option<V> {
        self.map.remove(k.to_u64(), self.elems.len()-1)
            .map(|oldi| self.elems.swap_remove(oldi))
    }
}

#[test]
fn map64() {
    let mut x: Map64<u8, String> = Map64::new();
    assert!(x.get(b'X').is_none());
    assert!(x.insert(b'X', "X is awesome".to_string()).is_none());
}

#[derive(Clone)]
struct Lay {
    elem_bits: u8,
    key_bits: u8,
}

impl Lay {
    fn keymask(&self) -> u64 {
        (1 << self.key_bits) - 1
    }
    fn getvalue(&self, x: u64) -> usize {
        (x >> self.key_bits) as usize - 1
    }
    fn putvalue(&self, v: usize) -> u64 {
        (v as u64 + 1) << self.key_bits
    }
    fn update(&self, k: u64, v: usize, len: usize) -> Option<Lay> {
        if log_2(v as u64+1) > self.elem_bits || log_2(k) > self.key_bits || len == v {
            Some(Lay {
                elem_bits: std::cmp::max(log_2(v as u64+1), self.elem_bits),
                key_bits: std::cmp::max(log_2(k), self.key_bits),
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct Map64U {
    lay: Lay,
    data: Box<[u64]>,
}

impl Map64U {
    fn new() -> Self {
        Map64U {
            lay: Lay {
                elem_bits: 0,
                key_bits: 0,
            },
            data: Box::new([]),
        }
    }
    fn get(&self, k: u64) -> Option<usize> {
        if self.lay.key_bits + self.lay.elem_bits <= 64 {
            for x in self.data.iter().cloned() {
                if x != 0 && x & self.lay.keymask() == k {
                    return Some( self.lay.getvalue(x) );
                }
            }
            None
        } else {
            unimplemented!()
        }
    }
    // It is contractual that we must *never* attempt to insert a
    // value that is different from the total number of elements.  If
    // the key is already present, the value of the element is *not*
    // changed, but instead the previous value is returned.
    fn insert(&mut self, k: u64, v: usize) -> Option<usize> {
        if self.data.len() == v {
            // need to allocate more space
            let mut newdata = vec![0; (v+1)*2].into_boxed_slice();
            for (i,x) in self.data.iter().cloned().enumerate() {
                newdata[i] = x;
            }
            self.data = newdata;
        }
        if let Some(newl) = self.lay.update(k,v, self.data.len()) {
            let mut newmap = Map64U {
                lay: newl,
                data: if self.data.len() == v {
                    vec![0; (v+1)*2].into_boxed_slice()
                } else {
                    self.data.clone()
                },
            };
            let mut vec = self.data.iter().cloned()
                .filter(|x| *x != 0)
                .map(|x| (self.lay.getvalue(x), x & self.lay.keymask()))
                .collect::<Vec<_>>();
            vec.sort();
            for (v,k) in vec.into_iter() {
                newmap.insert(k,v);
            }
            *self = newmap;
        }
        if self.lay.key_bits + self.lay.elem_bits <= 63 {
            for x in self.data.iter().cloned() {
                if x != 0 && x & self.lay.keymask() == k {
                    return Some( self.lay.getvalue(x) );
                }
            }
            for x in self.data.iter_mut() {
                if *x == 0 {
                    *x = k | self.lay.putvalue(v);
                    return None;
                }
            }
            unreachable!()
        } else {
            unimplemented!()
        }
    }
    // remove the element with key u64, and put the value of k in
    // whatever element has value sz, then return the old value of
    // k.
    fn remove(&mut self, k: u64, sz: usize) -> Option<usize> {
        if self.lay.key_bits + self.lay.elem_bits <= 63 {
            let mut oldval = None;
            for x in self.data.iter_mut() {
                if *x != 0 && *x & self.lay.keymask() == k {
                    oldval = Some(self.lay.getvalue(*x));
                    *x = 0;
                    break;
                }
            }
            if let Some(oldval) = oldval {
                for x in self.data.iter_mut() {
                    if *x != 0 && self.lay.getvalue(*x) == sz {
                        *x = (*x & self.lay.keymask()) | self.lay.putvalue(oldval);
                        return None;
                    }
                }
            }
            oldval
        } else {
            unimplemented!()
        }
    }
}

fn log_2(x: u64) -> u8 {
    if x == 0 {
        1
    } else {
        64 - x.leading_zeros() as u8
    }
}

impl<K: Copy + Eq + Ord + std::fmt::Display + std::fmt::Debug + crate::Fits64,
     V: Clone + Eq + Ord + std::fmt::Display + std::fmt::Debug> crate::anymap::AnyMap for Map64<K, V> {
    type Key = K;
    type Elem = V;
    fn ins(&mut self, k: Self::Key, v: Self::Elem) -> Option<Self::Elem> {
        self.insert(k, v)
    }
    fn rem(&mut self, k: Self::Key) -> Option<Self::Elem> {
        self.remove(k)
    }
    fn ge(&self, k: Self::Key) -> Option<&Self::Elem> {
        self.get(k)
    }
    fn con(&self, k: Self::Key) -> bool {
        self.contains_key(k)
    }
    fn vec(&self) -> Vec<(Self::Key, Self::Elem)> {
        unimplemented!()
    }
    fn ln(&self) -> usize {
        self.len()
    }
}

// #[cfg(test)]
// use proptest::prelude::*;
// #[cfg(test)]
// proptest!{
//     #[test]
//     fn check_string_maps(slice: Vec<(u64,String)>) {
//         crate::anymap::check_map::<Map64<u64,String>>(&slice);
//     }
//     #[test]
//     fn check_u8_maps(slice: Vec<(u8,i8)>) {
//         crate::anymap::check_map::<Map64<u8,i8>>(&slice);
//     }
//     #[test]
//     fn check_i8_maps(slice: Vec<(i8,u8)>) {
//         crate::anymap::check_map::<Map64<i8,u8>>(&slice);
//     }
// }
