//! A set that is compact in size.

use std;

use tinyset::HasInvalid;

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
pub struct U64Set {
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
            unimplemented!()
        }
    }
}

fn capacity_to_rawcapacity(cap: usize) -> usize {
    (cap*11/10).next_power_of_two()
}

impl U64Set {
    /// Creates an empty set..
    pub fn default() -> U64Set {
        Self::with_capacity(0)
    }
    /// Creates an empty set..
    pub fn new() -> U64Set {
        U64Set::with_capacity(0)
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_capacity(cap: usize) -> U64Set {
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
    pub fn with_max_and_capacity(max: u64, cap: usize) -> U64Set {
        U64Set { v: Data::with_max_cap(max, cap) }
    }
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        match &self.v {
            &Data::Su8(sz,_) => sz as usize,
            &Data::Vu8(sz,_) => sz as usize,
            &Data::Su16(sz,_) => sz as usize,
            &Data::Vu16(sz,_) => sz as usize,
            &Data::Su32(sz,_) => sz as usize,
            &Data::Vu32(sz,_) => sz as usize,
            &Data::Su64(sz,_) => sz as usize,
            &Data::Vu64(sz,_) => sz as usize,
        }
    }
    /// Reserves capacity for at least `additional` more elements to be
    /// inserted in the set. The collection may reserve more space
    /// to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        match self.v {
            Data::Su8(sz, v) if sz as usize + additional > NUM_U8 => {
                self.v = Data::Vu8(0, vec![u8::invalid();
                                           ((sz as usize+additional)*11/10).next_power_of_two()]
                                   .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64);
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
    pub fn reserve_with_max(&mut self, max: u64, additional: usize) {
        match self.v {
            Data::Su8(sz, v) if max >= u8::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as u64);
                }
                *self = n;
            },
            Data::Su8(sz, v) if sz as usize + additional > NUM_U8 => {
                self.v = Data::Vu8(0, vec![u8::invalid();
                                           ((sz as usize+additional)*11/10).next_power_of_two()]
                                   .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64);
                }
            },
            Data::Su8(_,_) => (),
            Data::Su16(sz, v) if max >= u16::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as u64);
                }
                *self = n;
            },
            Data::Su16(sz, v) if sz as usize + additional > NUM_U16 => {
                self.v = Data::Vu16(0, vec![u16::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64);
                }
            },
            Data::Su16(_,_) => (),
            Data::Su32(sz, v) if max >= u32::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as u64);
                }
                *self = n;
            },
            Data::Su32(sz, v) if sz as usize + additional > NUM_U32 => {
                self.v = Data::Vu32(0, vec![u32::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64);
                }
            },
            Data::Su32(_,_) => (),
            Data::Su64(sz, v) if max >= u64::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as u64);
                }
                *self = n;
            },
            Data::Su64(sz, v) if sz as usize + additional > NUM_U64 => {
                self.v = Data::Vu64(0, vec![u64::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as u64);
                }
            },
            Data::Su64(_,_) => (),
            Data::Vu8(sz, _) if max >= u8::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x);
                }
                *self = n;
            },
            Data::Vu16(sz, _) if max >= u16::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x);
                }
                *self = n;
            },
            Data::Vu32(sz, _) if max >= u32::invalid() as u64 => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x);
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
                        match search(v, value) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value);
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
                        match search(v, value) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value);
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
                        match search(v, value) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value);
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
                        match search(v, value) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => { v[i] = value; },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut v[i], &mut value);
                                steal(v, i, value);
                            },
                        }
                    }
                }
            },
            Data::Vu64(_,_) => (),
        }
    }
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn insert(&mut self, elem: u64) -> bool {
        self.reserve_with_max(elem, 1);
        self.insert_unchecked(elem)
    }
    fn insert_unchecked(&mut self, value: u64) -> bool {
        match self.v {
            Data::Su8(ref mut sz, ref mut v) => {
                let value = value as u8;
                for &x in v.iter().take(*sz as usize) {
                    if x == value {
                        return false;
                    }
                }
                v[*sz as usize] = value;
                *sz += 1;
                true
            },
            Data::Su16(ref mut sz, ref mut v) => {
                let value = value as u16;
                for &x in v.iter().take(*sz as usize) {
                    if x == value {
                        return false;
                    }
                }
                v[*sz as usize] = value;
                *sz += 1;
                true
            },
            Data::Su32(ref mut sz, ref mut v) => {
                let value = value as u32;
                for &x in v.iter().take(*sz as usize) {
                    if x == value {
                        return false;
                    }
                }
                v[*sz as usize] = value;
                *sz += 1;
                true
            },
            Data::Su64(ref mut sz, ref mut v) => {
                let value = value as u64;
                for &x in v.iter().take(*sz as usize) {
                    if x == value {
                        return false;
                    }
                }
                v[*sz as usize] = value;
                *sz += 1;
                true
            },
            Data::Vu8(ref mut sz, ref mut v) => {
                let mut value = value as u8;
                match search(v, value) {
                    SearchResult::Present(_) => false,
                    SearchResult::Empty(i) => {
                        v[i] = value;
                        *sz += 1;
                        true
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut v[i], &mut value);
                        steal(v, i, value);
                        true
                    },
                }
            },
            Data::Vu16(ref mut sz, ref mut v) => {
                let mut value = value as u16;
                match search(v, value) {
                    SearchResult::Present(_) => false,
                    SearchResult::Empty(i) => {
                        v[i] = value;
                        *sz += 1;
                        true
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut v[i], &mut value);
                        steal(v, i, value);
                        true
                    },
                }
            },
            Data::Vu32(ref mut sz, ref mut v) => {
                let mut value = value as u32;
                match search(v, value) {
                    SearchResult::Present(_) => {
                        false
                    },
                    SearchResult::Empty(i) => {
                        v[i] = value;
                        *sz += 1;
                        true
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut v[i], &mut value);
                        steal(v, i, value);
                        true
                    },
                }
            },
            Data::Vu64(ref mut sz, ref mut v) => {
                let mut value = value as u64;
                match search(v, value) {
                    SearchResult::Present(_) => false,
                    SearchResult::Empty(i) => {
                        v[i] = value;
                        *sz += 1;
                        true
                    },
                    SearchResult::Richer(i) => {
                        *sz += 1;
                        std::mem::swap(&mut v[i], &mut value);
                        steal(v, i, value);
                        true
                    },
                }
            },
        }
    }
    /// Returns true if the set contains a value.
    pub fn contains(&self, value: &u64) -> bool {
        let value = *value;
        match self.v {
            Data::Su8(sz, ref v) => {
                if value >= u8::invalid() as u64 {
                    return false;
                }
                let value = value as u8;
                for &x in v.iter().take(sz as usize) {
                    if x == value {
                        return true;
                    }
                }
                false
            },
            Data::Su16(sz, ref v) => {
                if value >= u16::invalid() as u64 {
                    return false;
                }
                let value = value as u16;
                for &x in v.iter().take(sz as usize) {
                    if x == value {
                        return true;
                    }
                }
                false
            },
            Data::Su32(sz, ref v) => {
                if value >= u32::invalid() as u64 {
                    return false;
                }
                let value = value as u32;
                for &x in v.iter().take(sz as usize) {
                    if x == value {
                        return true;
                    }
                }
                false
            },
            Data::Su64(sz, ref v) => {
                if value >= u64::invalid() as u64 {
                    return false;
                }
                let value = value as u64;
                for &x in v.iter().take(sz as usize) {
                    if x == value {
                        return true;
                    }
                }
                false
            },
            Data::Vu8(_, ref v) => {
                if value >= u8::invalid() as u64 {
                    return false;
                }
                let value = value as u8;
                match search(v, value) {
                    SearchResult::Present(_) => true,
                    SearchResult::Empty(_) => false,
                    SearchResult::Richer(_) => false,
                }
            },
            Data::Vu16(_, ref v) => {
                if value >= u16::invalid() as u64 {
                    return false;
                }
                let value = value as u16;
                match search(v, value) {
                    SearchResult::Present(_) => true,
                    SearchResult::Empty(_) => false,
                    SearchResult::Richer(_) => false,
                }
            },
            Data::Vu32(_, ref v) => {
                if value >= u32::invalid() as u64 {
                    return false;
                }
                let value = value as u32;
                match search(v, value) {
                    SearchResult::Present(_) => true,
                    SearchResult::Empty(_) => false,
                    SearchResult::Richer(_) => false,
                }
            },
            Data::Vu64(_, ref v) => {
                if value >= u64::invalid() as u64 {
                    return false;
                }
                let value = value as u64;
                match search(v, value) {
                    SearchResult::Present(_) => true,
                    SearchResult::Empty(_) => false,
                    SearchResult::Richer(_) => false,
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
                match search(v, value) {
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
                match search(v, value) {
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
                match search(v, value) {
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
                match search(v, value) {
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
                    slice: &v[0..sz as usize],
                    nleft: sz as usize,
                }
            },
            Data::Vu64(sz, ref v) => {
                Iter::U64 {
                    slice: v,
                    nleft: sz as usize,
                }
            },
        }
    }
    // /// Clears the set, returning all elements in an iterator.
    // pub fn drain(&mut self) -> IntoIter {
    //     let set = std::mem::replace(self, U64Set::new());
    //     let sz = set.len();
    //     IntoIter { set: set, nleft: sz }
    // }
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
        /// this really should be private
        slice: &'a [u64],
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
            &mut Iter::U64{ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    while slice[0] == u64::invalid() {
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
            &Iter::U64{slice: _, nleft} => (nleft, Some(nleft)),
        }
    }
}

// impl IntoIterator for &U64Set {
//     type Item = &T;
//     type IntoIter = Iter;

//     fn into_iter(self) -> Iter {
//         self.iter()
//     }
// }

// /// An iterator for `U64Set`.
// pub struct IntoIter {
//     set: U64Set,
//     nleft: usize,
// }

// impl Iterator for IntoIter {
//     type Item = usize;
//     fn next(&mut self) -> Option<&usize> {
//         if self.nleft == 0 {
//             None
//         } else {
//             self.nleft -= 1;
//             let mut i = self.nleft;
//             loop {
//                 let val = std::mem::replace(&mut self.set.v.mu()[i], T::invalid());
//                 if val != T::invalid() {
//                     return Some(val);
//                 }
//                 i -= 1;
//             }
//         }
//     }
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.nleft, Some(self.nleft))
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn it_works() {
        let mut ss = U64Set::new();
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
        let mut set = U64Set::new();
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
                if set.contains(&i) != refset.contains(&i) {
                    println!("trouble at {}", i);
                    assert_eq!(set.contains(&i), refset.contains(&i));
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_matches(steps: Vec<Result<u64,u64>>) -> bool {
            let mut steps = steps;
            let mut set = U64Set::new();
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
                    None => return true,
                }
                if set.len() != refset.len() { return false; }
                for i in 0..2550 {
                    if set.contains(&i) != refset.contains(&i) { return false; }
                }
            }
        }
    }

    #[cfg(test)]
    quickcheck! {
        fn prop_bigint(steps: Vec<Result<(u64,u8),(u64,u8)>>) -> bool {
            let mut steps = steps;
            let mut set = U64Set::new();
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
                    if set.contains(&i) != refset.contains(&i) {
                        println!("refset: {:?}", &refset);
                        println!("set: {:?}", &set);
                        for x in set.iter() {
                            print!(" {}", x);
                        }
                        println!();
                        assert_eq!(set.contains(&i), refset.contains(&i));
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
        let mut set = U64Set::new();
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
                if set.contains(&i) != refset.contains(&i) {
                    println!("refset: {:?}", &refset);
                    println!("set: {:?}", &set);
                    for x in set.iter() {
                        print!(" {}", x);
                    }
                    println!();
                    assert_eq!(set.contains(&i), refset.contains(&i));
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

fn search<T: HasInvalid>(v: &[T], elem: T) -> SearchResult {
    let h = elem.hash_usize();
    let invalid = T::invalid();
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

fn search_from<T: HasInvalid>(v: &[T], i_start: usize, elem: T) -> SearchResult {
    let h = elem.hash_usize();
    let mask = v.len() - 1;
    let invalid = T::invalid();
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

fn steal<T: HasInvalid>(v: &mut [T], mut i: usize, mut elem: T) {
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

/// This describes a type which can be stored in 64 bits without loss.
/// It is defined for all signed and unsigned integer types.  In both
/// cases, we store "small" integers as "small" `u64`s.
pub trait Fits64 : Clone + std::fmt::Debug {
    /// Convert back *from* a u64.  This is unsafe, since it is only
    /// infallible if the `u64` originally came from type `Self`.
    #[inline]
    unsafe fn from_u64(x: u64) -> Self;
    /// Convert to a `u64`.  This should be infallible.
    #[inline]
    fn to_u64(self) -> u64;
    /// verify that the conversion is lossless
    fn test_fits64(self) -> bool {
        println!("\ntest_fits64 {:?}", &self);
        let x = self.to_u64();
        let y = unsafe { Self::from_u64(x).to_u64() };
        println!("x: {}, and y: {}", x, y);
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

/// A set type that can store any type that fits in a `u64`.
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
    pub fn contains(&self, value: &T) -> bool {
        let x = value.clone().to_u64();
        self.0.contains(&x)
    }
    /// Removes an element, and returns true if that element was present.
    pub fn remove(&mut self, value: &T) -> bool {
        let x = value.clone().to_u64();
        self.0.remove(&x)
    }
}
