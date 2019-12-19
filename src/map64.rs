// Copyright 2019 David Roundy
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A map from [Fits64] types to any other type.

/// A map from a [Fits64] key to 
pub struct Map64<K,V> {
    map: Map64U,
    elems: Vec<V>,
    phantom: std::marker::PhantomData<K>,
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
}

#[test]
fn map64() {
    let mut x: Map64<u8, String> = Map64::new();
    assert!(x.get(b'X').is_none());
    assert!(x.insert(b'X', "X is awesome".to_string()).is_none());
}

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
}

fn log_2(x: u64) -> u8 {
    if x == 0 {
        1
    } else {
        64 - x.leading_zeros() as u8
    }
}
