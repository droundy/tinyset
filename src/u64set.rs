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
    fn reserve_with_max(&mut self, max: u64, additional: usize) {
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
    fn insert(&mut self, elem: u64) -> bool {
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
            &mut Drain::U64{ref mut slice, ref mut nleft} => {
                if *nleft == 0 {
                    None
                } else {
                    assert!(slice.len() >= *nleft);
                    let mut val = slice.pop().unwrap();
                    while val == u64::invalid() {
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
            &Drain::U64{slice: _, nleft} => (nleft, Some(nleft)),
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
        self.0.contains(&x)
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

const MAP_NUM_U8: usize = 23;
const MAP_NUM_U16: usize = 15;
const MAP_NUM_U32: usize = 9;
const MAP_NUM_U64: usize = 4;

/// A map of u64 elements to sequential integers
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
                keys: vec![u64::invalid(); nextcap].into_boxed_slice(),
                vals: vec![u64::invalid(); nextcap].into_boxed_slice(),
            }
        }
    }
    fn with_max_cap(max: u64, cap: usize) -> U64Map {
        let nextcap = capacity_to_rawcapacity(cap);
        if max < u8::invalid() as u64 {
            if cap <= NUM_U8 {
                U64Map::Su8 { sz: 0, keys: [0; MAP_NUM_U8], vals: [0; MAP_NUM_U8] }
            } else {
                U64Map::Vu8 {
                    sz: 0,
                    keys: vec![u8::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![u8::invalid(); nextcap].into_boxed_slice(),
                }
            }
        } else if max < u16::invalid() as u64 {
            if cap <= NUM_U16 {
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
        } else if max < u32::invalid() as u64 {
            if cap <= NUM_U32 {
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
        } else if max < u64::invalid() as u64 {
            if cap <= NUM_U64 {
                U64Map::Su64 {
                    sz: 0,
                    keys: [u64::invalid(); MAP_NUM_U64],
                    vals: [0; MAP_NUM_U64]
                }
            } else {
                U64Map::Vu64 {
                    sz: 0,
                    keys: vec![u64::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![u64::invalid(); nextcap].into_boxed_slice(),
                }
            }
        } else {
            unimplemented!()
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
                match search(keys, k) {
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
                        let oldv = vals[i];
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v);
                        Some(oldv as u64)
                    },
                }
            },
            &mut U64Map::Vu32 { ref mut sz, ref mut keys, ref mut vals } => {
                let mut k = k as u32;
                let mut v = v as u32;
                match search(keys, k) {
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
                        let oldv = vals[i];
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v);
                        Some(oldv as u64)
                    },
                }
            },
            &mut U64Map::Vu64 { ref mut sz, ref mut keys, ref mut vals } => {
                let mut k = k as u64;
                let mut v = v as u64;
                match search(keys, k) {
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
                        let oldv = vals[i];
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v);
                        Some(oldv as u64)
                    },
                }
            },
            &mut U64Map::Vu16 { ref mut sz, ref mut keys, ref mut vals } => {
                let mut k = k as u16;
                let mut v = v as u16;
                match search(keys, k) {
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
                        let oldv = vals[i];
                        std::mem::swap(&mut keys[i], &mut k);
                        std::mem::swap(&mut vals[i], &mut v);
                        mapsteal(keys, vals, i, k, v);
                        Some(oldv as u64)
                    },
                }
            },
        }
    }
    fn change_value(&mut self, oldv: u64, newv: u64) {
        match self {
            &mut U64Map::Su8 { sz, keys:_, ref mut vals } => {
                for x in vals.iter_mut().take(sz as usize) {
                    if *x == oldv as u8 {
                        *x = newv as u8;
                        return;
                    }
                }
            },
            &mut U64Map::Su16 { sz, keys:_, ref mut vals } => {
                for x in vals.iter_mut().take(sz as usize) {
                    if *x == oldv as u8 {
                        *x = newv as u8;
                        return;
                    }
                }
            },
            &mut U64Map::Su32 { sz, keys:_, ref mut vals } => {
                for x in vals.iter_mut().take(sz as usize) {
                    if *x == oldv as u8 {
                        *x = newv as u8;
                        return;
                    }
                }
            },
            &mut U64Map::Su64 { sz, keys:_, ref mut vals } => {
                for x in vals.iter_mut().take(sz as usize) {
                    if *x == oldv as u8 {
                        *x = newv as u8;
                        return;
                    }
                }
            },
            &mut U64Map::Vu8 { sz:_, keys:_, ref mut vals } => {
                for x in vals.iter_mut() {
                    if *x == oldv as u8 {
                        println!("changing value from {} to {}", oldv, newv);
                        *x = newv as u8;
                    }
                }
            },
            &mut U64Map::Vu16 { sz:_, keys:_, ref mut vals } => {
                for x in vals.iter_mut() {
                    if *x == oldv as u16 {
                        *x = newv as u16;
                    }
                }
            },
            &mut U64Map::Vu32 { sz:_, keys:_, ref mut vals } => {
                for x in vals.iter_mut() {
                    if *x == oldv as u32 {
                        *x = newv as u32;
                    }
                }
            },
            &mut U64Map::Vu64 { sz:_, keys:_, ref mut vals } => {
                for x in vals.iter_mut() {
                    if *x == oldv as u64 {
                        *x = newv as u64;
                    }
                }
            },
        }
    }
    /// Reserves capacity for at least `additional` more elements to
    /// be inserted in the set, with maximum value of `max`. The
    /// collection may reserve more space to avoid frequent
    /// reallocations.
    fn reserve_with_max(&mut self, max: u64, additional: usize) {
        let mut newself: Option<U64Map> = None;
        match *self {
            U64Map::Su8 { sz, keys: k, vals: v } if max >= u8::invalid() as u64 => {
                let mut n = Self::with_max_cap(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(k[i] as u64, v[i] as u64);
                }
                *self = n;
            },
            U64Map::Su8 { sz, keys, vals } if sz as usize + additional > NUM_U8 => {
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
            U64Map::Su16 { sz, keys: k, vals: v } if max >= u16::invalid() as u64 => {
                let mut n = Self::with_max_cap(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(k[i] as u64, v[i] as u64);
                }
                Some(n);
            },
            U64Map::Su16 { sz, keys, vals } if sz as usize + additional > NUM_U16 => {
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
            U64Map::Su32 { sz, keys: k, vals: v } if max >= u32::invalid() as u64 => {
                let mut n = Self::with_max_cap(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(k[i] as u64, v[i] as u64);
                }
                *self = n;
            },
            U64Map::Su32 { sz, keys, vals } if sz as usize + additional > NUM_U32 => {
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
            U64Map::Su64 { sz, keys: k, vals: v } if max >= u64::invalid() as u64 => {
                let mut n = Self::with_max_cap(max, sz as usize + additional);
                for i in 0..sz as usize {
                    n.insert_unchecked(k[i] as u64, v[i] as u64);
                }
                *self = n;
            },
            U64Map::Su64 { sz, keys, vals } if sz as usize + additional > NUM_U64 => {
                let nextcap = capacity_to_rawcapacity(sz as usize + additional);
                *self = U64Map::Vu64 {
                    sz: 0,
                    keys: vec![u64::invalid(); nextcap].into_boxed_slice(),
                    vals: vec![0; nextcap].into_boxed_slice(),
                };
                for i in 0..sz as usize {
                    self.insert_unchecked(keys[i] as u64, vals[i] as u64);
                }
            },
            U64Map::Su64 {sz:_,keys:_,vals:_} => (),
            U64Map::Vu8 {sz,ref keys,ref vals} if max >= u8::invalid() as u64 => {
                let mut n = Self::with_max_cap(max, sz as usize + additional);
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
                        match search(keys, key) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => {
                                keys[i] = key;
                                vals[i] = value;
                            },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut keys[i], &mut key);
                                std::mem::swap(&mut vals[i], &mut value);
                                mapsteal(keys, vals, i, key, value);
                            },
                        }
                    }
                }
            },
            U64Map::Vu8 {sz:_,keys:_,vals:_} => (),
            U64Map::Vu16 {sz,ref keys,ref vals} if max >= u16::invalid() as u64 => {
                let mut n = Self::with_max_cap(max, sz as usize + additional);
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
                        match search(keys, key) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => {
                                keys[i] = key;
                                vals[i] = value;
                            },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut keys[i], &mut key);
                                std::mem::swap(&mut vals[i], &mut value);
                                mapsteal(keys, vals, i, key, value);
                            },
                        }
                    }
                }
            },
            U64Map::Vu16 {sz:_,keys:_,vals:_} => (),
            U64Map::Vu32 {sz,ref keys,ref vals} if max >= u32::invalid() as u64 => {
                let mut n = Self::with_max_cap(max, sz as usize + additional);
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
                        match search(keys, key) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => {
                                keys[i] = key;
                                vals[i] = value;
                            },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut keys[i], &mut key);
                                std::mem::swap(&mut vals[i], &mut value);
                                mapsteal(keys, vals, i, key, value);
                            },
                        }
                    }
                }
            },
            U64Map::Vu32 {sz:_,keys:_,vals:_} => (),
            U64Map::Vu64 {sz,ref keys,ref vals} if max >= u64::invalid() as u64 => {
                let mut n = Self::with_max_cap(max, sz as usize + additional);
                for i in 0..keys.len() {
                    if keys[i] != u64::invalid() {
                        n.insert_unchecked(keys[i] as u64, vals[i] as u64);
                    }
                }
                newself = Some(n);
            },
            U64Map::Vu64 {sz,ref mut keys,ref mut vals} if sz as usize + additional > keys.len()*10/11 => {
                let newcap = capacity_to_rawcapacity(sz as usize+additional);
                let oldkeys = std::mem::replace(keys,
                                                vec![u64::invalid(); newcap].into_boxed_slice());
                let oldvals = std::mem::replace(vals,
                                                vec![0; newcap].into_boxed_slice());
                for (&k, &v) in oldkeys.iter().zip(oldvals.iter()) {
                    if k != u64::invalid() {
                        let mut key = k;
                        let mut value = v;
                        match search(keys, key) {
                            SearchResult::Present(_) => (),
                            SearchResult::Empty(i) => {
                                keys[i] = key;
                                vals[i] = value;
                            },
                            SearchResult::Richer(i) => {
                                std::mem::swap(&mut keys[i], &mut key);
                                std::mem::swap(&mut vals[i], &mut value);
                                mapsteal(keys, vals, i, key, value);
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
        self.reserve_with_max(k,1);
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
                if k >= u64::invalid() as u64 {
                    return None;
                }
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
                match search(keys, k) {
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
                match search(keys, k) {
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
                match search(keys, k) {
                    SearchResult::Present(i) => Some(vals[i] as u64),
                    SearchResult::Empty(_) => None,
                    SearchResult::Richer(_) => None,
                }
            },
            U64Map::Vu64 {sz:_, ref keys, ref vals } => {
                if k >= u64::invalid() as u64 {
                    return None;
                }
                let k = k as u64;
                match search(keys, k) {
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
                match search(keys, k) {
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
                match search(keys, k) {
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
                match search(keys, k) {
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
                if k >= u64::invalid() as u64 {
                    return None;
                }
                let k = k as u64;
                match search(keys, k) {
                    SearchResult::Present(mut i) => {
                        let oldval = vals[i];
                        *sz -= 1;
                        let mask = keys.len() - 1;
                        let invalid = u64::invalid();
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
}

fn mapsteal<K: HasInvalid, V>(k: &mut [K], v: &mut [V], mut i: usize, mut elem: K, mut val: V) {
    loop {
        match search_from(k, i, elem) {
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

/// A mapping from small things
pub struct Map64<K: Fits64, T> {
    m: U64Map,
    data: Vec<T>,
    ph: PhantomData<K>,
}

impl<K: Fits64 + std::fmt::Display,T: std::fmt::Debug> Map64<K,T> {
    /// Create a Map64
    pub fn new() -> Self {
        Map64 {
            m: U64Map::with_capacity(1),
            data: Vec::new(),
            ph: PhantomData,
        }
    }
    /// How many elems?
    pub fn len(&self) -> usize {
        self.m.len()
    }
    /// Insert a value
    pub fn insert(&mut self, k: K, v: T) {
        if let Some(i) = self.m.insert(k.to_u64(), self.data.len() as u64) {
            // FIXME: this is needlessly inefficient
            self.m.insert(k.to_u64(), i);
            self.data[i as usize] = v;
        } else {
            self.data.push(v);
        }
    }
    /// Remove avalue
    pub fn remove(&mut self, k: K) -> Option<T> {
        if let Some(i) = self.m.remove(k.to_u64()) {
            self.m.change_value(self.data.len() as u64-1, i);
            return Some(self.data.swap_remove(i as usize))
        }
        None
    }
    /// Get an element
    pub fn get(&self, k: K) -> Option<&T> {
        if let Some(i) = self.m.get(k.to_u64()) {
            return Some(&self.data[i as usize])
        }
        None
    }
}

#[cfg(test)]
mod map64_tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn size_unwasted() {
        println!("box size: {}", std::mem::size_of::<Box<[u64]>>());
        println!("small size: {}", std::mem::size_of::<Map64<u64,u64>>());
        println!(" hash size: {}", std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<Map64<u64,u64>>() <=
                2*std::mem::size_of::<HashMap<u64,u64>>());
        assert!(std::mem::size_of::<Map64<u64,u64>>() <= 72);
    }

    #[test]
    fn simple() {
        let mut m = Map64::new();
        m.insert(5,1);
        assert_eq!(m.len(), 1);
        assert_eq!(m.get(0), None);
        assert_eq!(m.get(5), Some(&1));
        for i in 6..80 {
            println!("inserting {}", i);
            m.insert(i,i);
            assert_eq!(m.get(5), Some(&1));
        }
        for i in 6..80 {
            assert_eq!(m.get(i), Some(&i));
        }
        for i in 81..300 {
            assert_eq!(m.get(i), None);
        }
        assert_eq!(m.get(5), Some(&1));
        for i in 6..80 {
            println!("removing {}", i);
            assert_eq!(m.get(i), Some(&i));
            assert_eq!(m.get(79), Some(&79));
            assert_eq!(m.remove(i), Some(i));
            assert_eq!(m.get(i), None);
        }
        assert_eq!(m.get(0), None);
        assert_eq!(m.get(5), Some(&1));
        assert_eq!(m.len(), 1);
    }

    // #[cfg(test)]
    // quickcheck! {
    //     fn prop_matches(steps: Vec<Result<(u64,u64),u64>>) -> bool {
    //         let mut map = U64Map::with_capacity(0);
    //         let mut refmap = HashMap::<u64,u64>::new();
    //         for x in steps {
    //             match x {
    //                 Ok((k,v)) => {
    //                     map.insert(k,v); refmap.insert(k,v);
    //                 },
    //                 Err(k) => {
    //                     map.remove(k); refmap.remove(&k);
    //                 }
    //             }
    //             if map.len() != refmap.len() {
    //                 return false;
    //             }
    //             for i in 0..2550 {
    //                 if map.get(i) != refmap.get(&i).map(|&v| v) {
    //                     return false;
    //                 }
    //             }
    //         }
    //         true
    //     }
    // }

}
