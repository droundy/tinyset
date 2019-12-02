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
    fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
    fn contains(&self, v: u32) -> bool {
        v >= self.start && v <= self.start + 30 && self.bits >> (v-self.start) & 1 != 0
    }
    fn insert(&mut self, v: u32) -> Option<bool> {
        if self.contains(v) {
            return Some(true);
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
}
#[cfg(test)]
proptest!{
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

//     fn new(mut v: Vec<u32>) -> Option<Self> {
//         if v.len() == 0 {
//             return None;
//         } else if v.len() > BITSPLITS.len() - 1 {
//             return None;
//         }
//         let sz = v.len() as u8;
//         v.sort();
//         v.dedup();
//         let mut last = 0;
//         let mut offset = 0;
//         let mut bits: usize = 0;
//         let bitsplits = BITSPLITS[sz as usize];
//         for (x,nbits) in v.into_iter().zip(bitsplits.iter().cloned()) {
//             let y = if offset == 0 {
//                 x
//             } else {
//                 x - last - 1
//             };
//             if log_2(y) > nbits {
//                 return None;
//             }
//             bits = bits | (y as usize) << offset;
//             offset += nbits;
//             last = x;
//         }
//         Some (Tiny { sz, bits, sz_spent: 0, last: 0 })
//     }
//     fn new_unchecked(v: impl Iterator<Item=u32>, sz: u8) -> Self {
//         let mut last = 0;
//         let mut offset = 0;
//         let mut bits: usize = 0;
//         let bitsplits = BITSPLITS[sz as usize];
//         for (x,nbits) in v.zip(bitsplits.iter().cloned()) {
//             let y = if offset == 0 {
//                 x
//             } else {
//                 x - last - 1
//             };
//             bits = bits | (y as usize) << offset;
//             offset += nbits;
//             last = x;
//         }
//         Tiny { sz, bits, sz_spent: 0, last: 0 }
//     }
//     fn insert(self, e: u32) -> Option<Self> {
//         let mut last = 0;
//         let mut offset = 0;
//         let mut bits: usize = 0;
//         let sz = self.sz + 1;
//         let bitsplits = BITSPLITS.get(sz as usize)?;
//         for (x,nbits) in self.clone().merge(Some(e).into_iter()).zip(bitsplits.iter().cloned()) {
//             let y = if offset == 0 {
//                 x
//             } else if x == last {
//                 return Some(self);
//             } else {
//                 x - last - 1
//             };
//             if log_2(y) > nbits {
//                 return None;
//             }
//             bits = bits | (y as usize) << offset;
//             offset += nbits;
//             last = x;
//         }
//         Some(Tiny { sz, bits, sz_spent: 0, last: 0 })
//     }
// }
