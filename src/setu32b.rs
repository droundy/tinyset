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
    fn min(mut self) -> Option<u32> {
        self.next()
    }
}

impl Tiny {
    #[cfg(target_pointer_width = "64")]
    fn to_usize(self) -> usize {
        (self.start as usize) << 32 | self.bits << 2 | 1
    }
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
}

#[cfg(test)]
proptest!{
    #[test]
    fn check_tiny_from_singleton(x: u32) {
        let t = Tiny::from_singleton(x).unwrap();
        assert_eq!(t.clone().next(), Some(x));
        assert_eq!(t.clone().count(), 1);
        assert_eq!(t.len(), 1);
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
