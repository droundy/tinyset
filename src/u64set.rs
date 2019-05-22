// Copyright 2017-2018 David Roundy
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A set that is compact in size.

use std;

use tinyset::HasInvalid;
use std::mem::ManuallyDrop;

enum SearchResult {
    Present(usize),
    Empty(usize),
    /// The element is not present, but there is someone richer than
    /// us we could steal from!
    Richer(usize),
}
use std::marker::PhantomData;

/// A set implemented of u64 elements
#[derive(Debug,Clone)]
struct U64Set {
    v: Data,
}

const NUM_U8: usize = 22;
const NUM_U16: usize = 11;
const NUM_U32: usize = 5;
const NUM_U64: usize = 2;

#[derive(Debug, Clone)]
enum Data {
    Su8(u8, [u8; NUM_U8]),
    Vu8(u8, Box<[u8]>),
    Su16(u8, [u16; NUM_U16]),
    Vu16(u16, Box<[u16]>),
    Su32(u8, [u32; NUM_U32]),
    Vu32(u32, Box<[u32]>),
    Su64(u32, [u64; NUM_U64]),
    Vu64(u32, Box<[u64]>),
    Badu64(u32, Box<[u64]>),
}
impl Data {
    fn new() -> Data {
        Data::Su8(0, [u8::invalid(); NUM_U8])
    }
    fn with_max_cap(max: u64, cap: usize) -> Data {
        if max < u8::invalid() as u64 {
            if cap <= NUM_U8 {
                Data::Su8(0, [u8::invalid(); NUM_U8])
            } else {
                Data::Vu8(0, vec![u8::invalid(); (cap*11/10).next_power_of_two()]
                          .into_boxed_slice())
            }
        } else if max < u16::invalid() as u64 {
            if cap <= NUM_U16 {
                Data::Su16(0, [u16::invalid(); NUM_U16])
            } else {
                Data::Vu16(0, vec![u16::invalid(); (cap*11/10).next_power_of_two()]
                           .into_boxed_slice())
            }
        } else if max < u32::invalid() as u64 {
            if cap <= NUM_U32 {
                Data::Su32(0, [u32::invalid(); NUM_U32])
            } else {
                Data::Vu32(0, vec![u32::invalid(); (cap*11/10).next_power_of_two()]
                           .into_boxed_slice())
            }
        } else if max < u64::invalid() as u64 {
            if cap <= NUM_U64 {
                Data::Su64(0, [u64::invalid(); NUM_U64])
            } else {
                Data::Vu64(0, vec![u64::invalid(); (cap*11/10).next_power_of_two()]
                           .into_boxed_slice())
            }
        } else {
            Data::Badu64(0, vec![u64::invalid(); ((cap+1)*11/10).next_power_of_two()+1]
                         .into_boxed_slice())
        }
    }
}

fn capacity_to_rawcapacity(cap: usize) -> usize {
    (cap*11/10).next_power_of_two()
}

impl U64Set {
    /// Creates an empty set with the specified capacity.
    fn with_capacity(cap: usize) -> U64Set {
        let nextcap = capacity_to_rawcapacity(cap);
        if cap <= NUM_U8 {
            U64Set { v: Data::new() }
        } else if cap < u8::invalid() as usize {
            U64Set { v: Data::Vu8( 0, vec![u8::invalid(); nextcap].into_boxed_slice()) }
        } else if cap < u16::invalid() as usize {
            U64Set { v: Data::Vu16( 0, vec![u16::invalid(); nextcap].into_boxed_slice()) }
        } else if cap < u32::invalid() as usize {
            U64Set { v: Data::Vu32( 0, vec![u32::invalid(); nextcap].into_boxed_slice()) }
        } else {
            U64Set { v: Data::Vu64(0, vec![u64::invalid(); nextcap].into_boxed_slice()) }
        }
    }
    /// Creates an empty set with the specified capacity.
    fn with_max_and_capacity(max: u64, cap: usize) -> U64Set {
        U64Set { v: Data::with_max_cap(max, cap) }
    }
    /// Returns the number of elements in the set.
    fn len(&self) -> usize {
        match &self.v {
            &Data::Su8(sz,_) => sz as usize,
            &Data::Vu8(sz,_) => sz as usize,
            &Data::Su16(sz,_) => sz as usize,
            &Data::Vu16(sz,_) => sz as usize,
            &Data::Su32(sz,_) => sz as usize,
            &Data::Vu32(sz,_) => sz as usize,
            &Data::Su64(sz,_) => sz as usize,
            &Data::Vu64(sz,_) => sz as usize,
            &Data::Badu64(sz,_) => sz as usize,
        }
    }
    /// Returns the array size.
    fn rawcapacity(&self) -> usize {
        match self.v {
            Data::Su8(_,_) => NUM_U8,
            Data::Vu8(_,ref v) => v.len(),
            Data::Su16(_,_) => NUM_U16,
            Data::Vu16(_,ref v) => v.len(),
            Data::Su32(_,_) => NUM_U32,
            Data::Vu32(_,ref v) => v.len(),
            Data::Su64(_,_) => NUM_U64,
            Data::Vu64(_,ref v) => v.len(),
            Data::Badu64(_,ref v) => v.len()-1,
        }
    }
    /// Reserves capacity for at least `additional` more elements to be
    /// inserted in the set. The collection may reserve more space
    /// to avoid frequent reallocations.
    fn reserve(&mut self, additional: usize) {
        match self.v {
            Data::Su8(sz, v) if sz as usize + additional > NUM_U8 => {
                self.v = Data::Vu8(0, vec![u8::invalid();
                                           ((sz as usize+additional)*11/10).next_power_of_two()]
                                   .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64).ok();
                }
            },
            Data::Su8(_,_) => (),
            _ => unimplemented!(),
        }
    }
    /// Reserves capacity for at least `additional` more elements to
    /// be inserted in the set, with maximum value of `max`. The
    /// collection may reserve more space to avoid frequent
    /// reallocations.
    fn reserve_with_max(&mut self, max: u64, additional: usize) {
        match self.v {
            Data::Su8(sz, v) if max >= u8::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as u64).ok();
                }
                *self = n;
            },
            Data::Su8(sz, v) if sz as usize + additional > NUM_U8 => {
                self.v = Data::Vu8(0, vec![u8::invalid();
                                           ((sz as usize+additional)*11/10).next_power_of_two()]
                                   .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64).ok();
                }
            },
            Data::Su8(_,_) => (),
            Data::Su16(sz, v) if max >= u16::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as u64).ok();
                }
                *self = n;
            },
            Data::Su16(sz, v) if sz as usize + additional > NUM_U16 => {
                self.v = Data::Vu16(0, vec![u16::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64).ok();
                }
            },
            Data::Su16(_,_) => (),
            Data::Su32(sz, v) if max >= u32::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as u64).ok();
                }
                *self = n;
            },
            Data::Su32(sz, v) if sz as usize + additional > NUM_U32 => {
                self.v = Data::Vu32(0, vec![u32::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64).ok();
                }
            },
            Data::Su32(_,_) => (),
            Data::Su64(sz, v) if max >= u64::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as u64).ok();
                }
                *self = n;
            },
            Data::Su64(sz, v) if sz as usize + additional > NUM_U64 => {
                self.v = Data::Vu64(0, vec![u64::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64).ok();
                }
            },
            Data::Su64(_,_) => (),
            Data::Vu8(sz, _) if max >= u8::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x).ok();
                }
                *self = n;
            },
            Data::Vu16(sz, _) if max >= u16::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x).ok();
                }
                *self = n;
            },
            Data::Vu32(sz, _) if max >= u32::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x).ok();
                }
                *self = n;
            },
            Data::Vu64(_, _) if max >= u64::invalid() as u64 => {
                unimplemented!();
            },
            Data::Vu8(sz, ref mut v) if sz as usize + additional > v.len()*10/11 => {
                let newcap = ((sz as usize+additional)*11/10).next_power_of_two();
                let oldv = std::mem::replace(v, vec![u8::invalid(); newcap]
                                             .into_boxed_slice());
                for &x in oldv.iter() {
                    if x != u8::invalid() {
                        let mut value = x;
                        match search(v, value, u8::invalid()) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value, u8::invalid());
                            },
                        }
                    }
                }
            },
            Data::Vu8(_,_) => (),
            Data::Vu16(sz, ref mut v) if sz as usize + additional > v.len()*10/11 => {
                let newcap = ((sz as usize+additional)*11/10).next_power_of_two();
                let oldv = std::mem::replace(v, vec![u16::invalid(); newcap]
                                             .into_boxed_slice());
                for &x in oldv.iter() {
                    if x != u16::invalid() {
                        let mut value = x;
                        match search(v, value, u16::invalid()) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value, u16::invalid());
                            },
                        }
                    }
                }
            },
            Data::Vu16(_,_) => (),
            Data::Vu32(sz, ref mut v) if sz as usize + additional > v.len()*10/11 => {
                let newcap = ((sz as usize+additional)*11/10).next_power_of_two();
                let oldv = std::mem::replace(v, vec![u32::invalid(); newcap]
                                             .into_boxed_slice());
                for &x in oldv.iter() {
                    if x != u32::invalid() {
                        let mut value = x;
                        match search(v, value, u32::invalid()) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value, u32::invalid());
                            },
                        }
                    }
                }
            },
            Data::Vu32(_,_) => (),
            Data::Vu64(sz, ref mut v) if sz as usize + additional > v.len()*10/11 => {
                let newcap = ((sz as usize+additional)*11/10).next_power_of_two();
                let oldv = std::mem::replace(v, vec![u64::invalid(); newcap]
                                             .into_boxed_slice());
                for &x in oldv.iter() {
                    if x != u64::invalid() {
                        let mut value = x;
                        match search(v, value, u64::invalid()) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value, u64::invalid());
                            },
                        }
                    }
                }
            },
            Data::Vu64(_,_) => (),
            Data::Badu64(sz, ref mut v) if sz as usize + additional + 1 > v.len()*10/11 => {
                let newcap = ((sz as usize+additional+1)*11/10).next_power_of_two();
                let invalid = v[v.len()-1];
                let oldv = std::mem::replace(v, vec![invalid; newcap+1]
                                             .into_boxed_slice());
                for &x in oldv.iter() {
                    if x != invalid {
                        let vlen = v.len();
                        let mut v = &mut v[..vlen-1];
                        let mut value = x;
                        match search(v, value, invalid) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value, invalid);
                            },
                        }
                    }
                }
            },
            Data::Badu64(_,_) => (),
        }
    }
    fn current_max(&self) -> u64 {
        match self.v {
            Data::Su8(_, _) => u8::invalid() as u64 - 1,
            Data::Su16(_, _) => u16::invalid() as u64 - 1,
            Data::Su32(_, _) => u32::invalid() as u64 - 1,
            Data::Su64(_, _) => u64::invalid() as u64 - 1,
            Data::Vu8(_, _) => u8::invalid() as u64 - 1,
            Data::Vu16(_, _) => u16::invalid() as u64 - 1,
            Data::Vu32(_, _) => u32::invalid() as u64 - 1,
            Data::Vu64(_, _) => u64::invalid() as u64 - 1,
            Data::Badu64(_, _) => u64::invalid(),
        }
    }
    fn index(&self, i: usize) -> Option<u64> {
        match self.v {
            Data::Su8(sz, ref v) =>
                if i < sz as usize && v[i] != u8::invalid() {
                    Some(v[i] as u64)
                } else {
                    None
                },
            Data::Su16(sz, ref v) =>
                if i < sz as usize && v[i] != u16::invalid() {
                    Some(v[i] as u64)
                } else {
                    None
                },
            Data::Su32(sz, ref v) =>
                if i < sz as usize && v[i] != u32::invalid() {
                    Some(v[i] as u64)
                } else {
                    None
                },
            Data::Su64(sz, ref v) =>
                if i < sz as usize && v[i] != u64::invalid() {
                    Some(v[i] as u64)
                } else {
                    None
                },
            Data::Vu8(_, ref v) =>
                if v[i] != u8::invalid() {
                    Some(v[i] as u64)
                } else {
                    None
                },
            Data::Vu16(_, ref v) =>
                if v[i] != u16::invalid() {
                    Some(v[i] as u64)
                } else {
                    None
                },
            Data::Vu32(_, ref v) =>
                if v[i] != u32::invalid() {
                    Some(v[i] as u64)
                } else {
                    None
                },
            Data::Vu64(_, ref v) =>
                if v[i] != u64::invalid() {
                    Some(v[i] as u64)
                } else {
                    None
                },
            Data::Badu64(_, ref v) => {
                let invalid = v[v.len()-1];
                if v[i] != invalid {
                    Some(v[i] as u64)
                } else {
                    None
                }
            },
        }
    }
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    fn insert(&mut self, elem: u64) -> bool {
        self.reserve_with_max(elem, 1);
        self.insert_unchecked(elem).is_ok()
    }
    /// An Ok value means that this is a new value, an Err value
    /// (sorry!) means that this thing was already in the set.
    fn insert_unchecked(&mut self, value: u64) -> Result<usize,usize> {
        match self.v {
            Data::Su8(ref mut sz, ref mut v) => {
                let value = value as u8;
                for (i,&x) in v.iter().enumerate().take(*sz as usize) {
                    if x == value {
                        return Err(i);
                    }
                }
                v[*sz as usize] = value;
                *sz += 1;
                Ok(*sz as usize -1)
            },
            Data::Su16(ref mut sz, ref mut v) => {
                let value = value as u16;
                for (i,&x) in v.iter().enumerate().take(*sz as usize) {
                    if x == value {
                        return Err(i);
                    }
                }
                v[*sz as usize] = value;
                *sz += 1;
                Ok(*sz as usize -1)
            },
            Data::Su32(ref mut sz, ref mut v) => {
                let value = value as u32;
                for (i,&x) in v.iter().enumerate().take(*sz as usize) {
                    if x == value {
                        return Err(i);
                    }
                }
                v[*sz as usize] = value;
                *sz += 1;
                Ok(*sz as usize -1)
            },
            Data::Su64(ref mut sz, ref mut v) => {
                let value = value as u64;
                for (i,&x) in v.iter().enumerate().take(*sz as usize) {
                    if x == value {
                        return Err(i);
                    }
                }
                v[*sz as usize] = value;
                *sz += 1;
                Ok(*sz as usize -1)
            },
            Data::Vu8(ref mut sz, ref mut v) => {
                let mut value = value as u8;
                match search(v, value, u8::invalid()) {
                    SearchResult::Present(i) => Err(i),
                    SearchResult::Empty(i) => {
                        v[i] = value;
                        *sz += 1;
                        Ok(i)
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut v[i], &mut value);
                        steal(v, i, value, u8::invalid());
                        Ok(i)
                    },
                }
            },
            Data::Vu16(ref mut sz, ref mut v) => {
                let mut value = value as u16;
                match search(v, value, u16::invalid()) {
                    SearchResult::Present(i) => Err(i),
                    SearchResult::Empty(i) => {
                        v[i] = value;
                        *sz += 1;
                        Ok(i)
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut v[i], &mut value);
                        steal(v, i, value, u16::invalid());
                        Ok(i)
                    },
                }
            },
            Data::Vu32(ref mut sz, ref mut v) => {
                let mut value = value as u32;
                match search(v, value, u32::invalid()) {
                    SearchResult::Present(i) => Err(i),
                    SearchResult::Empty(i) => {
                        v[i] = value;
                        *sz += 1;
                        Ok(i)
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut v[i], &mut value);
                        steal(v, i, value, u32::invalid());
                        Ok(i)
                    },
                }
            },
            Data::Vu64(ref mut sz, ref mut v) => {
                let mut value = value as u64;
                match search(v, value, u64::invalid()) {
                    SearchResult::Present(i) => Err(i),
                    SearchResult::Empty(i) => {
                        v[i] = value;
                        *sz += 1;
                        Ok(i)
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut v[i], &mut value);
                        steal(v, i, value, u64::invalid());
                        Ok(i)
                    },
                }
            },
            Data::Badu64(_,_) => {
                let mut invalid = 0;
                let mut old_invalid = 0;
                let mut value = value as u64;
                if let Data::Badu64(_, ref v) = self.v {
                    invalid = v[v.len()-1];
                    old_invalid = invalid;
                    if value == invalid {
                        // Need to pick a new invalid value.
                        invalid = invalid.wrapping_sub(1);
                        while self.contains(&invalid).is_some() {
                            invalid = invalid.wrapping_sub(1);
                        }
                    }
                }
                if let Data::Badu64(ref mut sz, ref mut v) = self.v {
                    let vlen = v.len();
                    if old_invalid != invalid {
                        for x in v.iter_mut() {
                            if *x == old_invalid {
                                *x = invalid;
                            }
                        }
                    }
                    let mut v = &mut v[..vlen-1];
                    match search(v, value, invalid) {
                        SearchResult::Present(i) => Err(i),
                        SearchResult::Empty(i) => {
                            v[i] = value;
                            *sz += 1;
                            Ok(i)
                        },
                        SearchResult::Richer(i) => {
                            *sz += 1;
                            std::mem::swap(&mut v[i], &mut value);
                            steal(v, i, value, invalid);
                            Ok(i)
                        },
                    }
                } else {
                    unreachable!()
                }
            },
        }
    }
    fn co_insert_unchecked<V>(&mut self, vals: &mut [V], k: u64, mut v: V) -> Option<V> {
        match self.v {
            Data::Su8(ref mut sz, ref mut keys) => {
                let k = k as u8;
                for i in 0..*sz as usize {
                    if keys[i] == k {
                        return Some(std::mem::replace(&mut vals[i], v));
                    }
                }
                keys[*sz as usize] = k;
                vals[*sz as usize] = v;
                *sz += 1;
                None
            },
            Data::Su16(ref mut sz, ref mut keys) => {
                let k = k as u16;
                for i in 0..*sz as usize {
                    if keys[i] == k {
                        return Some(std::mem::replace(&mut vals[i], v));
                    }
                }
                keys[*sz as usize] = k;
                vals[*sz as usize] = v;
                *sz += 1;
                None
            },
            Data::Su32(ref mut sz, ref mut keys) => {
                let k = k as u32;
                for i in 0..*sz as usize {
                    if keys[i] == k {
                        return Some(std::mem::replace(&mut vals[i], v));
                    }
                }
                keys[*sz as usize] = k;
                vals[*sz as usize] = v;
                *sz += 1;
                None
            },
            Data::Su64(ref mut sz, ref mut keys) => {
                let k = k as u64;
                for i in 0..*sz as usize {
                    if keys[i] == k {
                        return Some(std::mem::replace(&mut vals[i], v));
                    }
                }
                keys[*sz as usize] = k;
                vals[*sz as usize] = v;
                *sz += 1;
                None
            },
            Data::Vu8(ref mut sz, ref mut keys) => {
                let mut k = k as u8;
                match search(keys, k, u8::invalid()) {
                    SearchResult::Present(i) => {
                        return Some(std::mem::replace(&mut vals[i], v));
                    },
                    SearchResult::Empty(i) => {
                        keys[i] = k;
                        vals[i] = v;
                        *sz += 1;
                        None
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v, u8::invalid());
                        None
                    },
                }
            },
            Data::Vu16(ref mut sz, ref mut keys) => {
                let mut k = k as u16;
                match search(keys, k, u16::invalid()) {
                    SearchResult::Present(i) => {
                        return Some(std::mem::replace(&mut vals[i], v));
                    },
                    SearchResult::Empty(i) => {
                        keys[i] = k;
                        vals[i] = v;
                        *sz += 1;
                        None
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v, u16::invalid());
                        None
                    },
                }
            },
            Data::Vu32(ref mut sz, ref mut keys) => {
                let mut k = k as u32;
                match search(keys, k, u32::invalid()) {
                    SearchResult::Present(i) => {
                        return Some(std::mem::replace(&mut vals[i], v));
                    },
                    SearchResult::Empty(i) => {
                        keys[i] = k;
                        vals[i] = v;
                        *sz += 1;
                        None
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v, u32::invalid());
                        None
                    },
                }
            },
            Data::Vu64(ref mut sz, ref mut keys) => {
                let mut k = k as u64;
                match search(keys, k, u64::invalid()) {
                    SearchResult::Present(i) => {
                        return Some(std::mem::replace(&mut vals[i], v));
                    },
                    SearchResult::Empty(i) => {
                        keys[i] = k;
                        vals[i] = v;
                        *sz += 1;
                        None
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v, u64::invalid());
                        None
                    },
                }
            },
            Data::Badu64(ref mut sz, ref mut keys) => {
                let invalid = keys[keys.len()-1];
                let mut k = k as u64;
                let vlen = keys.len();
                let mut keys = &mut keys[..vlen-1];
                match search(keys, k, invalid) {
                    SearchResult::Present(i) => {
                        return Some(std::mem::replace(&mut vals[i], v));
                    },
                    SearchResult::Empty(i) => {
                        keys[i] = k;
                        vals[i] = v;
                        *sz += 1;
                        None
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v, invalid);
                        None
                    },
                }
            },
        }
    }
    /// Returns true if the set contains a value.
    pub fn contains(&self, value: &u64) -> Option<usize> {
        let value = *value;
        match self.v {
            Data::Su8(sz, ref v) => {
                if value >= u8::invalid() as u64 {
                    return None;
                }
                let value = value as u8;
                for (i,&x) in v.iter().enumerate().take(sz as usize) {
                    if x == value {
                        return Some(i);
                    }
                }
                None
            },
            Data::Su16(sz, ref v) => {
                if value >= u16::invalid() as u64 {
                    return None;
                }
                let value = value as u16;
                for (i,&x) in v.iter().enumerate().take(sz as usize) {
                    if x == value {
                        return Some(i);
                    }
                }
                None
            },
            Data::Su32(sz, ref v) => {
                if value >= u32::invalid() as u64 {
                    return None;
                }
                let value = value as u32;
                for (i,&x) in v.iter().enumerate().take(sz as usize) {
                    if x == value {
                        return Some(i);
                    }
                }
                None
            },
            Data::Su64(sz, ref v) => {
                if value >= u64::invalid() as u64 {
                    return None;
                }
                let value = value as u64;
                for (i,&x) in v.iter().enumerate().take(sz as usize) {
                    if x == value {
                        return Some(i);
                    }
                }
                None
            },
            Data::Vu8(_, ref v) => {
                if value >= u8::invalid() as u64 {
                    return None;
                }
                let value = value as u8;
                match search(v, value, u8::invalid()) {
                    SearchResult::Present(i) => Some(i),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            Data::Vu16(_, ref v) => {
                if value >= u16::invalid() as u64 {
                    return None;
                }
                let value = value as u16;
                match search(v, value, u16::invalid()) {
                    SearchResult::Present(i) => Some(i),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            Data::Vu32(_, ref v) => {
                if value >= u32::invalid() as u64 {
                    return None;
                }
                let value = value as u32;
                match search(v, value, u32::invalid()) {
                    SearchResult::Present(i) => Some(i),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            Data::Vu64(_, ref v) => {
                if value >= u64::invalid() as u64 {
                    return None;
                }
                let value = value as u64;
                match search(v, value, u64::invalid()) {
                    SearchResult::Present(i) => Some(i),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            Data::Badu64(_, ref v) => {
                let invalid = v[v.len()-1];
                if value == invalid as u64 {
                    return None;
                }
                let value = value as u64;
                let vlen = v.len();
                let mut v = &v[..vlen-1];
                match search(v, value, invalid) {
                    SearchResult::Present(i) => Some(i),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
        }
    }
    /// Removes an element, and returns true if that element was present.
    pub fn remove(&mut self, value: &u64) -> bool {
        let value = *value;
        match self.v {
            Data::Su8(ref mut sz, ref mut v) => {
                if value >= u8::invalid() as u64 {
                    return false;
                }
                let value = value as u8;
                let mut i = None;
                for (j, &x) in v.iter().enumerate().take(*sz as usize) {
                    if x == value {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    v[i] = v[*sz as usize -1];
                    *sz -= 1;
                    true
                } else {
                    false
                };
            },
            Data::Su16(ref mut sz, ref mut v) => {
                if value >= u16::invalid() as u64 {
                    return false;
                }
                let value = value as u16;
                let mut i = None;
                for (j, &x) in v.iter().enumerate().take(*sz as usize) {
                    if x == value {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    v[i] = v[*sz as usize -1];
                    *sz -= 1;
                    true
                } else {
                    false
                };
            },
            Data::Su32(ref mut sz, ref mut v) => {
                if value >= u32::invalid() as u64 {
                    return false;
                }
                let value = value as u32;
                let mut i = None;
                for (j, &x) in v.iter().enumerate().take(*sz as usize) {
                    if x == value {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    v[i] = v[*sz as usize -1];
                    *sz -= 1;
                    true
                } else {
                    false
                };
            },
            Data::Su64(ref mut sz, ref mut v) => {
                if value >= u64::invalid() as u64 {
                    return false;
                }
                let value = value as u64;
                let mut i = None;
                for (j, &x) in v.iter().enumerate().take(*sz as usize) {
                    if x == value {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    v[i] = v[*sz as usize -1];
                    *sz -= 1;
                    true
                } else {
                    false
                };
            },
            Data::Vu8(ref mut sz, ref mut v) => {
                if value >= u8::invalid() as u64 {
                    return false;
                }
                let value = value as u8;
                match search(v, value, u8::invalid()) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = v.len() - 1;
                        let invalid = u8::invalid();
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
            },
            Data::Vu16(ref mut sz, ref mut v) => {
                if value >= u16::invalid() as u64 {
                    return false;
                }
                let value = value as u16;
                match search(v, value, u16::invalid()) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = v.len() - 1;
                        let invalid = u16::invalid();
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
            },
            Data::Vu32(ref mut sz, ref mut v) => {
                if value >= u32::invalid() as u64 {
                    return false;
                }
                let value = value as u32;
                match search(v, value, u32::invalid()) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = v.len() - 1;
                        let invalid = u32::invalid();
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
            },
            Data::Vu64(ref mut sz, ref mut v) => {
                if value >= u64::invalid() as u64 {
                    return false;
                }
                let value = value as u64;
                match search(v, value, u64::invalid()) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = v.len() - 1;
                        let invalid = u64::invalid();
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
            },
            Data::Badu64(ref mut sz, ref mut v) => {
                let invalid = v[v.len()-1];
                if value == invalid {
                    return false;
                }
                let value = value as u64;
                let vlen = v.len();
                let mut v = &mut v[..vlen-1];
                match search(v, value, invalid) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = v.len() - 1;
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
            },
        }
    }
    /// Removes an element, and returns true if that element was present.
    pub fn co_remove<V>(&mut self, vals: &mut [V], k: u64) -> Option<V> {
        match self.v {
            Data::Su8(ref mut sz, ref mut keys) => {
                if k >= u8::invalid() as u64 {
                    return None;
                }
                let k = k as u8;
                let mut i = None;
                for (j, &x) in keys.iter().enumerate().take(*sz as usize) {
                    if x == k {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    if i == *sz as usize - 1 {
                        *sz -= 1;
                        Some(std::mem::replace(&mut vals[i],
                                               unsafe {std::mem::uninitialized()}))
                    } else {
                        let lastv = std::mem::replace(&mut vals[*sz as usize -1],
                                                      unsafe {std::mem::uninitialized()});
                        let oldv = std::mem::replace(&mut vals[i], lastv);
                        keys[i] = keys[*sz as usize -1];
                        *sz -= 1;
                        Some(oldv)
                    }
                } else {
                    None
                };
            },
            Data::Su16(ref mut sz, ref mut keys) => {
                if k >= u16::invalid() as u64 {
                    return None;
                }
                let k = k as u16;
                let mut i = None;
                for (j, &x) in keys.iter().enumerate().take(*sz as usize) {
                    if x == k {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    if i == *sz as usize - 1 {
                        *sz -= 1;
                        Some(std::mem::replace(&mut vals[i],
                                               unsafe {std::mem::uninitialized()}))
                    } else {
                        let lastv = std::mem::replace(&mut vals[*sz as usize -1],
                                                      unsafe {std::mem::uninitialized()});
                        let oldv = std::mem::replace(&mut vals[i], lastv);
                        keys[i] = keys[*sz as usize -1];
                        *sz -= 1;
                        Some(oldv)
                    }
                } else {
                    None
                };
            },
            Data::Su32(ref mut sz, ref mut keys) => {
                if k >= u32::invalid() as u64 {
                    return None;
                }
                let k = k as u32;
                let mut i = None;
                for (j, &x) in keys.iter().enumerate().take(*sz as usize) {
                    if x == k {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    if i == *sz as usize - 1 {
                        *sz -= 1;
                        Some(std::mem::replace(&mut vals[i],
                                               unsafe {std::mem::uninitialized()}))
                    } else {
                        let lastv = std::mem::replace(&mut vals[*sz as usize -1],
                                                      unsafe {std::mem::uninitialized()});
                        let oldv = std::mem::replace(&mut vals[i], lastv);
                        keys[i] = keys[*sz as usize -1];
                        *sz -= 1;
                        Some(oldv)
                    }
                } else {
                    None
                };
            },
            Data::Su64(ref mut sz, ref mut keys) => {
                if k >= u64::invalid() as u64 {
                    return None;
                }
                let k = k as u64;
                let mut i = None;
                for (j, &x) in keys.iter().enumerate().take(*sz as usize) {
                    if x == k {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    if i == *sz as usize - 1 {
                        *sz -= 1;
                        Some(std::mem::replace(&mut vals[i],
                                               unsafe {std::mem::uninitialized()}))
                    } else {
                        let lastv = std::mem::replace(&mut vals[*sz as usize -1],
                                                      unsafe {std::mem::uninitialized()});
                        let oldv = std::mem::replace(&mut vals[i], lastv);
                        keys[i] = keys[*sz as usize -1];
                        *sz -= 1;
                        Some(oldv)
                    }
                } else {
                    None
                };
            },
            Data::Vu8(ref mut sz, ref mut keys) => {
                if k >= u8::invalid() as u64 {
                    return None;
                }
                let k = k as u8;
                match search(keys, k, u8::invalid()) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = u8::invalid();
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(std::mem::replace(&mut vals[i],
                                                              unsafe {std::mem::uninitialized()}));
                            }
                            keys[i] = keys[iplus1];
                            vals.swap(i, iplus1);
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            Data::Vu16(ref mut sz, ref mut keys) => {
                if k >= u16::invalid() as u64 {
                    return None;
                }
                let k = k as u16;
                match search(keys, k, u16::invalid()) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = u16::invalid();
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(std::mem::replace(&mut vals[i],
                                                              unsafe {std::mem::uninitialized()}));
                            }
                            keys[i] = keys[iplus1];
                            vals.swap(i, iplus1);
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            Data::Vu32(ref mut sz, ref mut keys) => {
                if k >= u32::invalid() as u64 {
                    return None;
                }
                let k = k as u32;
                match search(keys, k, u32::invalid()) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = u32::invalid();
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(std::mem::replace(&mut vals[i],
                                                              unsafe {std::mem::uninitialized()}));
                            }
                            keys[i] = keys[iplus1];
                            vals.swap(i, iplus1);
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            Data::Vu64(ref mut sz, ref mut keys) => {
                if k >= u64::invalid() as u64 {
                    return None;
                }
                let k = k as u64;
                match search(keys, k, u64::invalid()) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = u64::invalid();
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(std::mem::replace(&mut vals[i],
                                                              unsafe {std::mem::uninitialized()}));
                            }
                            keys[i] = keys[iplus1];
                            vals.swap(i, iplus1);
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            Data::Badu64(ref mut sz, ref mut keys) => {
                let invalid = keys[keys.len()-1];
                if k == invalid {
                    return None;
                }
                let k = k as u64;
                let vlen = keys.len();
                let mut keys = &mut keys[..vlen-1];
                match search(keys, k, invalid) {
                    SearchResult::Present(mut i) => {
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(std::mem::replace(&mut vals[i],
                                                              unsafe {std::mem::uninitialized()}));
                            }
                            keys[i] = keys[iplus1];
                            vals.swap(i, iplus1);
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
        }
    }
    /// Returns an iterator over the set.
    pub fn iter(&self) -> Iter {
        match self.v {
            Data::Su8(sz, ref v) => {
                Iter::U8 {
                    slice: &v[0..sz as usize],
                    nleft: sz as usize,
                }
            },
            Data::Vu8(sz, ref v) => {
                Iter::U8 {
                    slice: v,
                    nleft: sz as usize,
                }
            },
            Data::Su16(sz, ref v) => {
                Iter::U16 {
                    slice: &v[0..sz as usize],
                    nleft: sz as usize,
                }
            },
            Data::Vu16(sz, ref v) => {
                Iter::U16 {
                    slice: v,
                    nleft: sz as usize,
                }
            },
            Data::Su32(sz, ref v) => {
                Iter::U32 {
                    slice: &v[0..sz as usize],
                    nleft: sz as usize,
                }
            },
            Data::Vu32(sz, ref v) => {
                Iter::U32 {
                    slice: v,
                    nleft: sz as usize,
                }
            },
            Data::Su64(sz, ref v) => {
                Iter::U64 {
                    invalid: u64::invalid(),
                    slice: &v[0..sz as usize],
                    nleft: sz as usize,
                }
            },
            Data::Vu64(sz, ref v) => {
                Iter::U64 {
                    invalid: u64::invalid(),
                    slice: v,
                    nleft: sz as usize,
                }
            },
            Data::Badu64(sz, ref v) => {
                Iter::U64 {
                    invalid: v[v.len()-1],
                    slice: &v[..v.len()-1],
                    nleft: sz as usize,
                }
            },
        }
    }
    /// Clears the set, returning all elements in an iterator.
    pub fn drain(&mut self) -> Drain {
        match self.v {
            Data::Su8(ref mut sz, ref mut v) => {
                let oldv = std::mem::replace(v, [u8::invalid(); NUM_U8]);
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(&oldv[0..oldsz]);
                Drain::U8 {
                    slice: oldv,
                    nleft: oldsz,
                }
            },
            Data::Vu8(ref mut sz, ref mut v) => {
                let len = v.len();
                let oldv = std::mem::replace(v,
                                             vec![u8::invalid(); len].into_boxed_slice());
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(oldv);
                Drain::U8 {
                    slice: oldv,
                    nleft: oldsz,
                }
            },
            Data::Su16(ref mut sz, ref mut v) => {
                let oldv = std::mem::replace(v, [u16::invalid(); NUM_U16]);
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(&oldv[0..oldsz]);
                Drain::U16 {
                    slice: oldv,
                    nleft: oldsz,
                }
            },
            Data::Vu16(ref mut sz, ref mut v) => {
                let len = v.len();
                let oldv = std::mem::replace(v,
                                             vec![u16::invalid(); len].into_boxed_slice());
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(oldv);
                Drain::U16 {
                    slice: oldv,
                    nleft: oldsz,
                }
            },
            Data::Su32(ref mut sz, ref mut v) => {
                let oldv = std::mem::replace(v, [u32::invalid(); NUM_U32]);
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(&oldv[0..oldsz]);
                Drain::U32 {
                    slice: oldv,
                    nleft: oldsz,
                }
            },
            Data::Vu32(ref mut sz, ref mut v) => {
                let len = v.len();
                let oldv = std::mem::replace(v,
                                             vec![u32::invalid(); len].into_boxed_slice());
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(oldv);
                Drain::U32 {
                    slice: oldv,
                    nleft: oldsz,
                }
            },
            Data::Su64(ref mut sz, ref mut v) => {
                let oldv = std::mem::replace(v, [u64::invalid(); NUM_U64]);
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(&oldv[0..oldsz]);
                Drain::U64 {
                    invalid: u64::invalid(),
                    slice: oldv,
                    nleft: oldsz,
                }
            },
            Data::Vu64(ref mut sz, ref mut v) => {
                let len = v.len();
                let oldv = std::mem::replace(v,
                                             vec![u64::invalid(); len].into_boxed_slice());
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(oldv);
                Drain::U64 {
                    invalid: u64::invalid(),
                    slice: oldv,
                    nleft: oldsz,
                }
            },
            Data::Badu64(ref mut sz, ref mut v) => {
                let len = v.len();
                let oldv = std::mem::replace(v,
                                             vec![u64::invalid(); len].into_boxed_slice());
                let oldsz = std::mem::replace(sz, 0) as usize;
                let oldv = Vec::from(oldv);
                Drain::U64 {
                    invalid: oldv[len-1],
                    slice: oldv,
                    nleft: oldsz,
                }
            },
        }
    }
}

impl std::iter::FromIterator<u64> for U64Set {
    fn from_iter<I: IntoIterator<Item=u64>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (sz,_) = iter.size_hint();
        let mut c = U64Set::with_capacity(sz);
        for i in iter {
            c.insert(i);
        }
        c
    }
}

/// An iterator for `U64Set`.
pub enum Iter<'a> {
    /// this really should be private
    U8 {
        /// this really should be private
        slice: &'a [u8],
        /// this really should be private
        nleft: usize,
    },
    /// this really should be private
    U16 {
        /// this really should be private
        slice: &'a [u16],
        /// this really should be private
        nleft: usize,
    },
    /// this really should be private
    U32 {
        /// this really should be private
        slice: &'a [u32],
        /// this really should be private
        nleft: usize,
    },
    /// this really should be private
    U64 {
        /// should be private
        invalid: u64,
        /// this really should be private
        slice: &'a [u64],
        /// this really should be private
        nleft: usize,
    },
}
/// A draining iterator for `U64Set`.
pub enum Drain {
    /// this really should be private
    U8 {
        /// this really should be private
        slice: Vec<u8>,
        /// this really should be private
        nleft: usize,
    },
    /// this really should be private
    U16 {
        /// this really should be private
        slice: Vec<u16>,
        /// this really should be private
        nleft: usize,
    },
    /// this really should be private
    U32 {
        /// this really should be private
        slice: Vec<u32>,
        /// this really should be private
        nleft: usize,
    },
    /// this really should be private
    U64 {
        /// this really should be private
        invalid: u64,
        /// this really should be private
        slice: Vec<u64>,
        /// this really should be private
        nleft: usize,
    },
}

impl<'a> Iterator for Iter<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        match self {
            &mut Iter::U8{ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    while slice[0] == u8::invalid() {
                        *slice = slice.split_first().unwrap().1;
                    }
                    let val = slice[0];
                    *slice = slice.split_first().unwrap().1;
                    *nleft -= 1;
                    Some(val as u64)
                }
            },
            &mut Iter::U16{ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    while slice[0] == u16::invalid() {
                        *slice = slice.split_first().unwrap().1;
                    }
                    let val = slice[0];
                    *slice = slice.split_first().unwrap().1;
                    *nleft -= 1;
                    Some(val as u64)
                }
            },
            &mut Iter::U32{ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    while slice[0] == u32::invalid() {
                        *slice = slice.split_first().unwrap().1;
                    }
                    let val = slice[0];
                    *slice = slice.split_first().unwrap().1;
                    *nleft -= 1;
                    Some(val as u64)
                }
            },
            &mut Iter::U64{invalid, ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    while slice[0] == invalid {
                        *slice = slice.split_first().unwrap().1;
                    }
                    let val = slice[0];
                    *slice = slice.split_first().unwrap().1;
                    *nleft -= 1;
                    Some(val as u64)
                }
            },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            &Iter::U8{slice: _, nleft} => (nleft, Some(nleft)),
            &Iter::U16{slice: _, nleft} => (nleft, Some(nleft)),
            &Iter::U32{slice: _, nleft} => (nleft, Some(nleft)),
            &Iter::U64{nleft, ..} => (nleft, Some(nleft)),
        }
    }
}

impl Iterator for Drain {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        match self {
            &mut Drain::U8{ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    let mut val = slice.pop().unwrap();
                    while val == u8::invalid() {
                        val = slice.pop().unwrap();
                    }
                    *nleft -= 1;
                    Some(val as u64)
                }
            },
            &mut Drain::U16{ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    let mut val = slice.pop().unwrap();
                    while val == u16::invalid() {
                        val = slice.pop().unwrap();
                    }
                    *nleft -= 1;
                    Some(val as u64)
                }
            },
            &mut Drain::U32{ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    let mut val = slice.pop().unwrap();
                    while val == u32::invalid() {
                        val = slice.pop().unwrap();
                    }
                    *nleft -= 1;
                    Some(val as u64)
                }
            },
            &mut Drain::U64{invalid, ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    let mut val = slice.pop().unwrap();
                    while val == invalid {
                        val = slice.pop().unwrap();
                    }
                    *nleft -= 1;
                    Some(val as u64)
                }
            },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            &Drain::U8{slice: _, nleft} => (nleft, Some(nleft)),
            &Drain::U16{slice: _, nleft} => (nleft, Some(nleft)),
            &Drain::U32{slice: _, nleft} => (nleft, Some(nleft)),
            &Drain::U64{nleft, ..} => (nleft, Some(nleft)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn it_works() {
        let mut ss = Set64::<u64>::new();
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
            assert!(ss.contains(&num));
        }
        assert!(!ss.remove(&2));
        assert!(ss.remove(&3));
        assert!(!ss.contains(&3));
        assert_eq!(ss.len(), 1);
    }
    #[test]
    fn size_unwasted() {
        println!("small size: {}", std::mem::size_of::<U64Set>());
        println!(" hash size: {}", std::mem::size_of::<HashSet<u64>>());
        assert!(std::mem::size_of::<U64Set>() <=
                2*std::mem::size_of::<HashSet<u64>>());
        assert!(std::mem::size_of::<U64Set>() <= 24);
    }

    #[test]
    fn test_matches() {
        let mut steps: Vec<Result<u64,u64>> = vec![Err(8), Ok(0), Ok(16), Ok(1), Ok(8)];
        let mut set = U64Set::with_capacity(1);
        let mut refset = HashSet::<u64>::new();
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
                if set.contains(&i).is_some() != refset.contains(&i) {
                    println!("trouble at {}", i);
                    assert_eq!(set.contains(&i).is_some(), refset.contains(&i));
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches(steps: Vec<Result<u64,u64>>) -> bool {
            let mut steps = steps;
            let mut set = U64Set::with_capacity(1);
            let mut refset = HashSet::<u64>::new();
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
                    if set.contains(&i).is_some() != refset.contains(&i) { return false; }
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches_with_invalid(steps: Vec<Result<u64,u64>>) -> bool {
            let mut steps = steps;
            steps.push(Ok(u64::invalid()));
            let mut set = U64Set::with_capacity(1);
            let mut refset = HashSet::<u64>::new();
            loop {
                match steps.pop() {
                    Some(Ok(v)) => {
                        set.insert(v);
                        assert!(set.contains(&v).is_some());
                        refset.insert(v);
                    },
                    Some(Err(v)) => {
                        set.remove(&v);
                        assert!(!set.contains(&v).is_some());
                        refset.remove(&v);
                    },
                    None => return true,
                }
                if set.len() != refset.len() { return false; }
                for i in 0..2550 {
                    if set.contains(&i).is_some() != refset.contains(&i) { return false; }
                }
                let inv = u64::invalid();
                for i in &[inv-3,inv-2,inv-1,inv] {
                    if set.contains(&i).is_some() != refset.contains(&i) { return false; }
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches_64(steps: Vec<Result<u64,u64>>) -> bool {
            let mut steps = steps;
            let mut set = Set64::<u64>::new();
            let mut refset = HashSet::<u64>::new();
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
                for x in set.iter() {
                    if !refset.contains(&x) { return false; }
                }
                for x in refset.iter() {
                    if !set.contains(x) { return false; }
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches_32(steps: Vec<Result<u32,u32>>) -> bool {
            let mut steps = steps;
            let mut set = Set64::<u32>::new();
            let mut refset = HashSet::<u32>::new();
            loop {
                match steps.pop() {
                    Some(Ok(v)) => {
                        set.insert(v); refset.insert(v);
                    },
                    Some(Err(v)) => {
                        set.remove(&v); refset.remove(&v);
                    },
                    None => {
                        for x in set.drain() {
                            if !refset.contains(&x) {
                                println!("draining {} not in {:?}", x, &refset);
                                return false;
                            }
                        }
                        if set.len() != 0 {
                            println!("len should be zero not {} {:?}", set.len(), &set);
                            return false;
                        }
                        return true;
                    },
                }
                if set.len() != refset.len() { return false; }
                for i in 0..2550 {
                    if set.contains(&i) != refset.contains(&i) { return false; }
                }
                for x in set.iter() {
                    if !refset.contains(&x) { return false; }
                }
                for x in refset.iter() {
                    if !set.contains(x) { return false; }
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_bigint(steps: Vec<Result<(u64,u8),(u64,u8)>>) -> bool {
            let mut steps = steps;
            let mut set = U64Set::with_capacity(1);
            let mut refset = HashSet::<u64>::new();
            loop {
                match steps.pop() {
                    Some(Ok((v,shift))) => {
                        let v = v << (shift & 31);
                        set.insert(v); refset.insert(v);
                    },
                    Some(Err((v,shift))) => {
                        let v = v << (shift & 31);
                        set.remove(&v); refset.remove(&v);
                    },
                    None => return true,
                }
                if set.len() != refset.len() { return false; }
                for i in 0..2550 {
                    if set.contains(&i).is_some() != refset.contains(&i) {
                        println!("refset: {:?}", &refset);
                        println!("set: {:?}", &set);
                        for x in set.iter() {
                            print!(" {}", x);
                        }
                        println!();
                        assert_eq!(set.contains(&i).is_some(), refset.contains(&i));
                        return false;
                    }
                }
            }
        }
    }

    #[test]
    fn specific_bigint() {
        let mut steps: Vec<Result<(u64,u8),(u64,u8)>> =
            vec![Ok((84, 30)), Ok((0, 0)), Ok((0, 0)), Ok((1, 0)),
                 Ok((1, 1)), Ok((1, 2)), Ok((2, 15))];
        let mut set = U64Set::with_capacity(1);
        let mut refset = HashSet::<u64>::new();
        loop {
            match steps.pop() {
                Some(Ok((v,shift))) => {
                    let v = v << (shift & 31);
                    println!(" adding {}", v);
                    println!("compare {}", u32::invalid());
                    set.insert(v); refset.insert(v);
                },
                Some(Err((v,shift))) => {
                    let v = v << (shift & 31);
                    println!("removing {}", v);
                    set.remove(&v); refset.remove(&v);
                },
                None => return,
            }
            if true || set.len() != refset.len() {
                println!("refset: {:?}", &refset);
                println!("set: {:?}", &set);
                for x in set.iter() {
                    print!(" {}", x);
                }
                println!();
            }
            assert_eq!(set.len(), refset.len());
            for i in 0..2550 {
                if set.contains(&i).is_some() != refset.contains(&i) {
                    println!("refset: {:?}", &refset);
                    println!("set: {:?}", &set);
                    for x in set.iter() {
                        print!(" {}", x);
                    }
                    println!();
                    assert_eq!(set.contains(&i).is_some(), refset.contains(&i));
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn fits64_i8(num: i8) -> bool {
            num.test_fits64()
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn fits64_u8(num: u8) -> bool {
            num.test_fits64()
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn fits64_i16(num: i16) -> bool {
            num.test_fits64()
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn fits64_u16(num: u16) -> bool {
            num.test_fits64()
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn fits32_i32(num: i32) -> bool {
            num.test_fits64()
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn fits32_u32(num: u32) -> bool {
            num.test_fits64()
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn fits64_i64(num: i64) -> bool {
            num.test_fits64()
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn fits64_u64(num: u64) -> bool {
            num.test_fits64()
        }
    }
}

fn search<T: HasInvalid>(v: &[T], elem: T, invalid: T) -> SearchResult {
    let h = elem.hash_usize();
    let mut dist = 0;
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

fn search_from<T: HasInvalid>(v: &[T], i_start: usize, elem: T, invalid: T) -> SearchResult {
    let h = elem.hash_usize();
    let mask = v.len() - 1;
    let mut dist = i_start.wrapping_sub(h) & mask;
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

fn steal<T: HasInvalid>(v: &mut [T], mut i: usize, mut elem: T, invalid: T) {
    loop {
        match search_from(v, i, elem, invalid) {
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
    /// verify that the conversion is lossless
    fn test_fits64(self) -> bool {
        // println!("\ntest_fits64 {:?}", &self);
        let x = self.to_u64();
        let y = unsafe { Self::from_u64(x).to_u64() };
        // println!("x: {}, and y: {}", x, y);
        // assert_eq!(x, y);
        x == y
    }
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
/// type is very space-efficient in storing small integers, while not
/// being bad at storing large integers (i.e. about half the size of a
/// large `fnv::HashSet`, for small sets of large integers about five
/// times smaller than `fnv::HashSet`.  For small numbers, `Set64` is
/// even more compact.
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
/// for x in &a {
///     assert!("Hello world".contains(x));
/// }
/// ```
///
/// # Storage details
///
/// A `Set64` is somewhat complicated in its data format, because it
/// has 8 possibilities, and which of those formats it takes depends
/// on the largest value stored and how many values are stored.  Note
/// that the size of value is defined in terms of the `u64` that the
/// element can be converted into.
///
/// 1. If there are 22 or less items that are less than 255, then the
///    set is stored as an array of `u8` with a single byte
///    indicating how many elements there are.  Search and addition is
///    linear in the number of elements, and way faster than `O(1)`
///    operations would be.  No heap storage is used.
/// 1. If there are 11 or less items that are less than 2^16-1, then the
///    set is stored as an array of `u16` with a single byte
///    indicating how many elements there are.  Search and addition is
///    linear in the number of elements, and way faster than `O(1)`
///    operations would be.  No heap storage is used.
/// 1. If there are 5 or less items that are less than 2^32-1, then the
///    set is stored as an array of `u32` with a single byte
///    indicating how many elements there are.  Search and addition is
///    linear in the number of elements, and way faster than `O(1)`
///    operations would be.  No heap storage is used.
/// 1. If there are 2 or less items that are less than 2^64-1, then
///    the set is stored as an array of `u64` with a single byte
///    indicating how many elements there are.  Search and addition is
///    linear in the number of elements, and way faster than `O(1)`
///    operations would be.  No heap storage is used.
/// 1. If there are many items that are less than 255, then the set is
///    stored on the heap as a Robin Hood hash set of `u8` values.
/// 1. If there are many items that are less than 2^16-1, then the set
///    is stored on the heap as a Robin Hood hash set of `u16` values.
/// 1. If there are many items that are less than 2^32-1, then the set
///    is stored on the heap as a Robin Hood hash set of `u32` values.
/// 1. If there are many large items, then the set is stored on the
///    heap as a Robin Hood hash set of `u64` values.
#[derive(Debug, Clone)]
pub struct Set64<T: Fits64>(U64Set, PhantomData<T>);

impl<T: Fits64> Set64<T> {
    /// Creates an empty set..
    pub fn default() -> Self {
        Set64(U64Set::with_capacity(0), PhantomData)
    }
    /// Creates an empty set..
    pub fn new() -> Self {
        Set64(U64Set::with_capacity(0), PhantomData)
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Set64(U64Set::with_capacity(cap), PhantomData)
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_max_and_capacity(max: T, cap: usize) -> Self {
        Set64(U64Set::with_max_and_capacity(max.to_u64(), cap), PhantomData)
    }
    /// Reserves capacity for at least `additional` more elements to be
    /// inserted in the set. The collection may reserve more space
    /// to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
    /// Reserves capacity for at least `additional` more elements to
    /// be inserted in the set, with maximum value of `max`. The
    /// collection may reserve more space to avoid frequent
    /// reallocations.
    pub fn reserve_with_max(&mut self, max: T, additional: usize) {
        self.0.reserve_with_max(max.to_u64(), additional);
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
        self.0.contains(&x).is_some()
    }
    /// Removes an element, and returns true if that element was present.
    pub fn remove(&mut self, value: &T) -> bool {
        let x = value.clone().to_u64();
        self.0.remove(&x)
    }
    /// Iterate
    pub fn iter(&self) -> Iter64<T> {
        Iter64( self.0.iter(), PhantomData )
    }
    /// Drain
    pub fn drain(&mut self) -> Drain64<T> {
        Drain64( self.0.drain(), PhantomData )
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

/// A drainer.
pub struct Drain64<T: Fits64>( Drain, PhantomData<T> );
impl<T: Fits64> Iterator for Drain64<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.next().map(|x| unsafe { T::from_u64(x) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// An iterator.
pub struct Iter64<'a, T: Fits64>( Iter<'a>, PhantomData<T> );

impl<'a, T: Fits64> Iterator for Iter64<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.next().map(|x| unsafe { T::from_u64(x) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T: Fits64> IntoIterator for &'a Set64<T> {
    type Item = T;
    type IntoIter = Iter64<'a, T>;

    fn into_iter(self) -> Iter64<'a, T> {
        self.iter()
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
    /// for x in &set {
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
    /// for x in &a {
    ///     assert!(expected.contains(&x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (sz,_) = iter.size_hint();
        self.reserve(sz);
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
    /// for x in &set {
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

#[cfg(target_pointer_width = "64")]
const MAP_NUM_U8: usize = 23;
#[cfg(target_pointer_width = "64")]
const MAP_NUM_U16: usize = 15;
#[cfg(target_pointer_width = "64")]
const MAP_NUM_U32: usize = 9;
#[cfg(target_pointer_width = "64")]
const MAP_NUM_U64: usize = 4;

#[cfg(not(target_pointer_width = "64"))]
const MAP_NUM_U8: usize = 13;
#[cfg(not(target_pointer_width = "64"))]
const MAP_NUM_U16: usize = 8;
#[cfg(not(target_pointer_width = "64"))]
const MAP_NUM_U32: usize = 5;
#[cfg(not(target_pointer_width = "64"))]
const MAP_NUM_U64: usize = 2;

/// A map of u64 elements to small integers
#[derive(Debug, Clone)]
enum U64Map {
    Su8 {
        sz: u8,
        keys: [u8; MAP_NUM_U8],
        vals: [u8; MAP_NUM_U8],
    },
    Vu8 {
        sz: u8,
        keys: Box<[u8]>,
        vals: Box<[u8]>,
    },
    Su16 {
        sz: u8,
        keys: [u16; MAP_NUM_U16],
        vals: [u8; MAP_NUM_U16],
    },
    Vu16 {
        sz: u16,
        keys: Box<[u16]>,
        vals: Box<[u16]>,
    },
    Su32 {
        sz: u8,
        keys: [u32; MAP_NUM_U32],
        vals: [u8; MAP_NUM_U32],
    },
    Vu32 {
        sz: u32,
        keys: Box<[u32]>,
        vals: Box<[u32]>,
    },
    Su64 {
        sz: u64,
        keys: [u64; MAP_NUM_U64],
        vals: [u8; MAP_NUM_U64],
    },
    Vu64 {
        sz: u64,
        keys: Box<[u64]>,
        vals: Box<[u64]>,
    },
}

impl U64Map {
    fn with_capacity(cap: usize) -> U64Map {
        let nextcap = capacity_to_rawcapacity(cap);
        if cap <= MAP_NUM_U8 {
            U64Map::Su8 { sz: 0, keys: [0; MAP_NUM_U8], vals: [0; MAP_NUM_U8] }
        } else if cap < u8::invalid() as usize {
            U64Map::Vu8 {
                sz: 0,
                keys: vec![u8::invalid(); nextcap].into_boxed_slice(),
                vals: vec![u8::invalid(); nextcap].into_boxed_slice(),
            }
        } else if cap < u16::invalid() as usize {
            U64Map::Vu16 {
                sz: 0,
                keys: vec![u16::invalid(); nextcap].into_boxed_slice(),
                vals: vec![u16::invalid(); nextcap].into_boxed_slice(),
            }
        } else if cap < u32::invalid() as usize {
            U64Map::Vu32 {
                sz: 0,
                keys: vec![u32::invalid(); nextcap].into_boxed_slice(),
                vals: vec![u32::invalid(); nextcap].into_boxed_slice(),
            }
        } else {
            U64Map::Vu64 {
                sz: 0,
                keys: vec![u64::invalid(); nextcap+1].into_boxed_slice(),
                vals: vec![u64::invalid(); nextcap].into_boxed_slice(),
            }
        }
    }
    fn with_maxes_cap(max_k: u64, max_v: u64, cap: usize) -> U64Map {
        let max_k = if max_k > max_v { max_k } else { max_v };
        let nextcap = capacity_to_rawcapacity(cap);
        if max_k < u8::invalid() as u64 {
            if cap <= NUM_U8 && max_v < 256 {
                U64Map::Su8 { sz: 0, keys: [0; MAP_NUM_U8], vals: [0; MAP_NUM_U8] }
            } else {
                U64Map::Vu8 {
                    sz: 0,
                    keys: vec![u8::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![u8::invalid(); nextcap].into_boxed_slice(),
                }
            }
        } else if max_k < u16::invalid() as u64 {
            if cap <= NUM_U16 && max_v < 256 {
                U64Map::Su16 {
                    sz: 0,
                    keys: [u16::invalid(); MAP_NUM_U16],
                    vals: [0; MAP_NUM_U16]
                }
            } else {
                U64Map::Vu16 {
                    sz: 0,
                    keys: vec![u16::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![u16::invalid(); nextcap].into_boxed_slice(),
                }
            }
        } else if max_k < u32::invalid() as u64 {
            if cap <= NUM_U32 && max_v < 256 {
                U64Map::Su32 {
                    sz: 0,
                    keys: [u32::invalid(); MAP_NUM_U32],
                    vals: [0; MAP_NUM_U32]
                }
            } else {
                U64Map::Vu32 {
                    sz: 0,
                    keys: vec![u32::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![u32::invalid(); nextcap].into_boxed_slice(),
                }
            }
        } else {
            if cap <= NUM_U64 && max_v < 256 {
                U64Map::Su64 {
                    sz: 0,
                    keys: [0; MAP_NUM_U64],
                    vals: [0; MAP_NUM_U64]
                }
            } else {
                U64Map::Vu64 {
                    sz: 0,
                    keys: vec![u64::invalid(); nextcap+1].into_boxed_slice(),
                    vals: vec![137; nextcap].into_boxed_slice(),
                }
            }
        }
    }
    fn insert_unchecked(&mut self, k: u64, v: u64) -> Option<u64> {
        match self {
            &mut U64Map::Su8 { ref mut sz, ref mut keys, ref mut vals } => {
                let k = k as u8;
                for i in 0..*sz as usize {
                    if keys[i] == k {
                        let oldv = vals[i];
                        vals[i] = v as u8;
                        return Some(oldv as u64);
                    }
                }
                keys[*sz as usize] = k;
                vals[*sz as usize] = v as u8;
                *sz += 1;
                None
            },
            &mut U64Map::Su16 { ref mut sz, ref mut keys, ref mut vals } => {
                let k = k as u16;
                for i in 0..*sz as usize {
                    if keys[i] == k {
                        let oldv = vals[i];
                        vals[i] = v as u8;
                        return Some(oldv as u64);
                    }
                }
                keys[*sz as usize] = k;
                vals[*sz as usize] = v as u8;
                *sz += 1;
                None
            },
            &mut U64Map::Su32 { ref mut sz, ref mut keys, ref mut vals } => {
                let k = k as u32;
                for i in 0..*sz as usize {
                    if keys[i] == k {
                        let oldv = vals[i];
                        vals[i] = v as u8;
                        return Some(oldv as u64);
                    }
                }
                keys[*sz as usize] = k;
                vals[*sz as usize] = v as u8;
                *sz += 1;
                None
            },
            &mut U64Map::Su64 { ref mut sz, ref mut keys, ref mut vals } => {
                let k = k as u64;
                for i in 0..*sz as usize {
                    if keys[i] == k {
                        let oldv = vals[i];
                        vals[i] = v as u8;
                        return Some(oldv as u64);
                    }
                }
                keys[*sz as usize] = k;
                vals[*sz as usize] = v as u8;
                *sz += 1;
                None
            },
            &mut U64Map::Vu8 { ref mut sz, ref mut keys, ref mut vals } => {
                let mut k = k as u8;
                let mut v = v as u8;
                match search(keys, k, u8::invalid()) {
                    SearchResult::Present(i) => {
                        let oldv = vals[i];
                        vals[i] = v;
                        Some(oldv as u64)
                    },
                    SearchResult::Empty(i) => {
                        keys[i] = k;
                        vals[i] = v;
                        *sz += 1;
                        None
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v, u8::invalid());
                        None
                    },
                }
            },
            &mut U64Map::Vu16 { ref mut sz, ref mut keys, ref mut vals } => {
                let mut k = k as u16;
                let mut v = v as u16;
                match search(keys, k, u16::invalid()) {
                    SearchResult::Present(i) => {
                        let oldv = vals[i];
                        vals[i] = v;
                        Some(oldv as u64)
                    },
                    SearchResult::Empty(i) => {
                        keys[i] = k;
                        vals[i] = v;
                        *sz += 1;
                        None
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v, u16::invalid());
                        None
                    },
                }
            },
            &mut U64Map::Vu32 { ref mut sz, ref mut keys, ref mut vals } => {
                let mut k = k as u32;
                let mut v = v as u32;
                match search(keys, k, u32::invalid()) {
                    SearchResult::Present(i) => {
                        let oldv = vals[i];
                        vals[i] = v;
                        Some(oldv as u64)
                    },
                    SearchResult::Empty(i) => {
                        keys[i] = k;
                        vals[i] = v;
                        *sz += 1;
                        None
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v, u32::invalid());
                        None
                    },
                }
            },
            &mut U64Map::Vu64 { .. } => {
                let mut k = k as u64;
                let mut v = v as u64;
                let mut invalid = 0;
                let mut old_invalid = 0;
                if let U64Map::Vu64 { ref keys, .. } = self {
                    let klen = keys.len();
                    invalid = keys[klen-1];
                    old_invalid = invalid;
                    if k == invalid {
                        // Need to change our invalid
                        invalid = invalid.wrapping_sub(1);
                        while self.get(invalid).is_some() {
                            invalid = invalid.wrapping_sub(1);
                        }
                    }
                }
                if let U64Map::Vu64 { ref mut sz, ref mut keys, ref mut vals } = self {
                    let klen = keys.len();
                    if old_invalid != invalid {
                        for x in keys.iter_mut() {
                            if *x == old_invalid {
                                *x = invalid;
                            }
                        }
                    }
                    let keys = &mut keys[..klen-1];
                    match search(keys, k, invalid) {
                        SearchResult::Present(i) => {
                            let oldv = vals[i];
                            vals[i] = v;
                            Some(oldv as u64)
                        },
                        SearchResult::Empty(i) => {
                            keys[i] = k;
                            vals[i] = v;
                            *sz += 1;
                            None
                        },
                        SearchResult::Richer(i) => {
                            *sz += 1;
                            std::mem::swap(&mut keys[i], &mut k);
                            std::mem::swap(&mut vals[i], &mut v);
                            mapsteal(keys, vals, i, k, v, invalid);
                            None
                        },
                    }
                } else {
                    unreachable!()
                }
            },
        }
    }
    /// Reserves capacity for at least `additional` more elements to
    /// be inserted in the set, with maximum value of `max`. The
    /// collection may reserve more space to avoid frequent
    /// reallocations.
    fn reserve_with_maxes(&mut self, max_k: u64, max_v: u64, additional: usize) {
        let max_k = if max_k > max_v { max_k } else { max_v };
        let mut newself: Option<U64Map> = None;
        match *self {
            U64Map::Su8 { sz, keys: k, vals: v } if max_k >= u8::invalid() as u64 => {
                let mut n = Self::with_maxes_cap(max_k, max_v, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(k[i] as u64, v[i] as u64);
                }
                *self = n;
            },
            U64Map::Su8 { sz, keys, vals } if sz as usize + additional > MAP_NUM_U8 => {
                let nextcap = capacity_to_rawcapacity(sz as usize + additional);
                *self = U64Map::Vu8 {
                    sz: 0,
                    keys: vec![u8::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![0; nextcap].into_boxed_slice(),
                };
                for i in 0..sz as usize {
                    self.insert_unchecked(keys[i] as u64, vals[i] as u64);
                }
            },
            U64Map::Su8 {sz:_,keys:_,vals:_} => (),
            U64Map::Su16 { sz, keys: k, vals: v } if max_k >= u16::invalid() as u64 => {
                let mut n = Self::with_maxes_cap(max_k, max_v, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(k[i] as u64, v[i] as u64);
                }
                Some(n);
            },
            U64Map::Su16 { sz, keys, vals } if sz as usize + additional > MAP_NUM_U16 => {
                let nextcap = capacity_to_rawcapacity(sz as usize + additional);
                *self = U64Map::Vu16 {
                    sz: 0,
                    keys: vec![u16::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![0; nextcap].into_boxed_slice(),
                };
                for i in 0..sz as usize {
                    self.insert_unchecked(keys[i] as u64, vals[i] as u64);
                }
            },
            U64Map::Su16 {sz:_,keys:_,vals:_} => (),
            U64Map::Su32 { sz, keys: k, vals: v } if max_k >= u32::invalid() as u64 => {
                let mut n = Self::with_maxes_cap(max_k, max_v, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(k[i] as u64, v[i] as u64);
                }
                *self = n;
            },
            U64Map::Su32 { sz, keys, vals } if sz as usize + additional > MAP_NUM_U32 => {
                let nextcap = capacity_to_rawcapacity(sz as usize + additional);
                *self = U64Map::Vu32 {
                    sz: 0,
                    keys: vec![u32::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![0; nextcap].into_boxed_slice(),
                };
                for i in 0..sz as usize {
                    self.insert_unchecked(keys[i] as u64, vals[i] as u64);
                }
            },
            U64Map::Su32 {sz:_,keys:_,vals:_} => (),
            U64Map::Su64 { sz, keys, vals } if sz as usize + additional > MAP_NUM_U64 => {
                let nextcap = capacity_to_rawcapacity(sz as usize + additional);
                *self = U64Map::Vu64 {
                    sz: 0,
                    keys: vec![u64::invalid(); nextcap+1].into_boxed_slice(),
                    vals: vec![0; nextcap].into_boxed_slice(),
                };
                for i in 0..sz as usize {
                    self.insert_unchecked(keys[i] as u64, vals[i] as u64);
                }
            },
            U64Map::Su64 {sz:_,keys:_,vals:_} => (),
            U64Map::Vu8 {sz,ref keys,ref vals} if max_k >= u8::invalid() as u64 => {
                let mut n = Self::with_maxes_cap(max_k, max_v, sz as usize + additional);
                for i in 0..keys.len() {
                    if keys[i] != u8::invalid() {
                        n.insert_unchecked(keys[i] as u64, vals[i] as u64);
                    }
                }
                newself = Some(n);
            },
            U64Map::Vu8 {sz,ref mut keys,ref mut vals} if sz as usize + additional > keys.len()*10/11 => {
                let newcap = capacity_to_rawcapacity(sz as usize+additional);
                let oldkeys = std::mem::replace(keys,
                                                vec![u8::invalid(); newcap].into_boxed_slice());
                let oldvals = std::mem::replace(vals,
                                                vec![0; newcap].into_boxed_slice());
                for (&k, &v) in oldkeys.iter().zip(oldvals.iter()) {
                    if k != u8::invalid() {
                        let mut key = k;
                        let mut value = v;
                        match search(keys, key, u8::invalid()) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => {
                                keys[i] = key;
                                vals[i] = value;
                            },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut keys[i], &mut key);
                                std::mem::swap(&mut vals[i], &mut value);
                                mapsteal(keys, vals, i, key, value, u8::invalid());
                            },
                        }
                    }
                }
            },
            U64Map::Vu8 {sz:_,keys:_,vals:_} => (),
            U64Map::Vu16 {sz,ref keys,ref vals} if max_k >= u16::invalid() as u64 => {
                let mut n = Self::with_maxes_cap(max_k, max_v, sz as usize + additional);
                for i in 0..keys.len() {
                    if keys[i] != u16::invalid() {
                        n.insert_unchecked(keys[i] as u64, vals[i] as u64);
                    }
                }
                newself = Some(n);
            },
            U64Map::Vu16 {sz,ref mut keys,ref mut vals} if sz as usize + additional > keys.len()*10/11 => {
                let newcap = capacity_to_rawcapacity(sz as usize+additional);
                let oldkeys = std::mem::replace(keys,
                                                vec![u16::invalid(); newcap].into_boxed_slice());
                let oldvals = std::mem::replace(vals,
                                                vec![0; newcap].into_boxed_slice());
                for (&k, &v) in oldkeys.iter().zip(oldvals.iter()) {
                    if k != u16::invalid() {
                        let mut key = k;
                        let mut value = v;
                        match search(keys, key, u16::invalid()) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => {
                                keys[i] = key;
                                vals[i] = value;
                            },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut keys[i], &mut key);
                                std::mem::swap(&mut vals[i], &mut value);
                                mapsteal(keys, vals, i, key, value, u16::invalid());
                            },
                        }
                    }
                }
            },
            U64Map::Vu16 {sz:_,keys:_,vals:_} => (),
            U64Map::Vu32 {sz,ref keys,ref vals} if max_k >= u32::invalid() as u64 => {
                let mut n = Self::with_maxes_cap(max_k, max_v, sz as usize + additional);
                for i in 0..keys.len() {
                    if keys[i] != u32::invalid() {
                        n.insert_unchecked(keys[i] as u64, vals[i] as u64);
                    }
                }
                newself = Some(n);
            },
            U64Map::Vu32 {sz,ref mut keys,ref mut vals} if sz as usize + additional > keys.len()*10/11 => {
                let newcap = capacity_to_rawcapacity(sz as usize+additional);
                let oldkeys = std::mem::replace(keys,
                                                vec![u32::invalid(); newcap].into_boxed_slice());
                let oldvals = std::mem::replace(vals,
                                                vec![0; newcap].into_boxed_slice());
                for (&k, &v) in oldkeys.iter().zip(oldvals.iter()) {
                    if k != u32::invalid() {
                        let mut key = k;
                        let mut value = v;
                        match search(keys, key, u32::invalid()) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => {
                                keys[i] = key;
                                vals[i] = value;
                            },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut keys[i], &mut key);
                                std::mem::swap(&mut vals[i], &mut value);
                                mapsteal(keys, vals, i, key, value, u32::invalid());
                            },
                        }
                    }
                }
            },
            U64Map::Vu32 {sz:_,keys:_,vals:_} => (),
            U64Map::Vu64 {sz,ref keys,ref vals} if max_k >= u64::invalid() as u64 => {
                let mut n = Self::with_maxes_cap(max_k, max_v, sz as usize + additional);
                let klen = keys.len();
                let invalid = keys[klen-1];
                let keys = &keys[..klen-1];
                for i in 0..keys.len() {
                    if keys[i] != invalid {
                        n.insert_unchecked(keys[i] as u64, vals[i] as u64);
                    }
                }
                newself = Some(n);
            },
            U64Map::Vu64 {sz,ref mut keys,ref mut vals} if sz as usize + additional > keys.len()*10/11 => {
                let newcap = capacity_to_rawcapacity(sz as usize+additional);
                let klen = keys.len();
                let invalid = keys[klen-1];
                let oldkeys = std::mem::replace(keys,
                                                vec![invalid; newcap+1].into_boxed_slice());
                let oldvals = std::mem::replace(vals,
                                                vec![0; newcap].into_boxed_slice());
                let oldkeys = &oldkeys[..klen-1];
                let keys = &mut keys[..newcap];
                for (&k, &v) in oldkeys.iter().zip(oldvals.iter()) {
                    if k != invalid {
                        let mut key = k;
                        let mut value = v;
                        match search(keys, key, invalid) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => {
                                keys[i] = key;
                                vals[i] = value;
                            },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut keys[i], &mut key);
                                std::mem::swap(&mut vals[i], &mut value);
                                mapsteal(keys, vals, i, key, value, invalid);
                            },
                        }
                    }
                }
            },
            U64Map::Vu64 {sz:_,keys:_,vals:_} => (),
        }
        if let Some(s) = newself {
            *self = s;
        }
    }
    fn insert(&mut self, k: u64, v: u64) -> Option<u64> {
        println!("reserving with maxes {} and {}", k, v);
        self.reserve_with_maxes(k,v,1);
        println!("   result is {:?}", self);
        self.insert_unchecked(k,v)
    }
    fn get(&self, k: u64) -> Option<u64> {
        match *self {
            U64Map::Su8 { sz, ref keys, ref vals } => {
                if k >= u8::invalid() as u64 {
                    return None;
                }
                let k = k as u8;
                for i in 0 .. sz as usize {
                    if keys[i] == k {
                        return Some(vals[i] as u64);
                    }
                }
                None
            },
            U64Map::Su16 { sz, ref keys, ref vals } => {
                if k >= u16::invalid() as u64 {
                    return None;
                }
                let k = k as u16;
                for i in 0 .. sz as usize {
                    if keys[i] == k {
                        return Some(vals[i] as u64);
                    }
                }
                None
            },
            U64Map::Su32 { sz, ref keys, ref vals } => {
                if k >= u32::invalid() as u64 {
                    return None;
                }
                let k = k as u32;
                for i in 0 .. sz as usize {
                    if keys[i] == k {
                        return Some(vals[i] as u64);
                    }
                }
                None
            },
            U64Map::Su64 { sz, ref keys, ref vals } => {
                let k = k as u64;
                for i in 0 .. sz as usize {
                    if keys[i] == k {
                        return Some(vals[i] as u64);
                    }
                }
                None
            },
            U64Map::Vu8 {sz:_, ref keys, ref vals } => {
                if k >= u8::invalid() as u64 {
                    return None;
                }
                let k = k as u8;
                match search(keys, k, u8::invalid()) {
                    SearchResult::Present(i) => Some(vals[i] as u64),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            U64Map::Vu16 {sz:_, ref keys, ref vals } => {
                if k >= u16::invalid() as u64 {
                    return None;
                }
                let k = k as u16;
                match search(keys, k, u16::invalid()) {
                    SearchResult::Present(i) => Some(vals[i] as u64),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            U64Map::Vu32 {sz:_, ref keys, ref vals } => {
                if k >= u32::invalid() as u64 {
                    return None;
                }
                let k = k as u32;
                match search(keys, k, u32::invalid()) {
                    SearchResult::Present(i) => Some(vals[i] as u64),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            U64Map::Vu64 {sz:_, ref keys, ref vals } => {
                let klen = keys.len();
                let invalid = keys[klen-1];
                let keys = &keys[..klen-1];
                if k == invalid as u64 {
                    return None;
                }
                let k = k as u64;
                match search(keys, k, invalid) {
                    SearchResult::Present(i) => Some(vals[i] as u64),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
        }
    }
    fn remove(&mut self, k: u64) -> Option<u64> {
        match *self {
            U64Map::Su8 { ref mut sz, ref mut keys, ref mut vals } => {
                if k >= u8::invalid() as u64 {
                    return None;
                }
                let k = k as u8;
                let mut i = None;
                for (j, &x) in keys.iter().enumerate().take(*sz as usize) {
                    if x == k {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    let oldv = vals[i];
                    vals[i] = vals[*sz as usize -1];
                    keys[i] = keys[*sz as usize -1];
                    *sz -= 1;
                    Some(oldv as u64)
                } else {
                    None
                };
            },
            U64Map::Su16 { ref mut sz, ref mut keys, ref mut vals } => {
                if k >= u16::invalid() as u64 {
                    return None;
                }
                let k = k as u16;
                let mut i = None;
                for (j, &x) in keys.iter().enumerate().take(*sz as usize) {
                    if x == k {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    let oldv = vals[i];
                    vals[i] = vals[*sz as usize -1];
                    keys[i] = keys[*sz as usize -1];
                    *sz -= 1;
                    Some(oldv as u64)
                } else {
                    None
                };
            },
            U64Map::Su32 { ref mut sz, ref mut keys, ref mut vals } => {
                if k >= u32::invalid() as u64 {
                    return None;
                }
                let k = k as u32;
                let mut i = None;
                for (j, &x) in keys.iter().enumerate().take(*sz as usize) {
                    if x == k {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    let oldv = vals[i];
                    vals[i] = vals[*sz as usize -1];
                    keys[i] = keys[*sz as usize -1];
                    *sz -= 1;
                    Some(oldv as u64)
                } else {
                    None
                };
            },
            U64Map::Su64 { ref mut sz, ref mut keys, ref mut vals } => {
                let k = k as u64;
                let mut i = None;
                for (j, &x) in keys.iter().enumerate().take(*sz as usize) {
                    if x == k {
                        i = Some(j);
                        break;
                    }
                }
                return if let Some(i) = i {
                    let oldv = vals[i];
                    vals[i] = vals[*sz as usize -1];
                    keys[i] = keys[*sz as usize -1];
                    *sz -= 1;
                    Some(oldv as u64)
                } else {
                    None
                };
            },
            U64Map::Vu8 { ref mut sz, ref mut keys, ref mut vals } => {
                if k >= u8::invalid() as u64 {
                    return None;
                }
                let k = k as u8;
                match search(keys, k, u8::invalid()) {
                    SearchResult::Present(mut i) => {
                        let oldval = vals[i];
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = u8::invalid();
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(oldval as u64);
                            }
                            keys[i] = keys[iplus1];
                            vals[i] = vals[iplus1];
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            U64Map::Vu16 { ref mut sz, ref mut keys, ref mut vals } => {
                if k >= u16::invalid() as u64 {
                    return None;
                }
                let k = k as u16;
                match search(keys, k, u16::invalid()) {
                    SearchResult::Present(mut i) => {
                        let oldval = vals[i];
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = u16::invalid();
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(oldval as u64);
                            }
                            keys[i] = keys[iplus1];
                            vals[i] = vals[iplus1];
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            U64Map::Vu32 { ref mut sz, ref mut keys, ref mut vals } => {
                if k >= u32::invalid() as u64 {
                    return None;
                }
                let k = k as u32;
                match search(keys, k, u32::invalid()) {
                    SearchResult::Present(mut i) => {
                        let oldval = vals[i];
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = u32::invalid();
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(oldval as u64);
                            }
                            keys[i] = keys[iplus1];
                            vals[i] = vals[iplus1];
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            U64Map::Vu64 { ref mut sz, ref mut keys, ref mut vals } => {
                let klen = keys.len();
                let invalid = keys[klen-1];
                let keys = &mut keys[..klen-1];
                if k == invalid as u64 {
                    return None;
                }
                let k = k as u64;
                match search(keys, k, invalid) {
                    SearchResult::Present(mut i) => {
                        let oldval = vals[i];
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = invalid;
                        loop {
                            let iplus1 = (i+1) & mask;
                            if keys[iplus1] == invalid ||
                                (keys[iplus1].hash_usize().wrapping_sub(iplus1) & mask) == 0
                            {
                                keys[i] = invalid;
                                return Some(oldval as u64);
                            }
                            keys[i] = keys[iplus1];
                            vals[i] = vals[iplus1];
                            i = iplus1;
                        }
                    },
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
        }
    }
    fn len(&self) -> usize {
        match *self {
            U64Map::Su8 {sz, keys:_, vals:_ } => sz as usize,
            U64Map::Su16 {sz, keys:_, vals:_ } => sz as usize,
            U64Map::Su32 {sz, keys:_, vals:_ } => sz as usize,
            U64Map::Su64 {sz, keys:_, vals:_ } => sz as usize,
            U64Map::Vu8 {sz, keys:_, vals:_ } => sz as usize,
            U64Map::Vu16 {sz, keys:_, vals:_ } => sz as usize,
            U64Map::Vu32 {sz, keys:_, vals:_ } => sz as usize,
            U64Map::Vu64 {sz, keys:_, vals:_ } => sz as usize,
        }
    }
    /// Iterate over tuples
    fn iter(&self) -> U64MapIter {
        U64MapIter { m: self, which: 0, nleft: self.len() }
    }
}

impl PartialEq for U64Map {
    fn eq(&self, other: &U64Map) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (k, v) in other.iter() {
            if self.get(k) != Some(v) {
                return false;
            }
        }
        true
    }
}
impl Eq for U64Map {}

/// Iterator for u64map
pub struct U64MapIter<'a> {
    m: &'a U64Map,
    which: usize,
    nleft: usize,
}

impl<'a> Iterator for U64MapIter<'a> {
    type Item = (u64,u64);
    fn next(&mut self) -> Option<(u64,u64)> {
        if self.nleft == 0 {
            return None;
        }
        self.nleft -= 1;
        match self.m {
            &U64Map::Su8 { sz:_, ref keys, ref vals } => {
                while keys[self.which] == u8::invalid() {
                    self.which += 1;
                }
                self.which += 1;
                Some((keys[self.which-1] as u64, vals[self.which-1] as u64))
            },
            &U64Map::Su16 { sz:_, ref keys, ref vals } => {
                while keys[self.which] == u16::invalid() {
                    self.which += 1;
                }
                self.which += 1;
                Some((keys[self.which-1] as u64, vals[self.which-1] as u64))
            },
            &U64Map::Su32 { sz:_, ref keys, ref vals } => {
                while keys[self.which] == u32::invalid() {
                    self.which += 1;
                }
                self.which += 1;
                Some((keys[self.which-1] as u64, vals[self.which-1] as u64))
            },
            &U64Map::Su64 { sz:_, ref keys, ref vals } => {
                self.which += 1;
                Some((keys[self.which-1] as u64, vals[self.which-1] as u64))
            },
            &U64Map::Vu8 { sz:_, ref keys, ref vals } => {
                while keys[self.which] == u8::invalid() {
                    self.which += 1;
                }
                self.which += 1;
                Some((keys[self.which-1] as u64, vals[self.which-1] as u64))
            },
            &U64Map::Vu16 { sz:_, ref keys, ref vals } => {
                while keys[self.which] == u16::invalid() {
                    self.which += 1;
                }
                self.which += 1;
                Some((keys[self.which-1] as u64, vals[self.which-1] as u64))
            },
            &U64Map::Vu32 { sz:_, ref keys, ref vals } => {
                while keys[self.which] == u32::invalid() {
                    self.which += 1;
                }
                self.which += 1;
                Some((keys[self.which-1] as u64, vals[self.which-1] as u64))
            },
            &U64Map::Vu64 { sz:_, ref keys, ref vals } => {
                let klen = keys.len();
                let invalid = keys[klen-1];
                let keys = &keys[..klen-1];
                while keys[self.which] == invalid {
                    self.which += 1;
                }
                self.which += 1;
                Some((keys[self.which-1] as u64, vals[self.which-1] as u64))
            },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.nleft, Some(self.nleft))
    }
}

fn mapsteal<K: HasInvalid, V>(k: &mut [K], v: &mut [V], mut i: usize, mut elem: K, mut val: V, invalid: K) {
    loop {
        match search_from(k, i, elem, invalid) {
            SearchResult::Present(i) => {
                v[i] = val;
                return;
            },
            SearchResult::Empty(i) => {
                k[i] = elem;
                v[i] = val;
                return;
            },
            SearchResult::Richer(inew) => {
                std::mem::swap(&mut elem, &mut k[inew]);
                std::mem::swap(&mut val, &mut v[inew]);
                i = inew;
            },
        }
    }
}

#[cfg(test)]
mod u64map_tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn size_unwasted() {
        println!("box size: {}", std::mem::size_of::<Box<[u64]>>());
        println!("small size: {}", std::mem::size_of::<U64Map>());
        println!(" hash size: {}", std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<U64Map>() <=
                2*std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<U64Map>() <= 48);
    }

    #[test]
    fn simple() {
        let mut m = U64Map::with_capacity(0);
        m.insert(5,1);
        assert_eq!(m.len(), 1);
        assert_eq!(m.get(0), None);
        assert_eq!(m.get(5), Some(1));
        for i in 6..80 {
            println!("inserting {}", i);
            m.insert(i,i);
            assert_eq!(m.get(5), Some(1));
        }
        for i in 6..80 {
            assert_eq!(m.get(i), Some(i));
        }
        for i in 81..300 {
            assert_eq!(m.get(i), None);
        }
        assert_eq!(m.get(5), Some(1));
        for i in 6..80 {
            println!("removing {}", i);
            assert_eq!(m.get(i), Some(i));
            assert_eq!(m.remove(i), Some(i));
            assert_eq!(m.get(i), None);
        }
        assert_eq!(m.get(0), None);
        assert_eq!(m.get(5), Some(1));
        assert_eq!(m.len(), 1);
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches(steps: Vec<Result<(u64,u64),u64>>) -> bool {
            let mut map = U64Map::with_capacity(0);
            let mut refmap = HashMap::<u64,u64>::new();
            for x in steps {
                match x {
                    Ok((k,v)) => {
                        map.insert(k,v); refmap.insert(k,v);
                    },
                    Err(k) => {
                        map.remove(k); refmap.remove(&k);
                    }
                }
                if map.len() != refmap.len() {
                    return false;
                }
                for i in 0..2550 {
                    if map.get(i) != refmap.get(&i).map(|&v| v) {
                        return false;
                    }
                }
            }
            true
        }
    }

}

/// A map type that can use any key that fits in a `u64` (i.e. that
/// satisfies trait `Fits64`).  This map type is very space-efficient
/// for keys that are small integers, while not being bad at storing
/// large integers.
///
/// **Major caveat** The `Map6464<K,V>` defines an iterator that
/// iterates over `(K, &V)` rather than `(&K, &V)`.  This is a break
/// with standard libray convention, and can be annoying if you are
/// translating code from `HashMap` to `Map6464`.  The motivation for
/// this is several-fold:
///
/// 1. `Map6464` does not store `K` or `V` directly in its data structures
/// (which would waste space), so there is no reference to the data to
/// take.  This does not make it impossible, but does mean we would
/// have to fabricate a `K` and return a reference to it, which is
/// awkward and ugly.
///
/// 2. There is no inefficiency involved in returning `K`, since it is
/// necessarily no larger than a pointer (except on a 32-bit system).
///
/// # Examples
///
/// ```
/// use tinyset::Map6464;
///
/// let mut a: Map6464<char,usize> = Map6464::new();
///
/// a.insert('a', 1);
/// a.insert('b', 2);
/// assert_eq!(a.get(&'a'), Some(1));
/// assert_eq!(a.get(&'b'), Some(2));
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Map6464<K: Fits64, V: Fits64> {
    m: U64Map,
    phk: PhantomData<K>,
    phv: PhantomData<V>,
}

impl<K: Fits64,V: Fits64> Map6464<K,V> {
    /// Creates an empty `Map6464`.
    pub fn new() -> Self {
        Map6464 {
            m: U64Map::with_capacity(1),
            phk: PhantomData,
            phv: PhantomData,
        }
    }
    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.m.len()
    }
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated,
    /// and the old value is returned.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let kk = k.to_u64();
        let vv = v.to_u64();
        self.m.reserve_with_maxes(kk, vv, 1);
        if let Some(i) = self.m.insert_unchecked(kk, vv) {
            Some( unsafe { V::from_u64(i) } )
        } else {
            None
        }
    }
    /// Reserves capacity for at least `additional` more elements to
    /// be inserted in the `Map6464`. The collection may reserve more
    /// space to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        self.m.reserve_with_maxes(0,0,additional);
    }
    /// Removes a key from the map, returning the value at the key if
    /// the key was previously in the map.
    pub fn remove(&mut self, k: &K) -> Option<V> {
        if let Some(i) = self.m.remove(k.to_u64()) {
            return Some( unsafe { V::from_u64(i) } );
        }
        None
    }
    /// Returns the value corresponding to the key.
    pub fn get(&self, k: &K) -> Option<V> {
        if let Some(i) = self.m.get(k.to_u64()) {
            return Some( unsafe { V::from_u64(i) } );
        }
        None
    }
    /// An iterator visiting all key-value pairs in arbitrary
    /// order. The iterator element type is (K, &V).
    pub fn iter(&self) -> Map6464Iter<K,V> {
        Map6464Iter {
            it: self.m.iter(),
            phk: PhantomData,
            phv: PhantomData,
        }
    }
}

/// Iterator for Map6464Iter
pub struct Map6464Iter<'a, K, V> {
    it: U64MapIter<'a>,
    phk: PhantomData<K>,
    phv: PhantomData<V>,
}

impl<'a, K: Fits64+'a, V: Fits64+'a> Iterator for Map6464Iter<'a, K, V> {
    type Item = (K,V);
    fn next(&mut self) -> Option<(K,V)> {
        if let Some((k,i)) = self.it.next() {
            return Some(unsafe { (K::from_u64(k), V::from_u64(i)) });
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

#[cfg(test)]
mod map6464_tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn size_unwasted() {
        println!("box size: {}", std::mem::size_of::<Box<[u64]>>());
        println!("small size: {}", std::mem::size_of::<Map6464<u64,u64>>());
        println!(" hash size: {}", std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<Map6464<u64,u64>>() <=
                2*std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<Map6464<u64,u64>>() <= 72);
    }

    #[test]
    fn simple_u8() {
        let mut m = Map6464::<u8,u8>::new();
        m.insert(5,1);
        assert_eq!(m.len(), 1);
        assert_eq!(m.get(&0), None);
        assert_eq!(m.get(&5), Some(1));
        for i in 6..80 {
            println!("inserting {}", i);
            m.insert(i,i);
            assert_eq!(m.get(&5), Some(1));
        }
        for i in 6..80 {
            assert_eq!(m.get(&i), Some(i));
        }
        for i in 81..255 {
            assert_eq!(m.get(&i), None);
        }
        assert_eq!(m.get(&5), Some(1));
        for i in 6..80 {
            println!("removing {}", i);
            assert_eq!(m.get(&i), Some(i));
            assert_eq!(m.get(&79), Some(79));
            assert_eq!(m.remove(&i), Some(i));
            assert_eq!(m.get(&i), None);
        }
        assert_eq!(m.get(&0), None);
        assert_eq!(m.get(&5), Some(1));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn simple() {
        let mut m = Map6464::new();
        m.insert(5,1);
        assert_eq!(m.len(), 1);
        assert_eq!(m.get(&0), None);
        assert_eq!(m.get(&5), Some(1));
        for i in 6..80 {
            println!("inserting {}", i);
            m.insert(i,i);
            assert_eq!(m.get(&5), Some(1));
        }
        for i in 6..80 {
            assert_eq!(m.get(&i), Some(i));
        }
        for i in 81..300 {
            assert_eq!(m.get(&i), None);
        }
        assert_eq!(m.get(&5), Some(1));
        for i in 6..80 {
            println!("removing {}", i);
            assert_eq!(m.get(&i), Some(i));
            assert_eq!(m.get(&79), Some(79));
            assert_eq!(m.remove(&i), Some(i));
            assert_eq!(m.get(&i), None);
        }
        assert_eq!(m.get(&0), None);
        assert_eq!(m.get(&5), Some(1));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn reproduce() {
        // let i = vec![Ok((0, 0)), Ok((2, 0)), Ok((3, 0)), Ok((5, 0)), Ok((6, 0)), Ok((1, 0)), Ok((7, 0)), Ok((8, 0)), Ok((21, 0)), Ok((9, 0)), Ok((10, 0)), Ok((11, 0)), Ok((12, 0)), Ok((13, 0)), Ok((14, 0)), Ok((48, 0)), Ok((15, 0)), Ok((17, 0)), Ok((4, 0)), Ok((18, 0)), Ok((20, 0)), Ok((22, 0)), Ok((19, 0)), Ok((16, 1))];
        let i = vec![Ok((0, 0)), Ok((7, 0)), Ok((3, 1))];

        let mut map = Map6464::<u8,u8>::new();
        let mut refmap = HashMap::<u8,u8>::new();
        for x in i {
            println!("  {:?}", map.m);
            match x {
                Ok((k,v)) => {
                    println!("inputting key {} as {}", k, v);
                    map.reserve(1);
                    println!("  after reserving {:?} {:?}", map.get(&7), map.m);
                    map.insert(k,v); refmap.insert(k,v);
                    assert_eq!(map.get(&k), Some(&v).map(|&x|x));
                },
                Err(k) => {
                    map.remove(k); refmap.remove(&k);
                }
            }
            assert_eq!(map.len(), refmap.len());
            for i in 0..255 {
                // println!("testing {}", i);
                if map.get(&i) != refmap.get(&i).map(|&x|x) {
                    println!("trouble with {}: {:?} and {:?}",
                             i, map.get(&i), refmap.get(&i));
                    println!("  {:?}", map.m);
                }
                assert_eq!(map.get(&i), refmap.get(&i).map(|&x|x));
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches(steps: Vec<Result<(u64,u64),u64>>) -> bool {
            let mut map = Map6464::<u64,u64>::new();
            let mut refmap = HashMap::<u64,u64>::new();
            for x in steps {
                match x {
                    Ok((k,v)) => {
                        map.insert(k,v); refmap.insert(k,v);
                    },
                    Err(k) => {
                        map.remove(&k); refmap.remove(&k);
                    }
                }
                if map.len() != refmap.len() {
                    return false;
                }
                for i in 0..2550 {
                    if map.get(&i) != refmap.get(&i).map(|&x|x) {
                        return false;
                    }
                }
            }
            true
        }

        fn prop_matches_with_invalid(steps: Vec<Result<(u64,u64),u64>>) -> bool {
            println!("\n\nstarting again...");
            let mut map = Map6464::<u64,u64>::new();
            let mut refmap = HashMap::<u64,u64>::new();
            for x in steps {
                match x {
                    Ok((mut k,mut v)) => {
                        if k == 8 {
                            k = u64::invalid();
                        }
                        println!("inserting {} -> {}", k, v);
                        if v == 32 {
                            v = u64::invalid();
                        }
                        map.insert(k,v); refmap.insert(k,v);
                        println!("map is now {:?}", map);
                        for (k,v) in map.iter() {
                            println!("    {}: {}", k, v);
                        }
                        println!("done with printing map... and asserting {} is some", k);
                        assert_eq!(map.get(&k), Some(v));
                        println!("done with assertion...");
                    },
                    Err(mut k) => {
                        if k == 8 {
                            k = u64::invalid();
                        }
                        println!("removing {}", k);
                        map.remove(&k); refmap.remove(&k);
                        assert!(map.get(&k).is_none());
                    }
                }
                if map.len() != refmap.len() {
                    return false;
                }
                for i in 0..2550 {
                    if map.get(&i) != refmap.get(&i).map(|&x|x) {
                        return false;
                    }
                }
                println!("done with checking");
            }
            true
        }

        fn prop_matches_u16(steps: Vec<Result<(u16,u16),u16>>) -> bool {
            let mut map = Map6464::<u16,u16>::new();
            let mut refmap = HashMap::<u16,u16>::new();
            for x in steps {
                match x {
                    Ok((k,v)) => {
                        map.insert(k,v); refmap.insert(k,v);
                    },
                    Err(k) => {
                        map.remove(&k); refmap.remove(&k);
                    }
                }
                if map.len() != refmap.len() {
                    return false;
                }
                for (k,v) in map.iter() {
                    if map.get(&k) != refmap.get(&k).map(|&x|x) {
                        return false;
                    }
                    if map.get(&k) != Some(v) {
                        return false;
                    }
                }
            }
            true
        }

        fn map64_matches_u8(steps: Vec<Result<(u8,u8),u8>>) -> bool {
            let mut map = Map6464::<u8,u8>::new();
            let mut refmap = HashMap::<u8,u8>::new();
            for x in steps {
                match x {
                    Ok((k,v)) => {
                        map.insert(k,v); refmap.insert(k,v);
                    },
                    Err(k) => {
                        map.remove(&k); refmap.remove(&k);
                    }
                }
                assert_eq!(map.len(), refmap.len());
                if map.len() != refmap.len() {
                    return false;
                }
                for i in 0..255 {
                    assert_eq!(map.get(&i), refmap.get(&i).map(|&x|x));
                    if map.get(&i) != refmap.get(&i).map(|&x|x) {
                        return false;
                    }
                }
            }
            true
        }
    }
}

/// A map type that can use any key that fits in a `u64` (i.e. that
/// satisfies trait `Fits64`).  This map type is very space-efficient
/// for keys that are small integers, while not being bad at storing
/// large integers.
///
/// **Major caveat** The `Map64<K,V>` defines an iterator that
/// iterates over `(K, &V)` rather than `(&K, &V)`.  This is a break
/// with standard libray convention, and can be annoying if you are
/// translating code from `HashMap` to `Map64`.  The motivation for
/// this is several-fold:
///
/// 1. `Map64` does not store `K` directly in its data structures
/// (which would waste space), so there is no reference to the data to
/// take.  This does not make it impossible, but does mean we would
/// have to fabricate a `K` and return a reference to it, which is
/// awkward and ugly.
///
/// 2. There is no inefficiency involved in returning `K`, since it is
/// necessarily no larger than a pointer (except on a 32-bit system).
///
/// # Examples
///
/// ```
/// use tinyset::Map64;
///
/// let mut a: Map64<char,&str> = Map64::new();
///
/// a.insert('a', "hello");
/// a.insert('b', "world");
/// assert_eq!(a.get(&'a'), Some(&"hello"));
/// assert_eq!(a.get(&'b'), Some(&"world"));
#[derive(Clone)]
pub struct Map64<K: Fits64, V> {
    set: U64Set,
    data: Box<[ManuallyDrop<V>]>,
    ph: PhantomData<K>,
}

impl<K: Fits64+std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for Map64<K,V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str("Map64 {\n  ")?;
        self.set.fmt(f)?;
        f.write_str(",\n  ")?;
        std::fmt::Pointer::fmt(&self.data, f)?;
        f.write_str(" [ ")?;
        let curcap = self.set.rawcapacity();
        for i in 0..curcap {
            if self.set.index(i).is_some() {
                (*self.data[i]).fmt(f)?;
            }
            if i < curcap-1 {
                f.write_str(",")?;
            }
        }
        f.write_str(" ]\n}")
    }
}

impl<K: Fits64, V> Map64<K,V> {
    /// Creates an empty `Map64`.
    pub fn new() -> Map64<K,V> {
        Map64::with_capacity(0)
    }
    /// Creates an empty `Map64` with the specified capacity.
    pub fn with_capacity(cap: usize) -> Map64<K,V> {
        let set = U64Set::with_capacity(cap);
        let mut v = Vec::new();
        for _ in 0..set.rawcapacity() {
            v.push(ManuallyDrop::new(unsafe{std::mem::uninitialized()}));
        }
        Map64 {
            set: set,
            data: v.into_boxed_slice(),
            ph: PhantomData,
        }
    }
    fn with_max_cap(max: u64, cap: usize) -> Map64<K,V> {
        let set = U64Set::with_max_and_capacity(max, cap);
        let mut v = Vec::new();
        for _ in 0..set.rawcapacity() {
            v.push(ManuallyDrop::new(unsafe{std::mem::uninitialized()}));
        }
        Map64 {
            set: set,
            data: v.into_boxed_slice(),
            ph: PhantomData,
        }
    }
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated,
    /// and the old value is returned.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        // First reserve space for the new thingy.
        let nextcap = capacity_to_rawcapacity(self.set.len()+1);
        let kk = k.to_u64();
        let curmax = self.set.current_max();
        let curcap = self.set.rawcapacity();
        if kk > curmax || nextcap > curcap {
            let max = if kk > curmax { kk } else { curmax };
            let mut n = Map64::with_max_cap(max, nextcap);
            for i in 0..curcap {
                if let Some(kkk) = self.set.index(i) {
                    let vvv = std::mem::replace(&mut self.data[i],
                                                ManuallyDrop::new(unsafe{std::mem::uninitialized()}));
                    n.insert_unchecked(kkk, vvv);
                }
            }
            std::mem::swap(&mut self.set, &mut n.set);
            n.set = U64Set::with_capacity(0);
            std::mem::swap(&mut self.data, &mut n.data);
        }
        self.insert_unchecked(k,ManuallyDrop::new(v))
    }
    fn insert_unchecked(&mut self, k: K, v: ManuallyDrop<V>) -> Option<V> {
        self.set.co_insert_unchecked(&mut self.data, k.to_u64(), v)
            .map(|x| ManuallyDrop::into_inner(x))
    }
    /// Removes a key from the map, returning the value at the key if
    /// the key was previously in the map.
    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.set.co_remove(&mut self.data, k.to_u64())
            .map(|x| ManuallyDrop::into_inner(x))
    }
    /// Returns true if the key is in the map.
    pub fn contains_key(&self, k: &K) -> bool {
        self.set.contains(&k.to_u64()).is_some()
    }
    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, k: &K) -> Option<&V> {
        self.set.contains(&k.to_u64()).map(|i| &*self.data[i])
    }
    /// len
    pub fn len(&self) -> usize {
        self.set.len()
    }
    /// An iterator visiting all key-value pairs in arbitrary
    /// order. The iterator element type is (K, &V).
    pub fn iter(&self) -> Map64Iter<K,V> {
        Map64Iter {
            m: self,
            which: 0,
            nleft: self.len(),
        }
    }
    /// An iterator visiting values in arbitrary
    /// order. The iterator element type is `&V`.
    pub fn values(&self) -> impl Iterator<Item=&V> + '_ {
        self.iter().map(|(_,v)| v)
    }
    /// A mutable iterator visiting values in arbitrary order. The
    /// iterator element type is `&mut V`.
    pub fn values_mut(&mut self) -> impl Iterator<Item=&mut V> + '_ {
        let set = &self.set;
        self.data.iter_mut().enumerate()
            .filter(move |(i,_)| set.index(*i).is_some())
            .map(|(_,x)| &mut **x)
    }
}

impl<K: Fits64, V> Drop for Map64<K,V> {
    fn drop(&mut self) {
        let curcap = self.set.rawcapacity();
        for i in 0..curcap {
            if self.set.index(i).is_some() {
                unsafe { ManuallyDrop::drop(&mut self.data[i]); }
            }
        }
    }
}

impl<K: Fits64, V: PartialEq> PartialEq for Map64<K,V> {
    fn eq(&self, other: &Map64<K,V>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (k, v) in other.iter() {
            if self.get(&k) != Some(v) {
                return false;
            }
        }
        true
    }
}
impl<K: Fits64, V: Eq> Eq for Map64<K,V> {}

/// Iterator for Map64
pub struct Map64Iter<'a, K: Fits64+'a, V: 'a> {
    m: &'a Map64<K,V>,
    which: usize,
    nleft: usize,
}

impl<'a,K: Fits64, V: 'a> Iterator for Map64Iter<'a,K,V> {
    type Item = (K, &'a V);
    fn next(&mut self) -> Option<(K,&'a V)> {
        if self.nleft == 0 {
            return None;
        }
        self.nleft -= 1;
        for i in self.which .. self.m.set.rawcapacity() {
            if let Some(k) = self.m.set.index(i) {
                self.which = i+1;
                return Some((unsafe { K::from_u64(k) }, &self.m.data[i]));
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.nleft, Some(self.nleft))
    }
}

#[cfg(test)]
mod mm64 {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn size_unwasted() {
        println!("\nsmall size: {}", std::mem::size_of::<Map64<u64,u64>>());
        println!(" hash size: {}", std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<Map64<u64,u64>>() <=
                2*std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<Map64<u64,u64>>() <= 48);
    }

    #[test]
    fn simple() {
        let mut m = Map64::<u64, String>::new();
        m.insert(0, String::from("hello"));
        assert_eq!(m.remove(&1), None);
        assert_eq!(m.remove(&0), Some(String::from("hello")));
        println!("goodbye {:?}", &m);
    }

    #[test]
    fn simple_u64() {
        let mut m = Map64::<u64, u64>::new();
        m.insert(0, 3);
        assert_eq!(m.remove(&1), None);
        println!("hello {:?}", &m);
        assert_eq!(m.remove(&0), Some(3));
        println!("goodbye {:?}", &m);
    }


    #[cfg(test)]
    quickcheck! {
        fn prop_matches(steps: Vec<Result<(u64,u64),u64>>) -> bool {
            let mut map = Map64::<u64,u64>::new();
            let mut refmap = HashMap::<u64,u64>::new();
            for x in steps {
                match x {
                    Ok((k,v)) => {
                        map.insert(k,v); refmap.insert(k,v);
                    },
                    Err(k) => {
                        map.remove(&k); refmap.remove(&k);
                    }
                }
                if map.len() != refmap.len() {
                    return false;
                }
                for i in 0..2550 {
                    if map.get(&i) != refmap.get(&i) {
                        return false;
                    }
                }
            }
            true
        }

        fn prop_matches_u16(steps: Vec<Result<(u16,u16),u16>>) -> bool {
            let mut map = Map64::<u16,u16>::new();
            let mut refmap = HashMap::<u16,u16>::new();
            for x in steps {
                match x {
                    Ok((k,v)) => {
                        map.insert(k,v); refmap.insert(k,v);
                    },
                    Err(k) => {
                        map.remove(&k); refmap.remove(&k);
                    }
                }
                if map.len() != refmap.len() {
                    return false;
                }
                for (k,v) in refmap.iter() {
                    if map.get(&k) != refmap.get(&k) {
                        return false;
                    }
                    if map.get(&k) != Some(&v) {
                        return false;
                    }
                }
                for (k,v) in map.iter() {
                    if map.get(&k) != refmap.get(&k) {
                        return false;
                    }
                    if map.get(&k) != Some(&v) {
                        return false;
                    }
                }
            }
            true
        }

        fn map64_matches_u8(steps: Vec<Result<(u8,u8),u8>>) -> bool {
            let mut map = Map64::<u8,u8>::new();
            let mut refmap = HashMap::<u8,u8>::new();
            for x in steps {
                match x {
                    Ok((k,v)) => {
                        assert_eq!(map.insert(k,v), refmap.insert(k,v));
                    },
                    Err(k) => {
                        map.remove(&k); refmap.remove(&k);
                    }
                }
                assert_eq!(map.len(), refmap.len());
                if map.len() != refmap.len() {
                    return false;
                }
                for i in 0..255 {
                    assert_eq!(map.get(&i), refmap.get(&i));
                    if map.get(&i) != refmap.get(&i) {
                        return false;
                    }
                }
            }
            true
        }
    }

    #[test]
    fn reproduce() {
        let i = vec![Ok((0, 0)), Ok((1, 0)), Ok((2, 0)), Ok((3, 0)), Ok((4, 0)),
                     Ok((5, 0)), Ok((6, 0)), Ok((7, 0)), Ok((8, 0)), Ok((9, 0)),
                     Ok((10, 0)), Ok((11, 0)), Ok((12, 0)), Ok((13, 0)), Ok((14, 0)),
                     Ok((0, 0))];

        let mut map = Map64::<u8,u8>::new();
        let mut refmap = HashMap::<u8,u8>::new();
        for x in i {
            println!("  {:?}", map);
            match x {
                Ok((k,v)) => {
                    println!("inputting key {} as {}", k, v);
                    assert_eq!(map.insert(k,v), refmap.insert(k,v));
                    assert_eq!(map.get(&k), Some(&v));
                },
                Err(k) => {
                    map.remove(k); refmap.remove(&k);
                }
            }
            println!("afterwards:  {:?}", map);
            assert_eq!(map.len(), refmap.len());
            for i in 0..255 {
                // println!("testing {}", i);
                if map.get(&i) != refmap.get(&i) {
                    println!("trouble with {}: {:?} and {:?}",
                             i, map.get(&i), refmap.get(&i));
                    println!("  {:?}", map);
                }
                assert_eq!(map.get(&i), refmap.get(&i));
            }
        }
    }
}
