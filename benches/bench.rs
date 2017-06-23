#![feature(test)]

extern crate test;
extern crate david_set;

use test::Bencher;
use std::collections::HashSet;

macro_rules! missing_bench {
    ($i: expr, $name_small: ident, $name_hash: ident) => {
        #[bench]
        fn $name_small(b: &mut Bencher) {
            let set: david_set::Set<usize> = (0..$i).collect();
            let not_here = $i+1;
            b.iter(|| set.contains(&not_here));
        }
        #[bench]
        fn $name_hash(b: &mut Bencher) {
            let set: HashSet<usize> = (0..$i).collect();
            let not_here = $i+1;
            b.iter(|| set.contains(&not_here));
        }
    };
}

missing_bench!(0, missing_smallset_00, missing_hashset_00);
missing_bench!(1, missing_smallset_01, missing_hashset_01);
missing_bench!(2, missing_smallset_02, missing_hashset_02);
missing_bench!(3, missing_smallset_03, missing_hashset_03);
missing_bench!(4, missing_smallset_04, missing_hashset_04);
missing_bench!(5, missing_smallset_05, missing_hashset_05);
missing_bench!(6, missing_smallset_06, missing_hashset_06);
missing_bench!(7, missing_smallset_07, missing_hashset_07);
missing_bench!(8, missing_smallset_08, missing_hashset_08);
missing_bench!(9, missing_smallset_09, missing_hashset_09);
missing_bench!(10, missing_smallset_10, missing_hashset_10);
missing_bench!(99, missing_smallset_99, missing_hashset_99);

macro_rules! present_bench {
    ($i: expr, $name_small: ident, $name_hash: ident) => {
        #[bench]
        fn $name_small(b: &mut Bencher) {
            let here = $i-1;
            let set: david_set::Set<usize> = (0..$i).collect();
            b.iter(|| set.contains(&here));
        }
        #[bench]
        fn $name_hash(b: &mut Bencher) {
            let here = $i-1;
            let set: HashSet<usize> = (0..$i).collect();
            b.iter(|| set.contains(&here));
        }
    };
}

present_bench!(0, present_smallset_00, present_hashset_00);
present_bench!(1, present_smallset_01, present_hashset_01);
present_bench!(2, present_smallset_02, present_hashset_02);
present_bench!(3, present_smallset_03, present_hashset_03);
present_bench!(4, present_smallset_04, present_hashset_04);
present_bench!(5, present_smallset_05, present_hashset_05);
present_bench!(6, present_smallset_06, present_hashset_06);
present_bench!(7, present_smallset_07, present_hashset_07);
present_bench!(8, present_smallset_08, present_hashset_08);
present_bench!(9, present_smallset_09, present_hashset_09);
present_bench!(10, present_smallset_10, present_hashset_10);
present_bench!(99, present_smallset_99, present_hashset_99);

macro_rules! collect_bench {
    ($i: expr, $name_small: ident, $name_hash: ident) => {
        #[bench]
        fn $name_small(b: &mut Bencher) {
            b.iter(|| {
                let s: david_set::Set<usize> = (0..$i).collect();
                s
            });
        }
        #[bench]
        fn $name_hash(b: &mut Bencher) {
            b.iter(|| {
                let s: HashSet<usize> = (0..$i).collect();
                s
            });
        }
    };
}

collect_bench!( 1, collect_smallset_01, collect_hashset_01);
collect_bench!( 2, collect_smallset_02, collect_hashset_02);
collect_bench!( 3, collect_smallset_03, collect_hashset_03);
collect_bench!( 4, collect_smallset_04, collect_hashset_04);
collect_bench!( 5, collect_smallset_05, collect_hashset_05);
collect_bench!( 6, collect_smallset_06, collect_hashset_06);
collect_bench!( 7, collect_smallset_07, collect_hashset_07);
collect_bench!( 8, collect_smallset_08, collect_hashset_08);
collect_bench!( 9, collect_smallset_09, collect_hashset_09);
collect_bench!(99, collect_smallset_99, collect_hashset_99);
