//! A set that is compact in size.

use std;

use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

use tinyset::HasInvalid;

enum SearchResult {
    Present(usize),
    Empty(usize),
    /// The element is not present, but there is someone richer than
    /// us we could steal from!
    Richer(usize),
}

/// A set implemented of usize elements
#[derive(Debug,Clone)]
pub struct USizeSet {
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
    fn with_max_cap(max: usize, cap: usize) -> Data {
        if max < u8::invalid() as usize {
            if cap <= NUM_U8 {
                Data::Su8(0, [u8::invalid(); NUM_U8])
            } else {
                Data::Vu8(0, vec![u8::invalid(); (cap*11/10).next_power_of_two()]
                          .into_boxed_slice())
            }
        } else if max < u16::invalid() as usize {
            if cap <= NUM_U16 {
                Data::Su16(0, [u16::invalid(); NUM_U16])
            } else {
                Data::Vu16(0, vec![u16::invalid(); (cap*11/10).next_power_of_two()]
                           .into_boxed_slice())
            }
        } else if max < u32::invalid() as usize {
            if cap <= NUM_U32 {
                Data::Su32(0, [u32::invalid(); NUM_U32])
            } else {
                Data::Vu32(0, vec![u32::invalid(); (cap*11/10).next_power_of_two()]
                           .into_boxed_slice())
            }
        } else if max < u64::invalid() as usize {
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

impl USizeSet {
    /// Creates an empty set..
    pub fn default() -> USizeSet {
        Self::with_capacity(0)
    }
    /// Creates an empty set..
    pub fn new() -> USizeSet {
        USizeSet::with_capacity(0)
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_capacity(cap: usize) -> USizeSet {
        let nextcap = capacity_to_rawcapacity(cap);
        if cap <= NUM_U8 {
            USizeSet { v: Data::new() }
        } else if cap < u8::invalid() as usize {
            USizeSet { v: Data::Vu8( 0, vec![u8::invalid(); nextcap].into_boxed_slice()) }
        } else {
            USizeSet {
                v: Data::Vu16(0, vec![u16::invalid(); nextcap].into_boxed_slice()),
            }
        }
    }
    /// Creates an empty set with the specified capacity.
    pub fn with_max_and_capacity(max: usize, cap: usize) -> USizeSet {
        USizeSet { v: Data::with_max_cap(max, cap) }
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
                    self.insert_unchecked(v[i] as usize);
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
    pub fn reserve_with_max(&mut self, max: usize, additional: usize) {
        match self.v {
            Data::Su8(sz, v) if max >= u8::invalid() as usize => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as usize);
                }
                *self = n;
            },
            Data::Su8(sz, v) if sz as usize + additional > NUM_U8 => {
                self.v = Data::Vu8(0, vec![u8::invalid();
                                           ((sz as usize+additional)*11/10).next_power_of_two()]
                                   .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as usize);
                }
            },
            Data::Su8(_,_) => (),
            Data::Su16(sz, v) if max >= u16::invalid() as usize => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as usize);
                }
                *self = n;
            },
            Data::Su16(sz, v) if sz as usize + additional > NUM_U16 => {
                self.v = Data::Vu16(0, vec![u16::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as usize);
                }
            },
            Data::Su16(_,_) => (),
            Data::Su32(sz, v) if max >= u32::invalid() as usize => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as usize);
                }
                *self = n;
            },
            Data::Su32(sz, v) if sz as usize + additional > NUM_U32 => {
                self.v = Data::Vu32(0, vec![u32::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as usize);
                }
            },
            Data::Su32(_,_) => (),
            Data::Su64(sz, v) if max >= u64::invalid() as usize => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(v[i] as usize);
                }
                *self = n;
            },
            Data::Su64(sz, v) if sz as usize + additional > NUM_U64 => {
                self.v = Data::Vu64(0, vec![u64::invalid();
                                            ((sz as usize+additional)*11/10).next_power_of_two()]
                                    .into_boxed_slice());
                for i in 0..sz as usize {
                    self.insert_unchecked(v[i] as usize);
                }
            },
            Data::Su64(_,_) => (),
            Data::Vu8(sz, _) if max >= u8::invalid() as usize => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x);
                }
                *self = n;
            },
            Data::Vu16(sz, _) if max >= u16::invalid() as usize => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x);
                }
                *self = n;
            },
            Data::Vu32(sz, _) if max >= u32::invalid() as usize => {
                let mut n = Self::with_max_and_capacity(max, sz as usize + additional);
                for x in self.iter() {
                    n.insert_unchecked(x);
                }
                *self = n;
            },
            Data::Vu64(sz, _) if max >= u64::invalid() as usize => {
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
    fn max_and_cap(&self) -> (usize, usize) {
        match self.v {
            Data::Su8(_, ref v) => (u8::invalid() as usize - 1, v.len()),
            Data::Vu8(_, ref v) => (u8::invalid() as usize - 1, v.len()*10/11),
            Data::Su16(_, ref v) => (u16::invalid() as usize - 1, v.len()),
            Data::Vu16(_, ref v) => (u16::invalid() as usize - 1, v.len()*10/11),
            Data::Su32(_, ref v) => (u32::invalid() as usize - 1, v.len()),
            Data::Vu32(_, ref v) => (u32::invalid() as usize - 1, v.len()*10/11),
            Data::Su64(_, ref v) => (u64::invalid() as usize - 1, v.len()),
            Data::Vu64(_, ref v) => (u64::invalid() as usize - 1, v.len()*10/11),
        }
    }
    fn capacity(&self) -> usize {
        match self.v {
            Data::Su8(_, ref v) => v.len(),
            Data::Vu8(_, ref v) => v.len()*10/11,
            Data::Su16(_, ref v) => v.len(),
            Data::Vu16(_, ref v) => v.len()*10/11,
            Data::Su32(_, ref v) => v.len(),
            Data::Vu32(_, ref v) => v.len()*10/11,
            Data::Su64(_, ref v) => v.len(),
            Data::Vu64(_, ref v) => v.len()*10/11,
        }
    }
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn insert(&mut self, elem: usize) -> bool {
        self.reserve_with_max(elem, 1);
        self.insert_unchecked(elem)
    }
    fn insert_unchecked(&mut self, value: usize) -> bool {
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
    pub fn contains(&self, value: &usize) -> bool {
        let value = *value;
        match self.v {
            Data::Su8(sz, ref v) => {
                if value >= u8::invalid() as usize {
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
                if value >= u16::invalid() as usize {
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
                if value >= u32::invalid() as usize {
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
                if value >= u64::invalid() as usize {
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
                if value >= u8::invalid() as usize {
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
                if value >= u16::invalid() as usize {
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
                if value >= u32::invalid() as usize {
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
                if value >= u64::invalid() as usize {
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
    pub fn remove(&mut self, value: &usize) -> bool {
        let value = *value;
        match self.v {
            Data::Su8(ref mut sz, ref mut v) => {
                if value >= u8::invalid() as usize {
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
                if value >= u16::invalid() as usize {
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
                if value >= u32::invalid() as usize {
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
                if value >= u64::invalid() as usize {
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
                if value >= u8::invalid() as usize {
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
                if value >= u16::invalid() as usize {
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
                if value >= u32::invalid() as usize {
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
                if value >= u64::invalid() as usize {
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
    //     let set = std::mem::replace(self, USizeSet::new());
    //     let sz = set.len();
    //     IntoIter { set: set, nleft: sz }
    // }
}

/// An iterator for `USizeSet`.
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
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
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
                    Some(val as usize)
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
                    Some(val as usize)
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
                    Some(val as usize)
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
                    Some(val as usize)
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

// impl IntoIterator for &USizeSet {
//     type Item = &T;
//     type IntoIter = Iter;

//     fn into_iter(self) -> Iter {
//         self.iter()
//     }
// }

// /// An iterator for `USizeSet`.
// pub struct IntoIter {
//     set: USizeSet,
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
    use rand::{XorShiftRng, SeedableRng, Rand};
    #[test]
    fn it_works() {
        let mut ss = USizeSet::new();
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
        println!("small size: {}", std::mem::size_of::<USizeSet>());
        println!(" hash size: {}", std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<USizeSet>() <=
                2*std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<USizeSet>() <= 24);
    }

    // macro_rules! initialize {
    //     ($set: expr, $item: ident, $num: expr) => {{
    //         let mut rng = XorShiftRng::from_seed([$num as u32,$num as u32,3,4]);
    //         let mut set = $set;
    //         let mut refset = HashSet::<$item>::new();
    //         if $num > 0 {
    //             while set.len() < $num {
    //                 let ins = $item::rand(&mut rng) % (2*$num as $item);
    //                 let rem = $item::rand(&mut rng) % (2*$num as $item);
    //                 set.insert(ins);
    //                 if !set.contains(&ins) {
    //                     println!("oops insert");
    //                 }
    //                 set.remove(&rem);
    //                 if set.contains(&rem) {
    //                     println!("oops remove");
    //                 }
    //                 refset.insert(ins);
    //                 refset.remove(&rem);
    //                 println!("inserting {}, removing {} => {}", ins, rem, set.len());
    //                 println!("set: {:?}", set);
    //                 println!("refset: {:?}", refset);
    //                 let mut fails = false;
    //                 for i in 0..255 {
    //                     fails = fails || set.contains(&i) != refset.contains(&i);
    //                 }
    //                 if fails {
    //                     for i in 0..255 {
    //                         println!("i {}", i);
    //                         assert_eq!(set.contains(&i), refset.contains(&i));
    //                     }
    //                 }
    //             }
    //         }
    //         set
    //     }};
    // }

    // #[test]
    // fn random_inserts_and_removals() {
    //     for sz in 0..50 {
    //         println!("\nUSizeSet {}\n", sz);
    //         let myset = initialize!(USizeSet::new(), usize, sz);
    //         println!("\nHashSet {}\n", sz);
    //         let refset = initialize!(HashSet::<usize>::new(), usize, sz);
    //         for i in 0..255 {
    //             assert_eq!(myset.contains(&i), refset.contains(&i));
    //         }
    //     }
    // }

    #[test]
    fn test_matches() {
        let mut steps: Vec<Result<usize,usize>> = vec![Err(8), Ok(0), Ok(16), Ok(1), Ok(8)];
        let mut set = USizeSet::new();
        let mut refset = HashSet::<usize>::new();
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
        fn prop_matches(steps: Vec<Result<usize,usize>>) -> bool {
            let mut steps = steps;
            let mut set = USizeSet::new();
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

    #[cfg(test)]
    quickcheck! {
        fn prop_bigint(steps: Vec<Result<(usize,u8),(usize,u8)>>) -> bool {
            let mut steps = steps;
            let mut set = USizeSet::new();
            let mut refset = HashSet::<usize>::new();
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
        let mut steps: Vec<Result<(usize,u8),(usize,u8)>> =
            vec![Ok((84, 30)), Ok((0, 0)), Ok((0, 0)), Ok((1, 0)),
                 Ok((1, 1)), Ok((1, 2)), Ok((2, 15))];
        let mut set = USizeSet::new();
        let mut refset = HashSet::<usize>::new();
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

