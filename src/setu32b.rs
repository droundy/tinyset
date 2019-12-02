//! This is a an awesome module.

#[cfg(test)]
use proptest::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tiny {
    start: u32,
    bits: usize,
}

impl Iterator for Tiny {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.bits != 0 {
            let off = self.bits.trailing_zeros();
            let v = self.start + off;
            self.bits = self.bits >> (off + 1);
            self.start += off + 1;
            Some(v)
        } else {
            None
        }
    }
    fn count(self) -> usize {
        self.bits.count_ones() as usize
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.bits.count_ones()  as usize, Some(self.bits.count_ones() as usize))
    }
    fn min(self) -> Option<u32> {
        Some(self.start + self.bits.trailing_zeros())
    }
    fn max(self) -> Option<u32> {
        println!("leading zeros {}", self.bits.leading_zeros());
        Some(self.start + 64 - self.bits.leading_zeros())
    }
}

impl Tiny {
    #[cfg(target_pointer_width = "64")]
    fn to_usize(self) -> usize {
        (self.start as usize) << 32 | self.bits << 2 | 1
    }
    #[cfg(target_pointer_width = "64")]
    fn from_usize(x: usize) -> Self {
        Tiny {
            start: (x >> 32) as u32,
            bits: (x >> 2) & ((1 << 30) - 1),
        }
    }
    fn from_singleton(x: u32) -> Option<Self> {
        Some(Tiny { start: x, bits: 1 })
    }
    fn from_slice(v: &[u32]) -> Option<Self> {
        if v.len() > 30 || v.len() == 0 {
            return None;
        }
        let mn = v.iter().cloned().min().unwrap();
        let mx = v.iter().cloned().max().unwrap();
        if mx > mn + 30 {
            None
        } else {
            let mut t = Tiny {
                start: mn,
                bits: 0,
            };
            for x in v.iter().cloned() {
                t.bits = t.bits | (1 << x - mn);
            }
            Some(t)
        }
    }
    fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
    fn contains(&self, v: u32) -> bool {
        v >= self.start && v <= self.start + 30 && self.bits >> (v-self.start) & 1 != 0
    }
    fn insert(&mut self, v: u32) -> Option<bool> {
        if v >= self.start && v <= self.start + 30 {
            if self.bits >> (v-self.start) & 1 != 0 {
                return Some(true);
            }
            self.bits = self.bits | 1 << (v-self.start);
            return Some(false)
        }
        let mx = self.clone().max().unwrap();
        let mn = self.clone().min().unwrap();
        if v + 30 < mx || v > mn + 30 {
            None
        } else if v < mn {
            self.bits = self.bits << (self.start-v) | 1;
            self.start = v;
            Some(false)
        } else if self.start < mn {
            self.bits = self.bits >> (mn - self.start) | 1 << (v-mn);
            self.start = mn;
            Some(false)
        } else {
            self.bits = self.bits | 1 << (v-self.start);
            Some(false)
        }
    }
    fn remove(&mut self, v: u32) -> bool {
        if self.contains(v) {
            self.bits = self.bits & !(1 << v-self.start);
            true
        } else {
            false
        }
    }
}

#[test]
fn check_tiny_insert() {
    let mut t = Tiny::from_singleton(0).unwrap();
    println!("starting with {:?}", t.clone().collect::<Vec<_>>());
    for v in [0,1,1,2,29,30].into_iter().cloned() {
        assert_eq!(Some(t.contains(v)), t.insert(v));
        println!(" after inserting {}: {:?}", v, t.clone().collect::<Vec<_>>());
        assert!(t.contains(v));
    }

    for v in [0,30,2,29,1].into_iter().cloned() {
        assert!(t.contains(v));
        assert!(t.remove(v));
        assert!(!t.remove(v));
        assert!(!t.contains(v));
    }
    assert_eq!(t.len(), 0);

    let mut t = Tiny::from_singleton(50).unwrap();
    println!("starting with {:?}", t.clone().collect::<Vec<_>>());
    for v in [49,40,30,21].into_iter().cloned() {
        assert_eq!(Some(t.contains(v)), t.insert(v));
        println!(" after inserting {}: {:?}", v, t.clone().collect::<Vec<_>>());
        assert!(t.contains(v));
    }
    for v in [49,40,30,21].into_iter().cloned() {
        println!("removing {} from {:?}", v, t.clone().collect::<Vec<_>>());
        assert!(t.contains(v));
        assert!(t.remove(v));
        assert!(!t.remove(v));
        assert!(!t.contains(v));
    }
}
#[cfg(test)]
proptest!{
    #[test]
    fn check_tiny_from_slice(v in prop::collection::vec(0..36u32, 0usize..34)) {
        if let Some(t) = Tiny::from_slice(&v) {
            for x in v.iter().cloned() {
                assert!(t.contains(x));
                assert_eq!(t.contains(x+1), v.contains(&(x+1)));
            }
        } else {
            prop_assume!(false); // do not count cases that cannot be generated
        }
    }
    #[test]
    fn check_tiny_insert_remove(x: Vec<Result<u32,u32>>) {
        let mut t = Tiny::from_singleton(0).unwrap();
        for job in x.iter().cloned() {
            match job {
                Ok(v) => {
                    if t.contains(v) {
                        assert_eq!(t.insert(v), Some(true));
                    } else {
                        if t.insert(v).is_some() {
                            assert!(t.contains(v));
                        }
                    }
                }
                Err(v) => {
                    assert_eq!(t.contains(v), t.remove(v));
                    assert!(!t.contains(v));
                }
            }
        }
    }
    #[test]
    fn check_tiny_from_singleton(x: u32) {
        let t = Tiny::from_singleton(x).unwrap();
        assert_eq!(t.clone().next(), Some(x));
        assert_eq!(t.clone().count(), 1);
        assert!(t.contains(x));
        assert!(!t.contains(x+1));
        assert_eq!(t.len(), 1);
    }
    #[test]
    fn check_tiny_from_inserts(x0: u32,
                               vals in prop::collection::vec(0..28u32, 1usize..10)) {
        let mut t = Tiny::from_singleton(x0).unwrap();
        for v in vals.iter().cloned().map(|v| v+x0) {
            assert_eq!(Some(t.contains(v)), t.insert(v));
            assert!(t.contains(v));
        }
    }
    #[test]
    fn check_to_from_tiny(x: usize) {
        prop_assume!(x & 3 != 0);
        let tiny = Tiny::from_usize(x);
        let x = tiny.to_usize();
        println!();
        println!("   x:   {:b}", x);
        println!("tiny: {:b} {:b}", tiny.start, tiny.bits);
        println!(" new: {:b} {:b}",
                 Tiny::from_usize(x).start,
                 Tiny::from_usize(x).bits);
        assert_eq!(x, Tiny::from_usize(x).to_usize());
    }
}

/// A set of u32
pub struct SetU32(*mut S);

#[repr(C)]
#[derive(Debug)]
struct S {
    sz: u32,
    cap: u32,
    array: u32,
}

enum Internal<'a> {
    Empty,
    Tiny(Tiny),
    Table {
        sz: u32,
        a: &'a [(u32,u32)],
    },
    Dense {
        sz: u32,
        a: &'a [u32],
    },
}
enum InternalMut<'a> {
    Empty,
    Tiny(Tiny),
    Table {
        sz: &'a mut u32,
        a: &'a mut [(u32,u32)],
    },
    Dense {
        sz: &'a mut u32,
        a: &'a mut [u32],
    },
}

impl SetU32 {
    fn internal<'a>(&'a self) -> Internal<'a> {
        if self.0 as usize == 0 {
            Internal::Empty
        } else if self.0 as usize & 3 == 1 {
            Internal::Tiny(Tiny::from_usize(self.0 as usize))
        } else if self.0 as usize & 3 == 0 {
            let s = unsafe { &*self.0 };
            let a = unsafe { std::slice::from_raw_parts((&s.array as *const u32) as *const (u32,u32),
                                                        s.cap as usize) };
            Internal::Table { sz: s.sz, a }
        } else {
            let ptr = (self.0 as usize & !3) as *mut S;
            let s = unsafe { &*ptr };
            let a = unsafe { std::slice::from_raw_parts(&s.array as *const u32,
                                                        s.cap as usize) };
            Internal::Dense { sz: s.sz, a }
        }
    }

    fn internal_mut<'a>(&'a mut self) -> InternalMut<'a> {
        if self.0 as usize == 0 {
            InternalMut::Empty
        } else if self.0 as usize & 3 == 1 {
            InternalMut::Tiny(Tiny::from_usize(self.0 as usize))
        } else if self.0 as usize & 3 == 0 {
            let s = unsafe { &mut *self.0 };
            let a = unsafe {
                std::slice::from_raw_parts_mut((&mut s.array as *mut u32) as *mut (u32,u32),
                                               s.cap as usize)
            };
            InternalMut::Table { sz: &mut s.sz, a }
        } else {
            let ptr = (self.0 as usize & !3) as *mut S;
            let s = unsafe { &mut *ptr };
            let a = unsafe { std::slice::from_raw_parts_mut(&mut s.array as *mut u32,
                                                            s.cap as usize) };
            InternalMut::Dense { sz: &mut s.sz, a }
        }
    }

    fn num_u32(&self) -> u32 {
        match self.internal() {
            Internal::Table { a, .. } => a.len() as u32*2,
            Internal::Dense { a, .. } => a.len() as u32,
            _ => 0,
        }
    }

    /// The number of elements in the set.
    pub fn len(&self) -> usize {
        match self.internal() {
            Internal::Table { sz, .. } => sz as usize,
            Internal::Dense { sz, .. } => sz as usize,
            Internal::Empty => 0,
            Internal::Tiny(t) => t.len(),
        }
    }

    /// Create a new empty set.
    ///
    /// This does no heap allocation.
    pub const fn new() -> Self {
        SetU32(0 as *mut S)
    }

    /// Check if the set contains `e`.
    pub fn contains(&self, e: u32) -> bool {
        match self.internal() {
            Internal::Empty => false,
            Internal::Tiny(t) => t.contains(e),
            Internal::Dense { a, .. } => {
                let key = e as usize >> 5;
                let bit = 1 << (e & 31);
                if a.len() > key {
                    a[key] & bit != 0
                } else {
                    false
                }
            }
            Internal::Table { .. } => {
                unimplemented!()
            }
        }
    }

    /// Insert a number into the set.
    ///
    /// Return a bool indicating if it was already present.
    pub fn insert(&mut self, e: u32) -> bool {
        match self.internal_mut() {
            InternalMut::Empty => {
                if let Some(t) = Tiny::from_singleton(e) {
                    *self = SetU32::tiny(t);
                } else if e < 64 {
                    *self = SetU32::dense_for_mx(e+1);
                    self.insert(e);
                } else {
                    *self = SetU32::table_with_cap(1);
                    self.insert(e);
                }
                false
            }
            InternalMut::Tiny(mut t) => {
                if let Some(b) = t.insert(e) {
                    *self = SetU32::tiny(t);
                    b
                } else {
                    *self = SetU32::table_with_cap(3);
                    for x in t {
                        self.insert(x);
                    }
                    self.insert(e);
                    false
                }
            }
            InternalMut::Dense { sz, a } => {
                let key = e as usize >> 5;
                let bit = 1 << (e & 31);
                if a.len() > key {
                    let was_here = a[key] & bit != 0;
                    a[key] = a[key] | bit;
                    was_here
                } else {
                    if key > 3*(*sz as usize) {
                        // It is getting sparse, so let us switch back
                        // to a non-hash table.
                        let mut new = SetU32::table_with_cap(2*(*sz + 1));
                        for x in DenseIter::new( *sz as usize, a ) {
                            new.insert(x);
                        }
                        new.insert(e);
                        *self = new;
                    } else {
                        unsafe { self.dense_increase_mx(key as u32)[key] = bit };
                    }
                    false
                }
            }
            InternalMut::Table { .. } => {
                unimplemented!()
            }
        }
    }

    fn tiny(t: Tiny) -> Self {
        SetU32(t.to_usize() as *mut S)
    }

    fn dense_for_mx(mx: u32) -> Self {
        let n = 1 + mx/32 + mx/128;
        unsafe {
            let x = SetU32(std::alloc::alloc_zeroed(layout_for_num_u32(n)) as *mut S);
            (*x.0).cap = n;
            x
        }
    }

    /// This requires that we currently be a dense!
    unsafe fn dense_increase_mx(&mut self, mx: u32) -> &mut [u32] {
        let ptr = (self.0 as usize & !3) as *mut S;
        let n = 1 + mx/32 + mx/128;

        let oldcap = (*ptr).cap;
        let newptr = std::alloc::realloc(ptr as *mut u8,
                                         layout_for_num_u32(oldcap),
                                         bytes_for_num_u32(n));
        if newptr as usize == 0 {
            println!("unable to realloc from {} to {}", oldcap, n);
            std::alloc::handle_alloc_error(layout_for_num_u32(n));
        }
        let x = SetU32((newptr as usize | 2) as *mut S);
        (*x.0).cap = n;
        *self = x;
        match self.internal_mut() {
            InternalMut::Dense { a, .. } => {
                for i in oldcap as usize .. a.len() {
                    a[i] = 0;
                }
                a
            }
            _ => unreachable!(),
        }
    }

    fn table_with_cap(cap: u32) -> Self {
        unsafe {
            let x = SetU32(std::alloc::alloc_zeroed(layout_for_num_u32(cap)) as *mut S);
            (*x.0).cap = cap;
            x
        }
    }
}

impl Default for SetU32 {
    fn default() -> Self {
        SetU32(0 as *mut S)
    }
}

const fn bytes_for_num_u32(sz: u32) -> usize {
    sz as usize*4+std::mem::size_of::<S>()-4
}

fn layout_for_num_u32(sz: u32) -> std::alloc::Layout {
    unsafe {
        std::alloc::Layout::from_size_align_unchecked(bytes_for_num_u32(sz), 4)
    }
}

impl Drop for SetU32 {
    fn drop(&mut self) {
        if self.0 as usize != 0 && self.0 as usize & 3 != 1 {
            // make it drop by moving it out
            let n = self.num_u32();
            if n != 0 {
                unsafe {
                    std::alloc::dealloc(self.0 as *mut u8, layout_for_num_u32(n));
                }
            }
        }
    }
}

#[derive(Debug)]
enum Iter<'a> {
    Tiny(Tiny),
    Dense(DenseIter<'a>),
}
impl<'a> Iterator for Iter<'a> {
    type Item = u32;
    #[inline]
    fn next(&mut self) -> Option<u32> {
        match self {
            Iter::Tiny(t) => t.next(),
            Iter::Dense(d) => d.next(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        match self {
            Iter::Tiny(t) => t.count(),
            Iter::Dense(d) => d.count(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Tiny(t) => t.size_hint(),
            Iter::Dense(d) => d.size_hint(),
        }
    }
    #[inline]
    fn min(mut self) -> Option<u32> {
        match self {
            Iter::Tiny(t) => t.min(),
            Iter::Dense(d) => d.min(),
        }
    }
    #[inline]
    fn max(self) -> Option<u32> {
        match self {
            Iter::Tiny(t) => t.max(),
            Iter::Dense(d) => d.max(),
        }
    }
    #[inline]
    fn last(self) -> Option<u32> {
        match self {
            Iter::Tiny(t) => t.last(),
            Iter::Dense(d) => d.last(),
        }
    }
}

#[derive(Debug)]
struct DenseIter<'a> {
    sz_left: usize,
    whichword: usize,
    whichbit: u32,
    a: &'a [u32],
}
impl<'a> DenseIter<'a> {
    fn new(sz_left: usize, a: &'a [u32]) -> Self {
        DenseIter {
            sz_left,
            whichword: 0,
            whichbit: 0,
            a,
        }
    }
}
impl<'a> Iterator for DenseIter<'a> {
    type Item = u32;
    #[inline]
    fn next(&mut self) -> Option<u32> {
        loop {
            if let Some(word) = self.a.get(self.whichword) {
                if *word != 0 {
                    while self.whichbit < 32 {
                        let bit = self.whichbit;
                        self.whichbit += 1;
                        if word & (1 << bit) != 0 {
                            self.sz_left -= 1;
                            return Some(((self.whichword as u32) << 5) + bit as u32);
                        }
                    }
                }
                self.whichword += 1;
                self.whichbit = 0;
            } else {
                return None;
            }
        }
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
    fn min(mut self) -> Option<u32> {
        self.next()
    }
}

#[test]
fn test_denseiter() {
    let v: Vec<u32> = DenseIter::new(5, &[1,1,1,0,1,0,2]).collect();
    assert_eq!(&v, &[0, 32, 64, 128, 193]);

    assert_eq!(Some(0), DenseIter::new(5, &[1,1,1,0,1,0,2]).min());

    assert_eq!(Some(34), DenseIter::new(5, &[0,4,1,0,1,0,2]).min());
}
