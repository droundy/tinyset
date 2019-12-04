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
    #[cfg(target_pointer_width = "64")]
    fn max(self) -> Option<u32> {
        Some(self.start + 63 - self.bits.leading_zeros())
    }
    #[cfg(target_pointer_width = "32")]
    fn max(self) -> Option<u32> {
        Some(self.start + 31 - self.bits.leading_zeros())
    }
}

#[cfg(target_pointer_width = "64")]
const MAX_TINY: u32 = std::u32::MAX;
#[cfg(target_pointer_width = "32")]
const MAX_TINY: u32 = std::u16::MAX as u32;

#[cfg(target_pointer_width = "64")]
const START_OFFSET: u32 = 32;
#[cfg(target_pointer_width = "32")]
const START_OFFSET: u32 = 16;

const NUM_BITS: u32 = START_OFFSET-2;

impl Tiny {
    fn to_usize(self) -> usize {
        (self.start as usize) << START_OFFSET | self.bits << 2 | 1
    }
    fn from_usize(x: usize) -> Self {
        Tiny {
            start: (x >> START_OFFSET) as u32,
            bits: (x >> 2) & ((1 << NUM_BITS) - 1),
        }
    }
    fn from_singleton(x: u32) -> Option<Self> {
        if x > MAX_TINY {
            None
        } else {
            Some(Tiny { start: x, bits: 1 })
        }
    }
    fn from_slice(v: &[u32]) -> Option<Self> {
        if v.len() > NUM_BITS as usize || v.len() == 0 {
            return None;
        }
        let mn = v.iter().cloned().min().unwrap();
        let mx = v.iter().cloned().max().unwrap();
        if mx > mn + NUM_BITS || mn > MAX_TINY {
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
        v >= self.start && v <= self.start + NUM_BITS && self.bits >> (v-self.start) & 1 != 0
    }
    fn insert(&mut self, v: u32) -> Option<bool> {
        if v >= self.start && v <= self.start + NUM_BITS {
            if self.bits >> (v-self.start) & 1 != 0 {
                return Some(true);
            }
            self.bits = self.bits | 1 << (v-self.start);
            return Some(false)
        }
        let mx = self.clone().max().unwrap();
        let mn = self.clone().min().unwrap();
        if v + NUM_BITS < mx || v > mn + NUM_BITS {
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

#[cfg(target_pointer_width = "64")]
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
    fn check_tiny_from_slice(v in prop::collection::vec(0..NUM_BITS+4, 0usize..34)) {
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
    fn check_tiny_from_singleton(x in 0..=MAX_TINY) {
        let t = Tiny::from_singleton(x).unwrap();
        assert_eq!(t.clone().next(), Some(x));
        assert_eq!(t.clone().count(), 1);
        assert!(t.contains(x));
        assert!(!t.contains(x+1));
        assert_eq!(t.len(), 1);
    }
    #[test]
    fn check_tiny_from_inserts(x0 in 0..=MAX_TINY,
                               vals in prop::collection::vec(0..NUM_BITS, 1usize..10)) {
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
                                                        s.cap as usize/2) };
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
                                               s.cap as usize/2)
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

    /// Iterate over the set
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=u32> + 'a {
        match self.internal() {
            Internal::Empty => Iter::Empty,
            Internal::Tiny(t) => Iter::Tiny( t ),
            Internal::Table { .. } => {
                unimplemented!()
                // Iter::Table {
                //     sz_left: sz as usize,
                //     bits: s.bits,
                //     whichbit: 0,
                //     array: a,
                // }
            }
            Internal::Dense { a, sz } => {
                Iter::Dense( DenseIter::new(sz as usize, a) )
            }
        }
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

    /// Remove a number from the set.
    ///
    /// Return a bool indicating if it was present.
    pub fn remove(&mut self, e: u32) -> bool {
        match self.internal_mut() {
            InternalMut::Empty => false,
            InternalMut::Tiny(mut t) => {
                let b = t.remove(e);
                *self = SetU32(t.to_usize() as *mut S);
                b
            }
            InternalMut::Dense { a, sz } => {
                let key = e as usize >> 5;
                let bit = 1 << (e & 31);
                if a.len() > key {
                    if a[key] & bit != 0 {
                        a[key] = a[key] & !bit;
                        *sz -= 1;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            InternalMut::Table { .. } => {
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
                } else if (e as usize) < (t.len() + 1)*64 {
                    println!("creating a dense thing");
                    *self = SetU32::dense_for_mx(e+1);
                    println!("foo is {:?}", self.0);
                    for x in t {
                        self.insert(x);
                    }
                    self.insert(e);
                    false
                } else {
                    println!("looks like {} > {}", e, (t.len()+1)*64);
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
                    if !was_here {
                        *sz += 1;
                    }
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
                        *sz += 1;
                        unsafe { self.dense_increase_mx(e)[key] = bit };
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
            let newptr = std::alloc::alloc_zeroed(layout_for_num_u32(n)) as *mut S;
            (*newptr).cap = n;
            SetU32((newptr as usize | 2) as *mut S)
        }
    }

    /// This requires that we currently be a dense!
    unsafe fn dense_increase_mx(&mut self, mx: u32) -> &mut [u32] {
        let ptr = (self.0 as usize & !3) as *mut S;
        let n = 1 + mx/32 + mx/128;

        let oldcap = (*ptr).cap;
        let newptr = std::alloc::realloc(ptr as *mut u8,
                                         layout_for_num_u32(oldcap),
                                         bytes_for_num_u32(n)) as *mut S;
        if newptr as usize == 0 {
            println!("unable to realloc from {} to {}", oldcap, n);
            std::alloc::handle_alloc_error(layout_for_num_u32(n));
        }
        (*newptr).cap = n;
        self.0 = (newptr as usize | 2) as *mut S;
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
            let n = self.num_u32();
            if n != 0 {
                let ptr = (self.0 as usize & !3) as *mut u8;
                unsafe {
                    std::alloc::dealloc(ptr, layout_for_num_u32(n));
                }
            }
        }
    }
}

#[test]
fn test_insert_remove() {
    let mut v: Vec<u32> = Vec::new();
    let mut s = SetU32::new();
    assert_eq!(&v, &s.iter().collect::<Vec<u32>>());
    assert!(!s.insert(5));
    v.push(5);
    assert_eq!(&v, &s.iter().collect::<Vec<u32>>());

    assert_eq!(v.len(), s.iter().count());
    assert_eq!(v.len(), s.len());
    assert_eq!(v.iter().cloned().max(), s.iter().max());
    assert_eq!(v.iter().cloned().min(), s.iter().min());

    assert!(!s.insert(10));
    assert!(s.insert(10));
    v.push(10);
    assert_eq!(&v, &s.iter().collect::<Vec<u32>>());

    assert!(!s.insert(30));
    v.push(30);
    assert_eq!(&v, &s.iter().collect::<Vec<u32>>());
    assert_eq!(v.len(), s.iter().count());
    assert_eq!(v.len(), s.len());
    assert_eq!(v.iter().cloned().max(), s.iter().max());
    assert_eq!(v.iter().cloned().min(), s.iter().min());

    println!("inserting 40");
    assert!(!s.insert(40));
    v.push(40);
    assert_eq!(&v, &s.iter().collect::<Vec<u32>>());
    assert_eq!(v.len(), s.iter().count());
    assert_eq!(v.len(), s.len());
    assert_eq!(v.iter().cloned().max(), s.iter().max());
    assert_eq!(v.iter().cloned().min(), s.iter().min());

    println!("inserting 50");
    assert!(!s.insert(50));
    v.push(50);
    assert_eq!(&v, &s.iter().collect::<Vec<u32>>());
    assert_eq!(v.len(), s.iter().count());
    assert_eq!(v.len(), s.len());
    assert_eq!(v.iter().cloned().max(), s.iter().max());
    assert_eq!(v.iter().cloned().min(), s.iter().min());

    println!("removing 40");
    assert!(s.remove(40));
    v.retain(|x| *x != 40);
    assert_eq!(&v, &s.iter().collect::<Vec<u32>>());
    assert_eq!(v.len(), s.iter().count());
    assert_eq!(v.len(), s.len());
    assert_eq!(v.iter().cloned().max(), s.iter().max());
    assert_eq!(v.iter().cloned().min(), s.iter().min());

    for i in 0..500 {
        let n = i*2;
        println!("inserting {}", n);
        s.insert(n);
        v.push(n);
        v.sort();
        v.dedup();
        assert_eq!(&v, &s.iter().collect::<Vec<u32>>());
        assert_eq!(v.len(), s.iter().count());
        assert_eq!(v.len(), s.len());
        assert_eq!(v.iter().cloned().max(), s.iter().max());
        assert_eq!(v.iter().cloned().min(), s.iter().min());
    }

}

#[derive(Debug)]
enum Iter<'a> {
    Empty,
    Tiny(Tiny),
    Dense(DenseIter<'a>),
}
impl<'a> Iterator for Iter<'a> {
    type Item = u32;
    #[inline]
    fn next(&mut self) -> Option<u32> {
        match self {
            Iter::Empty => None,
            Iter::Tiny(t) => t.next(),
            Iter::Dense(d) => d.next(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        match self {
            Iter::Empty => 0,
            Iter::Tiny(t) => t.count(),
            Iter::Dense(d) => d.count(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Empty => (0, Some(0)),
            Iter::Tiny(t) => t.size_hint(),
            Iter::Dense(d) => d.size_hint(),
        }
    }
    #[inline]
    fn min(self) -> Option<u32> {
        match self {
            Iter::Empty => None,
            Iter::Tiny(t) => t.min(),
            Iter::Dense(d) => d.min(),
        }
    }
    #[inline]
    fn max(self) -> Option<u32> {
        match self {
            Iter::Empty => None,
            Iter::Tiny(t) => t.max(),
            Iter::Dense(d) => d.max(),
        }
    }
    #[inline]
    fn last(self) -> Option<u32> {
        match self {
            Iter::Empty => None,
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
                            println!("found {}", ((self.whichword as u32) << 5) + bit as u32);
                            println!("sz_left is {}", self.sz_left);
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



fn p_poverty(k: u32, idx: usize, n: usize) -> usize {
    ((idx % n) + n - (k % n as u32) as usize) % n
}

/// This inserts k into the array, and requires that there be room for
/// one more element.  Otherwise, things will be sad.
fn p_insert(k: u32, a: &mut [(u32,u32)]) -> usize {
    let n = a.len();
    for pov in 0..n {
        let ii = ((k + pov as u32) % n as u32) as usize;
        let ki = a[ii].0;
        let pov_ki = p_poverty(ki, ii, n);
        if a[ii] == (0,0) || ki == k {
            // println!("already got a spot");
            return ii;
        } else if pov_ki < pov {
            // println!("need to steal from {} < {} at spot {}", pov_ki, pov, ii);
            // need to steal
            let stolen = ii;
            let mut displaced = a[ii];
            // println!("displaced value is {}", displaced);
            let mut pov_displaced = pov_ki;
            a[stolen] = (0,0);

            for j in 1..n {
                pov_displaced += 1;
                let jj = (stolen + j) % n;
                let kj = a[jj].0;
                let pov_kj = p_poverty(kj, jj, n);
                if kj == 0 {
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
    let mut a = [(0,0),(0,0),(0,0),(0,0)];
    assert_eq!(2, p_insert(2, &mut a));
    assert_eq!(&a, &[(0,0),(0,0),(0,0),(0,0)]);
    for i in 0..10 {
        assert_eq!((0,0), a[p_insert(i, &mut a)]);
    }

    let mut a = [(0,0),(0,0),(6,6),(0,0)];
    assert_eq!(3, p_insert(2, &mut a));
    assert_eq!(&a, &[(0,0),(0,0),(6,6),(0,0)]);
    for i in 0..10 {
        assert!([0,i].contains(&a[p_insert(i, &mut a)].0));
    }

    let mut a = [(0,0),(0,0),(6,6),(3,3)];
    assert_eq!(3, p_insert(2, &mut a));
    assert_eq!(&a, &[(3,3),(0,0),(6,6),(0,0)]);
    for i in 0..10 {
        assert!([0,i].contains(&a[p_insert(i, &mut a)].0));
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

fn p_lookfor(k: u32, a: &[(u32,u32)]) -> LookedUp {
    let n = a.len();
    for pov in 0..n {
        let ii = ((k + pov as u32) % n as u32) as usize;
        if a[ii] == (0,0) {
            // println!("got empty spot at {} for key {}", ii, k);
            return LookedUp::EmptySpot(ii);
        }
        let ki = a[ii].0;
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
    assert_eq!(LookedUp::NeedInsert, p_lookfor(5, &[(3,10),(1,5),(2,2)]));
    assert_eq!(LookedUp::NeedInsert, p_lookfor(5, &[(3,10),(0,0),(2,3)]));
    assert_eq!(LookedUp::KeyFound(3), p_lookfor(7, &[(0,0),(0,0),(0,0),(7,7)]));
}

fn p_remove(k: u32, a: &mut [(u32,u32)]) -> bool {
    let n = a.len();
    for i in 0..n {
        let ii = ((k + i as u32) % n as u32) as usize;
        // println!("    looking to remove at distance {} slot {}", i, ii);
        if a[ii] == (0,0) {
            return false;
        }
        let ki = a[ii].0;
        let iki = (((ii + n) as u32 - (ki % n as u32)) % n as u32) as usize;
        if i > iki {
            return false;
        } else if ki == k {
            // println!("found {} at location {}", k, ii);
            a[ii] = (0,0);
            // Now we need to return anything that might have been
            // stolen from... to massacre my grammar.
            let mut previous = ii;
            for j in 1..n {
                let jj = (ii + j) % n;
                // println!("looking at removing offset {} at location {}", j, jj);
                let kj = a[jj].0;
                let jkj = (((jj + n) as u32 - (kj % n as u32)) % n as u32) as usize;
                if kj == 0 || jkj == 0 {
                    // We found an unoccupied spot or a perfectly
                    // happy customer, so nothing else could have been
                    // bumped.
                    return true;
                }
                // need to undo some stealing!
                a[previous] = a[jj];
                a[jj] = (0,0);
                previous = jj;
            }
            return true;
        }
    }
    panic!("bug: we should have had space in {:?} for {}", a, k)
}

#[cfg(test)]
fn test_p_insert_remove(x: u32, a: &mut [(u32,u32)]) {
    println!("test_insert_remove({}, {:?})", x, a);
    let v: Vec<(u32,u32)> = a.iter().cloned().collect();
    assert!(!p_remove(x, a));
    assert!(!a.contains(&(x,137))); // otherwise the test won't work right.
    assert!(!a.iter().any(|z| z.0 == x)); // otherwise the test won't work right.
    assert!(!p_lookfor(x, a).key_found());
    a[p_insert(x, a)] = (x,137);
    assert!(a.contains(&(x,137)));
    println!("  after insertion of {} a is {:?}", x, a);
    assert!(p_lookfor(x, a).key_found());
    assert_eq!((x,137), a[p_lookfor(x, a).unwrap()]);
    assert!(p_remove(x, a));
    println!("  after remove of {} a is {:?}", x, a);
    assert_eq!(a, &v[..]);
}

#[test]
fn test_p_remove() {
    let mut a = [(0,0),(0,0),(2,1)];
    a[p_insert(5,&mut a)] = (5,3);
    assert_eq!(&[(5,3),
                 (0,0),
                 (2,1)], &a);
    p_remove(2,&mut a);
    println!("after removal {:?}", a);
    assert!(p_lookfor(5, &a).key_found());
    assert_eq!(&[(0,0),
                 (0,0),
                 (5,3)], &a);

    test_p_insert_remove(7,&mut [(0,0),(0,0),(0,0),(0,0)]);

    test_p_insert_remove(2,&mut [(0,0),(1,1),(5,5),(0,0)]);

    test_p_insert_remove(5,&mut [(0,0),(0,0),(2,8)]);
}
