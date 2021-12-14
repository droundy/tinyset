#![deny(missing_docs)]
//! This is a crate for the tiniest sets ever.

use itertools::Itertools;

const fn num_bits<T>() -> u64 { std::mem::size_of::<T>() as u64 * 8 }

fn log_2(x: u64) -> u64 {
    if x == 0 {
        1
    } else {
        num_bits::<u64>() as u64 - x.leading_zeros() as u64
    }
}

#[test]
fn test_log_2() {
    assert_eq!(log_2(0), 1);
    assert_eq!(log_2(1), 1);
    assert_eq!(log_2(7), 3);
    assert_eq!(log_2(8), 4);
}

fn compute_array_bits(mx: u64) -> u64 {
    if log_2(mx) < 2 {
        return 62;
    } else if log_2(mx) > 62 {
        return 0;
    }
    let mut bits = num_bits::<u64>() - log_2(mx);
    while num_bits::<u64>() - log_2(mx) < bits {
        bits += 1;
    }
    bits
}

fn split_u64(x: u64, bits: u64) -> (u64, u64) {
    if bits > 0 {
        (x / bits, (x % bits))
    } else {
        (x, 0)
    }
}

fn unsplit_u64(k: u64, offset: u64, bits: u64) -> u64 {
    if bits > 0 {
        k*bits + offset
    } else {
        k
    }
}

/// A set of u64
pub struct SetU64(*mut S);

unsafe impl Send for SetU64 {}
unsafe impl Sync for SetU64 {}

impl std::fmt::Debug for SetU64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "SetU64 {:?}", self.iter().collect::<Vec<_>>())?;
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug)]
struct S {
    sz: usize,
    cap: usize,
    bits: u64,
    array: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tiny {
    sz: u8,
    sz_spent: u8,
    bits: usize,
    last: usize,
}

fn mask(bits: usize) -> u64 {
    (1 << bits) - 1
}

impl Iterator for Tiny {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let bitsplits = BITSPLITS[self.sz as usize];
        if self.sz_spent < self.sz {
            let nbits = bitsplits[self.sz_spent as usize];
            let difference = self.bits & mask(nbits as usize) as usize;
            if self.sz_spent == 0 {
                self.last = difference;
            } else {
                self.last = self.last + 1 + difference
            }
            self.bits = self.bits >> nbits;
            self.sz_spent += 1;
            Some(self.last as u64)
        } else {
            None
        }
    }
    fn count(self) -> usize {
        self.sz as usize
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz as usize, Some(self.sz as usize))
    }
    fn min(mut self) -> Option<u64> {
        self.next()
    }
}

#[cfg(target_pointer_width = "64")]
static BITSPLITS: [&[u64]; 8] = [
    &[],
    &[61],
    &[40,21],
    &[31,15,15],
    &[25,12,12,12],
    &[21,10,10,10,10],
    &[21, 8, 8, 8, 8, 8],
    &[19, 7, 7, 7, 7, 7, 7],
];

#[cfg(target_pointer_width = "32")]
static BITSPLITS: [&[u64]; 8] = [
    &[],
    &[29],
    &[20,9],
    &[15,7,7],
    &[12, 5, 5, 5],
    &[10, 4, 4, 4, 4],
    &[ 9, 4, 4, 4, 4, 4],
    &[ 8, 3, 3, 3, 3, 3, 3],
];

impl Tiny {
    #[cfg(test)]
    fn debug_me(self, msg: &str) {
        println!("{}: {:?} => {:?}", msg, self, self.collect::<Vec<_>>());
    }
    fn to_usize(self) -> usize {
        self.sz as usize | self.bits << 3
    }
    fn from_usize(x: usize) -> Self {
        Tiny {
            sz: x as u8 & 7,
            bits: x >> 3,
            sz_spent: 0,
            last: 0,
        }
    }
    fn from_singleton(x: u64) -> Option<Self> {
        if log_2(x) > BITSPLITS[1][0] {
            None
        } else {
            Some(Tiny {
                sz: 1,
                bits: x as usize,
                sz_spent: 0,
                last: 0,
            })
        }
    }
    fn new_sorted_deduped(v: &[u64]) -> Option<Self> {
        if v.len() == 0 {
            return None;
        } else if v.len() > BITSPLITS.len() - 1 {
            return None;
        }
        let sz = v.len() as u8;
        let mut last = 0;
        let mut offset = 0;
        let mut bits: usize = 0;
        let bitsplits = BITSPLITS[sz as usize];
        for (x,nbits) in v.iter().cloned().zip(bitsplits.iter().cloned()) {
            let y = if offset == 0 {
                x
            } else {
                x - last - 1
            };
            if log_2(y) > nbits {
                return None;
            }
            bits = bits | (y as usize) << offset;
            offset += nbits;
            last = x;
        }
        Some (Tiny { sz, bits, sz_spent: 0, last: 0 })
    }
    fn insert(mut self, e: u64) -> Option<Self> {
        if e > std::usize::MAX as u64 {
            return None;
        }
        let mut e = e as usize;
        let old_bitsplits = BITSPLITS[self.sz as usize];
        if let Some(new_bitsplits) = BITSPLITS.get(self.sz as usize + 1) {
            let mut new = Tiny {
                bits: 0,
                sz: self.sz + 1,
                last: 0,
                sz_spent: 0,
            };
            let backup = self.clone();
            let mut offset = 0;
            let mut old_iter = old_bitsplits.iter().cloned();
            let mut new_iter = new_bitsplits.iter().cloned();
            while let Some(newb) = new_iter.next() {
                if let Some(oldb) = old_iter.next() {
                    let mut n = self.bits & mask(oldb as usize) as usize;
                    if e == n {
                        return Some(backup);
                    } else if log_2(n as u64) > newb {
                        if e < n {
                            return None;
                        }
                        e -= n + 1;
                        self.bits = self.bits >> oldb;
                        for oldb in old_iter {
                            let n = self.bits & mask(oldb as usize) as usize;
                            if n == e { return Some(backup); }
                            if e < n { return None; }
                            e -= n + 1;
                            self.bits = self.bits >> oldb;
                        }
                        return None;
                    } else if e < n {
                        new.bits = new.bits | (e << offset);
                        offset += newb;
                        n = n - e - 1;
                        let newb = new_iter.next().unwrap();
                        if log_2(n as u64) > newb {
                            return None;
                        }
                        new.bits = new.bits | (n << offset);
                        offset += newb;
                        self.bits = self.bits >> oldb;
                        for newb in new_iter {
                            let oldb = old_iter.next().unwrap();
                            let n = self.bits & mask(oldb as usize) as usize;
                            if log_2(n as u64) > newb {
                                return None;
                            }
                            new.bits = new.bits | (n << offset);
                            self.bits = self.bits >> oldb;
                            offset += newb;
                        }
                        return Some(new);
                    }
                    e -= n + 1;
                    new.bits = new.bits | (n << offset);
                    offset += newb;
                    self.bits = self.bits >> oldb;
                } else {
                    // the new one is last
                    if log_2(e as u64) > newb {
                        return None;
                    }
                    new.bits = new.bits | (e << offset);
                }
            }
            Some(new)
        } else {
            if self.clone().any(|x| x == e as u64) {
                Some(self)
            } else {
                None
            }
        }
    }
    fn contains(mut self, e: u64) -> bool {
        if e > std::usize::MAX as u64 {
            return false;
        }
        let mut e = e as usize;
        let bitsplits = BITSPLITS[self.sz as usize];
        for b in bitsplits.iter().cloned() {
            let n: usize = self.bits & mask(b as usize) as usize;
            if e == n {
                return true;
            } else if e < n {
                return false;
            }
            e -= n + 1;
            self.bits = self.bits >> b;
        }
        false
    }
}

#[cfg(test)]
fn test_vec(v: Vec<u64>) {
    println!("\ntesting {:?}", v);
    assert_eq!(Tiny::new_sorted_deduped(&v).unwrap().collect::<Vec<_>>(), v);
}

#[test]
fn test_tiny() {
    assert_eq!(Tiny::new_sorted_deduped(&[]), None);
    test_vec(vec![1]);
    test_vec(vec![1024]);
    test_vec(vec![1,2]);
    test_vec(vec![1,2,3]);
    test_vec(vec![1,2,3,4,5]);
    test_vec(vec![1,2,3,4,5,6]);
    test_vec(vec![1,2,3,4,5,6,7]);
}

enum Internal<'a> {
    Empty,
    Stack(Tiny),
    Heap {
        s: &'a S,
        a: &'a [u64],
    },
    Big {
        s: &'a S,
        a: &'a [u64],
    },
    Dense {
        sz: usize,
        a: &'a [u64],
    },
}
enum InternalMut<'a> {
    Empty,
    Stack(Tiny),
    Heap {
        s: &'a mut S,
        a: &'a mut [u64],
    },
    Big {
        s: &'a mut S,
        a: &'a mut [u64],
    },
    Dense {
        sz: &'a mut usize,
        a: &'a mut [u64],
    },
}

#[derive(Debug)]
enum Iter<'a> {
    Empty,
    Stack(Tiny),
    Heap(HeapIter<'a>),
    Big(BigIter<'a>),
    Dense(DenseIter<'a>),
}

impl<'a> Iterator for Iter<'a> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<u64> {
        match self {
            Iter::Empty => None,
            Iter::Stack(ref mut t) => t.next(),
            Iter::Dense(it) => it.next(),
            Iter::Big(it) => it.next(),
            Iter::Heap(it) => it.next(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        match self {
            Iter::Empty => 0,
            Iter::Stack(t) => t.count(),
            Iter::Dense(it) => it.count(),
            Iter::Big(it) => it.count(),
            Iter::Heap(it) => it.count(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Empty => (0, Some(0)),
            Iter::Stack(t) => t.size_hint(),
            Iter::Dense(it) => it.size_hint(),
            Iter::Big(it) => it.size_hint(),
            Iter::Heap(it) => it.size_hint(),
        }
    }
    #[inline]
    fn min(self) -> Option<u64> {
        match self {
            Iter::Empty => None,
            Iter::Stack(t) => t.min(),
            Iter::Dense(it) => it.min(),
            Iter::Big(it) => it.min(),
            Iter::Heap(it) => it.min(),
        }
    }
    #[inline]
    fn max(self) -> Option<u64> {
        match self {
            Iter::Empty => None,
            Iter::Stack(t) => t.max(),
            Iter::Dense(it) => it.max(),
            Iter::Big(it) => it.max(),
            Iter::Heap(it) => it.max(),
        }
    }
}
/// An iterator over a set of `u64`.
#[derive(Debug)]
pub struct IntoIter {
    iter: Iter<'static>,
    set: SetU64,
}
impl IntoIterator for SetU64 {
    type Item = u64;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        let iter = unsafe { std::mem::transmute(self.private_iter()) };
        IntoIter {
            iter,
            set: self,
        }
    }
}
impl Iterator for IntoIter {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }
    #[inline]
    fn min(self) -> Option<Self::Item> {
        self.iter.min()
    }
    #[inline]
    fn max(self) -> Option<Self::Item> {
        self.iter.max()
    }
    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl Clone for SetU64 {
    fn clone(&self) -> Self {
        if self.0 as usize & 7 == 0 && self.0 != std::ptr::null_mut() {
            let c = self.capacity();
            unsafe {
                let ptr = std::alloc::alloc_zeroed(layout_for_capacity(c)) as *mut S;
                if ptr == std::ptr::null_mut() {
                    std::alloc::handle_alloc_error(layout_for_capacity(c));
                }
                std::ptr::copy_nonoverlapping(self.0 as *const u8, ptr as *mut u8,
                                              bytes_for_capacity(c));
                SetU64(ptr)
            }
        } else {
            SetU64(self.0)
        }
    }
}

#[test]
fn just_clone() {
    let mut x = SetU64::with_capacity_and_max(100, 1000000);
    x.insert(100); x.insert(1000);
    let y = x.clone();
    assert_eq!(x.len(), y.len());
    assert_eq!(x.len(), x.into_iter().count());
    assert_eq!(y.len(), y.clone().into_iter().count());
}

impl SetU64 {
    /// The number of elements in the set
    #[inline]
    pub fn len(&self) -> usize {
        match self.internal() {
            Internal::Empty => 0,
            Internal::Stack(t) => t.sz as usize,
            Internal::Heap { s, .. } => {
                s.sz
            }
            Internal::Big { s, .. } => {
                s.sz
            }
            Internal::Dense { sz, .. } => {
                sz
            }
        }
    }
    /// The capacity of the set
    #[inline]
    pub fn capacity(&self) -> usize {
        match self.internal() {
            Internal::Empty => 0,
            Internal::Stack(_) => 0,
            Internal::Heap { a, .. } => {
                a.len()
            }
            Internal::Big { a, .. } => {
                a.len()
            }
            Internal::Dense { a, .. } => {
                a.len()
            }
        }
    }
    /// Print debugging information about this set.
    pub fn debug_me(&self, msg: &str) {
        match self.internal() {
            Internal::Empty => println!("empty set: {}", msg),
            Internal::Stack(t) => println!("{}: stack {:?} => {:?}", msg, t, t.collect::<Vec<_>>()),
            Internal::Heap { s, a } => {
                println!("{}: heap {:?}", msg, s);
                for (i,x) in a.iter().cloned().enumerate() {
                    println!("      {} (key {} pov {}): {:0b} ({})",
                             (x >> s.bits)*s.bits, x >> s.bits,
                             p_poverty(x >> s.bits, i, a.len()),
                             x & mask(s.bits as usize), x);
                }
                println!("    => {:?}", self.iter().collect::<Vec<_>>());
            }
            Internal::Big { s, a } => {
                println!("{}: big {:?}\n    {:?}", msg, s, a);
                let v: Vec<_> = a.iter().cloned().map(|x| x % a.len() as u64).collect();
                println!("     >>>{:?}", v);
            }
            Internal::Dense { sz, a } => {
                println!("{}: dense {:?}\n    {:?}\n    => {:?}",
                         msg, sz, a, self.iter().collect::<Vec<u64>>());
                println!("    foo {:?}", self.iter());
            }
        }
    }
    /// Tally up how much memory is in use.
    #[inline]
    pub fn mem_used(&self) -> usize {
        match self.internal() {
            Internal::Empty => std::mem::size_of::<Self>(),
            Internal::Stack(_) => std::mem::size_of::<Self>(),
            Internal::Heap { s, .. } =>
                std::mem::size_of::<Self>() + s.cap*8-8,
            Internal::Dense { a, .. } =>
                std::mem::size_of::<Self>() + a.len()*8-8,
            Internal::Big { s, .. } =>
                std::mem::size_of::<Self>() + s.cap*8-8,
        }
    }
    fn dense_with_max(mx: u64) -> SetU64 {
        let cap = 1 + mx/64 + mx/256;
        // This should be stored in a dense bitset.
        unsafe {
            let x = SetU64(std::alloc::alloc_zeroed(layout_for_capacity(cap as usize))
                           as *mut S);
            (*x.0).cap = cap as usize;
            (*x.0).bits = 64;
            x
        }
    }

    /// Create a set with the given capacity
    pub fn with_capacity_and_max(cap: usize, mx: u64) -> SetU64 {
        if cap as u64 > mx >> 7 {
            SetU64::dense_with_max(mx)
        } else {
            SetU64::with_capacity_and_bits(cap, compute_array_bits(mx))
        }
    }
    /// Create a set with the given capacity and bits
    pub fn with_capacity_and_bits(cap: usize, bits: u64) -> SetU64 {
        if cap > 0 {
            unsafe {
                let x = SetU64(std::alloc::alloc_zeroed(layout_for_capacity(cap)) as *mut S);
                (*x.0).cap = cap;
                (*x.0).bits = if bits == 0 {
                    let mut b = 0;
                    while b <= 64 {
                        b = crate::rand::rand64();
                    }
                    b
                } else {
                    bits
                };
                x
            }
        } else {
            SetU64(0 as *mut S)
        }
    }
    /// An empty set
    #[inline]
    pub const fn new() -> Self {
        SetU64(0 as *mut S)
    }

    /// Insert and return true if it was not present.
    pub fn insert(&mut self, e: u64) -> bool {
        match self.internal_mut() {
            InternalMut::Empty => {
                if let Some(t) = Tiny::from_singleton(e) {
                    *self = SetU64(t.to_usize() as *mut S);
                    return true;
                }
                // println!("I could not create tiny set with singleton {}", e);
                *self = Self::with_capacity_and_max(1, e);
            }
            InternalMut::Stack(t) => {
                if let Some(newt) = t.insert(e) {
                    *self = SetU64(newt.to_usize() as *mut S);
                    return newt.sz != t.sz;
                }
                *self = Self::with_capacity_and_max(t.sz as usize + 1,
                                                    t.merge(Some(e).into_iter()).max().unwrap());
                // self.debug_me("empty array");
                for x in t {
                    self.insert(x);
                    // self.debug_me(&format!("   ...after inserting {}", x));
                }
                self.insert(e);
                return true;
            }
            _ => (),
        }
        match self.internal_mut() {
            InternalMut::Empty => unreachable!(),
            InternalMut::Stack(_) => unreachable!(),
            InternalMut::Dense { sz, a } => {
                let key = (e >> 6) as usize;
                if let Some(bits) = a.get_mut(key) {
                    let whichbit = 1 << (e & 63);
                    let present = *bits & whichbit != 0;
                    *bits = *bits | whichbit;
                    if !present {
                        *sz = *sz + 1;
                    }
                    !present
                } else {
                    // println!("key is {}", key);
                    if key > 128*(*sz as usize) {
                        // It is getting sparse, so let us switch back
                        // to a non-hash table.
                        let cap = 2*(*sz + 1);
                        let mut new = SetU64::with_capacity_and_bits(cap as usize, 0);
                        for x in self.iter() {
                            new.insert(x);
                        }
                        new.insert(e);
                        *self = new;
                    } else {
                        let mut new = SetU64::with_capacity_and_bits(1 + key + key/4, 64);
                        match new.internal_mut() {
                            InternalMut::Empty => unreachable!(),
                            InternalMut::Stack(_) => unreachable!(),
                            InternalMut::Big { .. } => unreachable!(),
                            InternalMut::Heap { .. } => unreachable!(),
                            InternalMut::Dense { sz: newsz, a: na } => {
                                for (i,v) in a.iter().cloned().enumerate() {
                                    na[i] = v;
                                }
                                na[key] = 1 << (e & 63);
                                *newsz = *sz + 1;
                            }
                        }
                        *self = new;
                    }
                    true
                }
            }
            InternalMut::Heap { s, a } => {
                if compute_array_bits(e) < s.bits {
                    let mut new = Self::with_capacity_and_bits(s.cap+1+2*(crate::rand::rand_usize() % s.cap),
                                                               compute_array_bits(e));
                    // new.debug_me("\n\nnew set");
                    for d in self.iter() {
                        new.insert(d);
                        // new.debug_me(&format!("\n -- after inserting {}", d));
                    }
                    new.insert(e);
                    // new.debug_me(&format!("\n -- after inserting {}", e));
                    *self = new;
                    return true;
                }
                let (key, offset) = split_u64(e, s.bits);
                match p_lookfor(key, a, s.bits) {
                    LookedUp::KeyFound(idx) => {
                        if a[idx] & (1 << offset) != 0 {
                            return false;
                        } else {
                            a[idx] = a[idx] | (1 << offset);
                            s.sz += 1;
                            return true;
                        }
                    }
                    LookedUp::EmptySpot(idx) => {
                            a[idx] = key << s.bits | 1 << offset;
                            s.sz += 1;
                            return true;
                    }
                    LookedUp::NeedInsert => {
                    },
                }
                // println!("looking for space in sparse... {:?}", a);
                if a.iter().cloned().any(|x| x == 0) {
                    let idx = p_insert(key, a, s.bits);
                    // println!("about to insert key {} with elem {} at {}",
                    //          key, e, idx);
                    a[idx] = (key << s.bits) | (1 << offset);
                    s.sz += 1;
                    return true;
                }
                // println!("no room in the sparse set... {:?}", a);
                // We'll have to expand the set.
                let mx = a.iter().cloned().map(|x| (x >> s.bits)*s.bits + s.bits).max().unwrap();
                let mx = if e > mx { e } else { mx };
                if s.cap as u64 > mx >> 6 {
                    // A dense set will save memory
                    let mut new = Self::dense_with_max(mx);
                    for x in self.iter() {
                        new.insert(x);
                    }
                    new.insert(e);
                    *self = new;
                } else {
                    // Let's keep things sparse
                    // A dense set will cost us memory
                    let newcap: usize = s.cap + 1 + (crate::rand::rand_usize() % s.cap);
                    let mut new = Self::with_capacity_and_bits(newcap, s.bits);
                    // new.debug_me("initial new");
                    for v in self.iter() {
                        new.insert(v);
                    }
                    new.insert(e);
                    *self = new;
                }
                true
            }
            InternalMut::Big { s, a } => {
                if e == s.bits {
                    // Pick a new number not present in the set.  We
                    // use a cryptographically secure random number
                    // generator to look for a new number not present,
                    // which feels like overkill.  But the cost of
                    // changing the "bits" is $O(N)$, so it's worth
                    // a high O(1) cost to reduce collisions.
                    let had_zero = p_remove(s.bits, a, 0);
                    loop {
                        let i: u64 = crate::rand::rand64();
                        if i > 64 && !a.iter().any(|&v| v == i) {
                            s.bits = i;
                            break;
                        }
                    }
                    if had_zero {
                        a[p_insert(s.bits, a, 0)] = s.bits;
                    }
                }
                let e = if e == 0 { s.bits } else { e };
                match p_lookfor(e, a, 0) {
                    LookedUp::KeyFound(_) => {
                        return false;
                    }
                    LookedUp::EmptySpot(idx) => {
                        a[idx] = e;
                        s.sz += 1;
                        return true;
                    }
                    LookedUp::NeedInsert => (),
                }
                // println!("looking for space in... {:?}", a);
                if a.iter().cloned().any(|x| x == 0) {
                    // println!("about to insert at {}", p_insert(e, a, 0));
                    a[p_insert(e, a, 0)] = e;
                    s.sz += 1;
                    return true;
                }
                // println!("no room in the set... {:?}", a);
                let newcap: usize = s.cap + 1 + (crate::rand::rand_usize() % (2*s.cap));
                let mut new = Self::with_capacity_and_bits(newcap, s.bits);
                // new.debug_me("initial new");
                match new.internal_mut() {
                    InternalMut::Empty => unreachable!(),
                    InternalMut::Stack(_) => unreachable!(),
                    InternalMut::Dense { .. } => unreachable!(),
                    InternalMut::Heap { .. } => unreachable!(),
                    InternalMut::Big { s: ns, a: na } => {
                        for v in a.iter().cloned().filter(|&x| x != 0) {
                            na[p_insert(v, na, 0)] = v;
                            // println!("  with {} gives {:?}", v, na);
                            // let v: Vec<_> = na.iter().cloned().map(|x| x % na.len() as u64).collect();
                            // println!("     >>>{:?}", v);
                        }
                        na[p_insert(e, na, 0)] = e;
                        // println!("  with {} gives {:?}", e, na);
                        ns.sz = s.sz + 1;
                        // println!("  size ends up as {}", ns.sz);
                        // new.debug_me("aftr growing");
                    }
                }
                *self = new;
                true
            }
        }
    }

    /// Remove
    pub fn remove(&mut self, e: u64) -> bool {
        match self.internal_mut() {
            InternalMut::Empty => false,
            InternalMut::Stack(t) => {
                if t.clone().any(|x| x == e) {
                    let sz = t.sz - 1;
                    if sz == 0 {
                        *self = SetU64(0 as *mut S);
                    } else {
                        *self = t.filter(|&x| x != e).collect();
                    }
                    true
                } else {
                    false
                }
            }
            InternalMut::Dense { sz, a } => {
                let key = e >> 6;
                if let Some(bits) = a.get_mut(key as usize) {
                    let whichbit = 1 << (e & 63);
                    let present = *bits & whichbit != 0;
                    *bits = *bits & !whichbit;
                    if present {
                        *sz = *sz - 1;
                    }
                    present
                } else {
                    false
                }
            }
            InternalMut::Heap { s, a } => {
                if compute_array_bits(e) < s.bits {
                    return false;
                }
                let (key, offset) = split_u64(e, s.bits);
                if let LookedUp::KeyFound(idx) = p_lookfor(key, a, s.bits) {
                    if a[idx] & (1 << offset) != 0 {
                        let newa = a[idx] & !(1<<offset);
                        s.sz -= 1;
                        if newa == key << s.bits {
                            // We've removed everything with this key,
                            // so remove the whole key!
                            p_remove(key, a, s.bits);
                        } else {
                            a[idx] = newa;
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            InternalMut::Big { s, a } => {
                if e == s.bits {
                    return false;
                }
                let e = if e == 0 { s.bits } else { e };
                let had_e = p_remove(e, a, 0);
                if had_e {
                    s.sz -= 1;
                }
                had_e
            }
        }
    }

    /// Contais
    pub fn contains(&self, e: u64) -> bool {
        match self.internal() {
            Internal::Empty => false,
            Internal::Stack(t) => t.contains(e), // t.clone().any(|x| x == e),
            Internal::Dense { a, .. } => {
                let key = e >> 6;
                if let Some(bits) = a.get(key as usize) {
                    bits & (1 << (e & 63)) != 0
                } else {
                    false
                }
            }
            Internal::Heap { s, a } => {
                if compute_array_bits(e) < s.bits {
                    // println!("too big a thing");
                    return false;
                }
                let (key, offset) = split_u64(e, s.bits);
                if let LookedUp::KeyFound(idx) = p_lookfor(key, a, s.bits) {
                    a[idx] & (1 << offset) != 0
                } else {
                    // self.debug_me(&format!("did not find key {} from {}", key, e));
                    false
                }
            }
            Internal::Big { s, a } => {
                if e == s.bits {
                    return false;
                }
                let e = if e == 0 { s.bits } else { e };
                p_lookfor(e, a, 0).key_found()
            }
        }
    }

    /// Iterate over
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=u64> + 'a + std::fmt::Debug {
        self.private_iter()
    }
    fn private_iter<'a>(&'a self) -> Iter<'a> {
        match self.internal() {
            Internal::Empty => Iter::Empty,
            Internal::Stack(t) => Iter::Stack( t ),
            Internal::Heap { s, a } => {
                Iter::Heap( HeapIter {
                    sz_left: s.sz,
                    bits: s.bits,
                    whichbit: 0,
                    array: a,
                })
            }
            Internal::Big { s, a } => {
                Iter::Big(BigIter { sz_left: s.sz, bits: s.bits, a })
            }
            Internal::Dense { a, sz } => {
                Iter::Dense(DenseIter { sz_left: sz, whichword: 0, whichbit: 0, a })
            }
        }
    }
    /// Clears the set, returning all elements in an iterator.
    #[inline]
    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item=u64> + 'a {
        let set: SetU64 = std::mem::replace(self, SetU64::new());
        let iter = unsafe { std::mem::transmute(set.private_iter()) };
        IntoIter {
            iter,
            set,
        }
    }

    fn internal<'a>(&'a self) -> Internal<'a> {
        if self.0 as usize == 0 {
            Internal::Empty
        } else if self.0 as usize & 7 != 0 {
            Internal::Stack(Tiny::from_usize(self.0 as usize))
        } else {
            let s = unsafe { &*self.0 };
            let a = unsafe { std::slice::from_raw_parts(&s.array as *const u64, s.cap) };
            if s.bits == 0 || s.bits > 64 {
                Internal::Big { s, a }
            } else if s.bits == 64 {
                Internal::Dense { sz: s.sz, a }
            } else {
                Internal::Heap { s, a }
            }
        }
    }

    fn internal_mut<'a>(&'a mut self) -> InternalMut<'a> {
        if self.0 as usize == 0 {
            InternalMut::Empty
        } else if self.0 as usize & 7 != 0 {
            InternalMut::Stack(Tiny::from_usize(self.0 as usize))
        } else {
            let s = unsafe { &mut *self.0 };
            let a = unsafe { std::slice::from_raw_parts_mut(&mut s.array as *mut u64, s.cap) };
            if s.bits == 0 || s.bits > 64 {
                InternalMut::Big { s, a }
            } else if s.bits == 64 {
                InternalMut::Dense { sz: &mut s.sz, a }
            } else {
                InternalMut::Heap { s, a }
            }
        }
    }
}

#[derive(Debug)]
struct BigIter<'a> {
    sz_left: usize,
    bits: u64,
    a: &'a [u64],
}

impl<'a> Iterator for BigIter<'a> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<u64> {
        while let Some((&x, rest)) = self.a.split_first() {
            self.a = rest;
            if x != 0 {
                self.sz_left -= 1;
                return Some( if x == self.bits { 0 } else { x });
            }
        }
        None
    }
    #[inline]
    fn last(self) -> Option<u64> {
        self.a.into_iter().rev().cloned()
            .filter(|&x| x != 0)
            .map(|x| if x == self.bits { 0 } else { x })
            .next()
    }
    #[inline]
    fn count(self) -> usize {
        self.sz_left
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz_left, Some(self.sz_left))
    }
    #[inline]
    fn min(self) -> Option<u64> {
        if self.sz_left == 0 {
            None
        } else {
            self.a.into_iter().cloned()
                .filter(|x| *x != 0)
                .map(|x| if x == self.bits { 0 } else { x }).min()
        }
    }
}

#[derive(Debug)]
struct HeapIter<'a> {
    sz_left: usize,
    bits: u64,
    whichbit: u64,
    array: &'a [u64],
}

impl<'a> Iterator for HeapIter<'a> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits > 0 {
            while let Some(&x) = self.array.first() {
                while self.whichbit < self.bits {
                    let oldbit = self.whichbit;
                    self.whichbit += 1;
                    if (x & (1 << oldbit)) != 0 {
                        self.sz_left -= 1;
                        return Some(unsplit_u64(x >> self.bits, oldbit, self.bits));
                    }
                }
                self.array = self.array.split_first().unwrap().1;
                self.whichbit = 0;
            }
        } else {
            if let Some((&first,rest)) = self.array.split_first() {
                self.array = rest;
                self.sz_left -= 1;
                return Some(first);
            }
        }
        None
    }
    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.array.into_iter().rev().cloned()
            .filter(|&x| x != 0)
            .map(|x| x >> self.bits
                 + (x & mask(self.bits as usize)).leading_zeros() as u64 - 63)
            .next()
    }
    #[inline]
    fn count(self) -> usize {
        self.sz_left
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz_left, Some(self.sz_left))
    }
    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        if self.sz_left == 0 {
            None
        } else if self.whichbit == 0 {
            let x = self.array.into_iter().cloned()
                .filter(|x| *x != 0).min().unwrap();
            Some((x >> self.bits)*self.bits + x.trailing_zeros() as u64)
        } else {
            let mut min = self.next().unwrap();
            while let Some(x) = self.next() {
                if x < min {
                    min = x;
                }
            }
            Some(min)
        }
    }
    #[inline]
    fn max(mut self) -> Option<Self::Item> {
        if self.sz_left == 0 {
            None
        } else if self.whichbit == 0 {
            let x = self.array.into_iter().cloned()
                .filter(|x| *x != 0).max().unwrap();
            let reference = (x >> self.bits)*self.bits;
            let m = mask(self.bits as usize);
            let extra = 63 - (x & m).leading_zeros() as u64;
            Some(reference + extra)
        } else {
            let mut max = self.next().unwrap();
            while let Some(x) = self.next() {
                if x > max {
                    max = x;
                }
            }
            Some(max)
        }
    }
}

#[derive(Debug)]
struct DenseIter<'a> {
    sz_left: usize,
    whichword: usize,
    whichbit: u64,
    a: &'a [u64],
}

impl<'a> Iterator for DenseIter<'a> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(word) = self.a.get(self.whichword) {
                while self.whichbit < 64 {
                    let bit = self.whichbit;
                    self.whichbit = 1 + bit;
                    if word & (1 << bit) != 0 {
                        self.sz_left -= 1;
                        return Some(((self.whichword as u64) << 6) + bit as u64);
                    }
                }
                self.whichbit = 0;
                self.whichword += 1;
            } else {
                return None;
            }
        }
    }
    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.sz_left == 0 {
            return None;
        }
        let zero_words = self.a.iter().rev().cloned()
            .take_while(|&x| x == 0).count() as u64;
        let zero_bits = self.a[self.a.len() - 1 - zero_words as usize].leading_zeros() as u64;
        Some(self.a.len() as u64*64 - zero_bits - 1 - zero_words*64)
    }
    #[inline]
    fn max(self) -> Option<Self::Item> {
        self.last()
    }
    #[inline]
    fn count(self) -> usize {
        self.sz_left
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz_left, Some(self.sz_left))
    }
    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl crate::copyset::CopySet for SetU64 {
    type Item = u64;
    type Iter = IntoIter;
    fn ins(&mut self, e: u64) -> bool {
        self.insert(e)
    }
    fn rem(&mut self, e: u64) -> bool {
        self.remove(e)
    }
    fn con(&self, e: u64) -> bool {
        self.contains(e)
    }
    fn vec(&self) -> Vec<u64> {
        self.iter().collect()
    }
    fn ln(&self) -> usize {
        self.len()
    }
    fn it(self) -> Self::Iter {
        self.into_iter()
    }
}

impl Default for SetU64 {
    fn default() -> Self {
        SetU64(0 as *mut S)
    }
}

impl std::iter::FromIterator<u64> for SetU64 {
    fn from_iter<T>(iter: T) -> Self
        where
        T: IntoIterator<Item = u64>
    {
        let mut v: Vec<_> = iter.into_iter().collect();
        v.sort();
        v.dedup();
        if let Some(mx) = v.iter().cloned().max() {
            if let Some(t) = Tiny::new_sorted_deduped(&v) {
                SetU64(t.to_usize() as *mut S)
            } else {
                if v.len() as u64 > mx >> 4 {
                    // This should be stored in a dense bitset.
                    let mut s = SetU64::with_capacity_and_max(v.len(), mx);
                    for value in v.into_iter() {
                        s.insert(value);
                    }
                    return s;
                }
                let bits = compute_array_bits(mx);
                if bits == 0 {
                    let mut s = SetU64::with_capacity_and_bits(v.len(), bits);
                    for value in v.into_iter() {
                        s.insert(value);
                    }
                    s
                } else {
                    let mut keys: Vec<_> = v.iter().map(|&x| x/bits).collect();
                    keys.sort();
                    keys.dedup();
                    let sz = (keys.len()+1)*11/10;
                    let mut s = SetU64::with_capacity_and_bits(sz, bits);
                    for value in v.into_iter() {
                        s.insert(value);
                    }
                    s
                }
            }
        } else {
            SetU64(0 as *mut S)
        }
    }
}

#[cfg(test)]
fn test_a_collect(v: Vec<u64>) {
    let s: SetU64 = v.iter().cloned().collect();
    let vv: Vec<_> = s.iter().collect();
    let ss: SetU64 = vv.iter().cloned().collect();
    let vvv: Vec<_> = ss.iter().collect();
    assert_eq!(vv, vvv);
}

#[test]
fn test_collect() {
    test_a_collect(vec![]);
    test_a_collect(vec![0]);
    test_a_collect(vec![0,1<<60]);
    test_a_collect(vec![0,1<<30,1<<60]);
    test_a_collect((0..1024).collect());
}

fn bytes_for_capacity(sz: usize) -> usize {
    sz*8+std::mem::size_of::<S>()-8
}
fn layout_for_capacity(sz: usize) -> std::alloc::Layout {
    unsafe {
        std::alloc::Layout::from_size_align_unchecked(bytes_for_capacity(sz), 8)
    }
}

impl Drop for SetU64 {
    fn drop(&mut self) {
        if self.0 as usize > 0 {
            // make it drop by moving it out
            let c = self.capacity();
            if c == 0 {
            } else {
                unsafe {
                    std::alloc::dealloc(self.0 as *mut u8, layout_for_capacity(c));
                }
            }
        }
    }
}

#[cfg(test)]
impl heapsize::HeapSizeOf for SetU64 {
    fn heap_size_of_children(&self) -> usize {
        match self.internal() {
            Internal::Empty => 0,
            Internal::Stack(_) => 0,
            Internal::Heap { a, .. } => {
                std::mem::size_of::<S>() - 8 + a.len()*8
            }
            Internal::Big { a, .. } => {
                std::mem::size_of::<S>() - 8 + a.len()*8
            }
            Internal::Dense { a, .. } => {
                std::mem::size_of::<S>() - 8 + a.len()*8
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapsize::HeapSizeOf;

    fn check_set(elems: &[u64]) {
        println!("\n\ncheck_set {:?}\n", elems);
        let mut s = SetU64::default();
        s.debug_me("default set");
        let mut count = 0;
        for x in elems.iter().cloned() {
            let was_here = s.contains(x);
            s.debug_me(&format!("\n\n\nabout to insert {}", x));
            let changed_something = s.insert(x);
            s.debug_me(&format!("\n\nafter inserting {}", x));
            if changed_something {
                count += 1;
                println!("    {} is new now count {}", x, count);
            }
            assert_eq!(!was_here, changed_something);
            s.debug_me(&format!("after inserting {} length is {}", x, s.len()));
            println!("what is this? count {} does it have {}?", count, x);
            assert!(s.contains(x));
            assert_eq!(s.len(), count);
            assert_eq!(s.iter().count(), count);
        }
        assert!(elems.len() >= s.len());
        assert_eq!(elems.iter().cloned().min(),
                   s.iter().min());
        println!("set {:?} with length {}", elems, s.len());
        for x in s.iter() {
            println!("    {}", x);
        }
        s.debug_me("finally");
        assert_eq!(s.iter().count(), s.len());
        for x in s.iter() {
            println!("looking for {}", x);
            assert!(elems.contains(&x));
        }
        for x in s.iter() {
            println!("found {}", x);
            assert!(elems.contains(&x));
        }
        for x in elems.iter().cloned() {
            println!("YYYY looking for {}", x);
            assert!(s.contains(x));
        }
        for x in elems.iter().cloned() {
            println!("removing {}", x);
            s.remove(x);
            s.debug_me("  after remove");
        }
        for x in elems.iter().cloned() {
            println!("XXXX looking for {}", x);
            assert!(!s.contains(x));
        }
        s.debug_me("after everything was removed");
        assert_eq!(s.len(), 0);
        check_size(elems);
    }

    #[test]
    fn check_sets() {
        check_set(&[2020521336280004635,
                    17919264261434241137,
                    2238430514865874295,
                    6993057942733118921,
                    151320868361192487]);


        check_set(&[1121917459475225854,
                    2080724155257001326,
                    4615424731220156355,
                    0]);

        check_set(&[12891372448885225674,
                    7003397808690412416,
                    129282323776365774,
                    9248739364838008708]);

        check_set(&[]);
        check_set(&[10]);
        check_set(&[1024]);
        check_set(&[1]);
        check_set(&[0]);
        check_set(&[0, 1]);
        check_set(&[0, 1]);

        check_set(&[0, 1, 2, 3]);
        check_set(&[0, 1, 0, 2, 3]);

        check_set(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        check_set(&[0, 1, 2, 3, 4, 5, 6, 7<<30, 8, 9, 10]);

        check_set(&[0, 2097153, 2, 2305843009213693952]);

        check_set(&[0, 1024]);
        check_set(&[0, 1024, 1 << 61]);
        check_set(&[0, 1024, 1 << 63]);
    }

    fn check_tiny_from_vec(slice: Vec<u64>) {
        println!("\n\n\ncheck_tiny_from_vec({:?})", slice);
        let mut t = if let Some(t) = Tiny::from_singleton(slice[0]) {
            t
        } else {
            return; // hokey, but avoids a crash
        };
        t.debug_me("to start with");
        let mut count = 1;
        let mut included = Vec::new();
        for x in slice.iter().cloned() {
            t.debug_me(&format!("starting in on {}", x));
            assert_eq!(t.clone().any(|e| e == x), t.contains(x));
            let already = t.clone().any(|e| e == x);
            let next = t.insert(x);
            if let Some(tt) = next {
                tt.debug_me("    inserting gives");
                assert_eq!(tt.sz == t.sz, already);
                if tt.sz != t.sz {
                    count += 1;
                    included.push(x);
                }
                t = tt;
                t.debug_me("hello");
                assert!(t.clone().any(|e| e == x));
                assert!(t.insert(x).is_some()); // inserting a second time must succeeed
                t.debug_me("hello again");
                assert_eq!(t.sz, count);
                for xx in t.clone() {
                    assert!(t.contains(xx));
                    assert!(t.clone().any(|x| x == xx));
                }
            } else {
                assert!(!already);
            }
            t.debug_me("at end of loop");
        }
        for x in included.iter().cloned() {
            assert!(t.contains(x));
        }
    }
    #[test]
    fn check_specific_tinies() {
        check_tiny_from_vec(vec![49,50,1]);
        check_tiny_from_vec(vec![1]);
        check_tiny_from_vec(vec![1,2]);
        check_tiny_from_vec(vec![1000000, 1000030]);
        check_tiny_from_vec(vec![3000027656, 3000030504]);
        check_tiny_from_vec(vec![3127827656, 3125730504]);
        check_tiny_from_vec(vec![3127827656, 3125730504,1]);
        check_tiny_from_vec(vec![2,3,143,251,1,130,251]);
        check_tiny_from_vec(vec![1,130,131,132,133,251]);
    }

    use proptest::prelude::*;
    proptest!{
        #[test]
        fn copycheck_random_sets(slice in prop::collection::vec(1u64..5, 1usize..10)) {
            crate::copyset::check_set::<SetU64>(&slice);
        }
        #[test]
        fn copycheck_medium_sets(slice in prop::collection::vec(1u64..255, 1usize..100)) {
            crate::copyset::check_set::<SetU64>(&slice);
        }
        #[test]
        fn copycheck_big_sets(slice: Vec<u64>) {
            crate::copyset::check_set::<SetU64>(&slice);
        }
    }
    proptest!{
        #[test]
        fn check_random_sets(slice in prop::collection::vec(1u64..5, 1usize..10)) {
            check_set(&slice);
        }
        #[test]
        fn check_medium_sets(slice in prop::collection::vec(1u64..255, 1usize..100)) {
            check_set(&slice);
        }
        #[test]
        fn check_big_sets(slice: Vec<u64>) {
            check_set(&slice);
        }
        #[test]
        fn check_tiny(slice in prop::collection::vec(1u64..0xffffffff, 1usize..9)) {
            check_tiny_from_vec(slice);
        }
    }
    #[test]
    fn check_specific_sets() {
        check_set(&[2847318633310315892, 63058418965769059, 2042910419467651321, 1999840121589118486, 16041957357413958548, 3644150915196633528, 11391567487966916668, 2789376712080388913, 8889702475440805467, 16888113214698725429, 1249634136756270040, 15461625332902556004, 8159161795026448273, 12009229139422836646, 15912096166435473807]);
    }

    fn total_size_of<T: HeapSizeOf>(x: &T) -> usize {
        std::mem::size_of::<T>() + x.heap_size_of_children()
    }

    fn check_size(v: &[u64]) {
        let s: SetU64 = v.iter().cloned().collect();
        let hs: std::collections::HashSet<_> = v.iter().cloned().collect();
        // let bs: std::collections::BTreeSet<_> = v.iter().cloned().collect();
        println!("setu64: {}", total_size_of(&s));
        println!("hashst: {}", total_size_of(&hs));
        // println!("btrees: {}", total_size_of(&bs));
        assert!(total_size_of(&s) < total_size_of(&hs));
        // assert!(total_size_of(&s) < total_size_of(&bs));
    }


    #[cfg(test)]
    fn collect_size_is(v: &[u64], sz: usize) {
        let s: SetU64 = v.iter().cloned().collect();
        println!("collect_size_is {:?} == {} =? {}", v, total_size_of(&s), sz);
        println!("    num elements = {}", s.len());
        assert_eq!(total_size_of(&s), sz);
    }

    #[cfg(test)]
    fn incremental_size_le(v: &[u64], sz: usize) {
        // repeat the size tests because our expansion is random in a
        // lame attempt to foil DOS attacks.
        for _ in 1..1000 {
            let sc: SetU64 = v.iter().cloned().collect();
            if total_size_of(&sc) > sz {
                println!("collect_size {:?} = {} > {}", v, total_size_of(&sc), sz);
            }
            assert!(total_size_of(&sc) <= sz);

            let mut s = SetU64::new();
            let mut vv = Vec::new();
            for x in v.iter().cloned() {
                s.insert(x);
                vv.push(x);
                if total_size_of(&s) > sz {
                    println!("incremental_size {:?} = {} > {}", vv, total_size_of(&s), sz);
                }
                assert!(total_size_of(&s) <= sz);
            }
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<S>(), 32);
        collect_size_is(&[0], 8);
        incremental_size_le(&[0], 8);
        collect_size_is(&[], 8);
        incremental_size_le(&[], 8);
        collect_size_is(&[3000], 8);
        collect_size_is(&[1,2,3,4,5,6], 8);
        incremental_size_le(&[1,2,3,4,5,6], 8);
        collect_size_is(&[1000,1002,1004,1006,1008], 8);
        incremental_size_le(&[1000,1002,1004,1006,1008], 8);
        collect_size_is(&[255,260,265,270,275,280,285], 8);
        collect_size_is(&[1000,1002,1004,1006,1008,1009,1010], 8);

        incremental_size_le(& (0..7).collect::<Vec<_>>(), 8);
        incremental_size_le(& (10..10+7).collect::<Vec<_>>(), 8);
        incremental_size_le(& (100..100+7).collect::<Vec<_>>(), 8);
        incremental_size_le(& (1000..1000+7).collect::<Vec<_>>(), 8);
        incremental_size_le(& (10000..10000+7).collect::<Vec<_>>(), 8);
        incremental_size_le(& (100000..100000+7).collect::<Vec<_>>(), 8);

        incremental_size_le(& (1..30).collect::<Vec<_>>(), 40);
        incremental_size_le(& (1..60).collect::<Vec<_>>(), 40);

        incremental_size_le(& (1..160).collect::<Vec<_>>(), 88);

        incremental_size_le(& (0..100).map(|x| x*10).collect::<Vec<_>>(), 400);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<S>(), 24);
        collect_size_is(&[0], 4);
        incremental_size_le(&[0], 4);
        collect_size_is(&[], 4);
        incremental_size_le(&[], 4);
        collect_size_is(&[3000], 4);
        collect_size_is(&[1,2,3,4,5,6], 4);
        incremental_size_le(&[1,2,3,4,5,6], 4);
        collect_size_is(&[1000,1002,1004,1006,1008], 4);
        incremental_size_le(&[1000,1002,1004,1006,1008], 4);

        collect_size_is(&[255,260,265,270,275,280,285], 4);

        incremental_size_le(& (1..30).collect::<Vec<_>>(), 32);
        incremental_size_le(& (1..60).collect::<Vec<_>>(), 32);

        incremental_size_le(& (1..160).collect::<Vec<_>>(), 84);

        incremental_size_le(& (0..100).map(|x| x*10).collect::<Vec<_>>(), 400);
    }

    fn check_set_primitives(elems: &[u64]) {
        let sz = elems.len();
        println!("\n\nprimitives: {:?}\n", elems);
        let mut a = vec![0; sz];
        for x in elems.iter().cloned() {
            if p_lookfor(x, &a, 0).key_found() {
                println!("we already have {}", x);
            } else {
                let i = p_insert(x, &mut a, 0);
                a[i] = x;
            }
            println!("    after inserting {} we have {:?}", x, a);
            assert!(p_lookfor(x, &a, 0).key_found());
        }
        for x in elems.iter().cloned() {
            println!("looking for {}", x);
            assert!(p_lookfor(x, &a, 0).key_found());
        }
        for x in elems.iter().cloned() {
            println!("removing {}", x);
            p_remove(x, &mut a, 0);
            println!("    after removing {} we have {:?}", x, a);
        }
        for x in elems.iter().cloned() {
            println!("XXXX looking for {}", x);
            assert!(p_lookfor(x, &a, 0).empty_spot());
        }
        println!("after everything was removed: {:?}", a);
        for x in a.iter().cloned() {
            assert_eq!(x, 0);
        }
    }

    proptest!{
        #[test]
        fn check_random_primitives(slice in prop::collection::vec(1u64..50, 1usize..100)) {
            check_set_primitives(&slice);
        }
    }

}

fn p_poverty(k: u64, idx: usize, n: usize) -> usize {
    ((idx % n) + n - (k % n as u64) as usize) % n
}

/// This inserts k into the array, and requires that there be room for
/// one more element.  Otherwise, things will be sad.
fn p_insert(k: u64, a: &mut [u64], offset: u64) -> usize {
    let n = a.len();
    for pov in 0..n {
        let ii = ((k + pov as u64) % n as u64) as usize;
        let ki = a[ii] >> offset;
        let pov_ki = p_poverty(ki, ii, n);
        if a[ii] == 0 || ki == k {
            // println!("already got a spot");
            return ii;
        } else if pov_ki < pov {
            // println!("need to steal from {} < {} at spot {}", pov_ki, pov, ii);
            // need to steal
            let stolen = ii;
            let mut displaced = a[ii];
            // println!("displaced value is {}", displaced);
            let mut pov_displaced = pov_ki;
            a[stolen] = 0;

            for j in 1..n {
                pov_displaced += 1;
                let jj = (stolen + j) % n;
                let kj = a[jj] >> offset;
                let pov_kj = p_poverty(kj, jj, n);
                if a[jj] == 0 {
                    // We finally found an unoccupied spot!
                    // println!("put the displaced at {}", jj);
                    a[jj] = displaced;
                    return stolen;
                }
                if pov_kj < pov_displaced {
                    // need to steal again!
                    std::mem::swap(&mut a[jj], &mut displaced);
                    pov_displaced = pov_kj;
                }
            }
            panic!("p_insert was called when there was no room!")
        }
    }
    unreachable!()
}

#[test]
fn test_insert() {
    let mut a = [0,0,0,0];
    assert_eq!(2, p_insert(2, &mut a, 0));
    assert_eq!(&a, &[0,0,0,0]);
    for i in 0..10 {
        assert_eq!(0, a[p_insert(i, &mut a, 0)]);
    }

    let mut a = [0,0,6,0];
    assert_eq!(3, p_insert(2, &mut a, 0));
    assert_eq!(&a, &[0,0,6,0]);
    for i in 0..10 {
        assert!([0,i].contains(&a[p_insert(i, &mut a, 0)]));
    }

    let mut a = [0,0,6,3];
    assert_eq!(3, p_insert(2, &mut a, 0));
    assert_eq!(&a, &[3,0,6,0]);
    for i in 0..10 {
        assert!([0,i].contains(&a[p_insert(i, &mut a, 0)]));
    }
}

#[derive(Debug,Eq,PartialEq,Clone,Copy)]
enum LookedUp {
    EmptySpot(usize),
    KeyFound(usize),
    NeedInsert,
}
impl LookedUp {
    fn key_found(self) -> bool {
        if let LookedUp::KeyFound(_) = self {
            true
        } else {
            false
        }
    }
    #[cfg(test)]
    fn empty_spot(self) -> bool {
        if let LookedUp::EmptySpot(_) = self {
            true
        } else {
            false
        }
    }
    #[cfg(test)]
    fn unwrap(self) -> usize {
        if let LookedUp::KeyFound(idx) = self {
            idx
        } else {
            panic!("unwrap called on {:?}", self)
        }
    }
}

fn p_lookfor(k: u64, a: &[u64], offset: u64) -> LookedUp {
    let n = a.len();
    for pov in 0..n {
        let ii = ((k + pov as u64) % n as u64) as usize;
        // println!("looking in spot ii = {} with pov={}", ii, pov);
        if a[ii] == 0 {
            // println!("got empty spot at {} for key {}", ii, k);
            return LookedUp::EmptySpot(ii);
        }
        let ki = a[ii] >> offset;
        let pov_ki = p_poverty(ki, ii, n);
        if ki == k {
            // println!("lookfor already got a spot");
            return LookedUp::KeyFound(ii);
        } else if pov_ki < pov {
            // println!("at index {} we have {} > {}", ii, pov, pov_ki);
            return LookedUp::NeedInsert;
        }
    }
    LookedUp::NeedInsert
}

#[test]
fn test_lookfor() {
    assert_eq!(LookedUp::NeedInsert, p_lookfor(5, &[3,1,2], 0));
    assert_eq!(LookedUp::NeedInsert, p_lookfor(5, &[3,0,2], 0));
    assert_eq!(LookedUp::KeyFound(3), p_lookfor(7, &[0,0,0,7], 0));
}

fn p_remove(k: u64, a: &mut [u64], offset: u64) -> bool {
    let n = a.len();
    for i in 0..n {
        let ii = ((k + i as u64) % n as u64) as usize;
        // println!("    looking to remove at distance {} slot {}", i, ii);
        if a[ii] == 0 {
            return false;
        }
        let ki = a[ii] >> offset;
        let iki = (((ii + n) as u64 - (ki % n as u64)) % n as u64) as usize;
        if i > iki {
            return false;
        } else if ki == k {
            // println!("found {} at location {}", k, ii);
            a[ii] = 0;
            // Now we need to return anything that might have been
            // stolen from... to massacre my grammar.
            let mut previous = ii;
            for j in 1..n {
                let jj = (ii + j) % n;
                // println!("looking at removing offset {} at location {}", j, jj);
                let kj = a[jj] >> offset;
                let pov_kj = p_poverty(kj, jj, n);
                if a[jj] == 0 || pov_kj == 0 {
                    // We found an unoccupied spot or a perfectly
                    // happy customer, so nothing else could have been
                    // bumped.
                    return true;
                }
                // need to undo some stealing!
                a[previous] = a[jj];
                a[jj] = 0;
                previous = jj;
            }
            return true;
        }
    }
    panic!("bug: we should have had space in {:?} for {}", a, k)
}

#[cfg(test)]
fn test_insert_remove(x: u64, a: &mut [u64]) {
    println!("test_insert_remove({}, {:?})", x, a);
    let v: Vec<u64> = a.iter().cloned().collect();
    assert!(!p_remove(x, a, 0));
    assert!(!a.contains(&x)); // otherwise the test won't work right.
    assert!(!p_lookfor(x, a, 0).key_found());
    a[p_insert(x, a, 0)] = x;
    assert!(a.contains(&x));
    println!("  after insertion of {} a is {:?}", x, a);
    assert!(p_lookfor(x, a, 0).key_found());
    assert_eq!(x, a[p_lookfor(x, a, 0).unwrap()]);
    assert!(p_remove(x, a, 0));
    println!("  after remove of {} a is {:?}", x, a);
    assert_eq!(a, &v[..]);
}

#[test]
fn test_remove() {
    let mut a = [0,0,2];
    a[p_insert(5,&mut a,0)] = 5;
    assert_eq!(&[5,0,2], &a);
    p_remove(2,&mut a, 0);
    println!("after removal {:?}", a);
    assert!(p_lookfor(5, &a, 0).key_found());
    assert_eq!(&[0,0,5], &a);

    test_insert_remove(7,&mut [0,0,0,0]);

    test_insert_remove(2,&mut [0,1,5,0]);

    test_insert_remove(5,&mut [0,0,2]);
}
