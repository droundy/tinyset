// Copyright 2017-2018 David Roundy
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate tinyset;
extern crate rand;
extern crate smallset;
extern crate fnv;

use std::time::{Instant, Duration};

use rand::{XorShiftRng, SeedableRng, Rand};

use std::collections::BTreeSet;
use std::collections::HashSet;
use tinyset::Set;
use tinyset::VecSet;
use tinyset::TinySet;
use fnv::FnvHashSet;
use tinyset::Set64;

type SmallSet<T> = smallset::SmallSet<[T; 8]>;

macro_rules! initialize {
    ($set: expr, $item: ident, $num: expr, $mx: expr) => {{
        let mut rng = XorShiftRng::from_seed([$num as u32,$num as u32,3,4]);
        let mut set = $set;
        if $num > 0 {
            while set.len() < $num {
                set.insert($item::rand(&mut rng) % ($mx as $item));
                set.remove(& ($item::rand(&mut rng) % ($mx as $item)));
            }
        }
        let len_stack = std::mem::size_of_val(&set);
        (set, len_stack)
    }};
}

macro_rules! time_me {
    ($fn: expr, $num: expr) => {{
        let now = Instant::now();
        for _ in 0..$num {
            $fn;
        }
        duration_to_f64(now.elapsed())
    }};
}

macro_rules! bench_contains {
    ($set: expr, $item: ident, $size: expr, $mx: expr, $iters: expr) => {{
        let (set, my_stack) = initialize!($set, $item, $size, $mx);
        let mut total = 0;
        let mut rng = XorShiftRng::from_seed([$size as u32,$iters as u32,1,1000]);
        let my_time = time_me!({
            let to_check = $item::rand(&mut rng) % ($mx as $item);
            if set.contains(&to_check) { total += 1; }
        }, $iters);
        (total, my_time, my_stack)
    }};
}

macro_rules! bench_c {
    ($set: expr, $item: ident, $size: expr, $mx: expr, $iters: expr,
     $hash_time: expr, $total_true: expr) => {{
        let (total, my_time, my_stack)
            = bench_contains!($set, $item, $size, $mx, $iters);
         if total != $total_true {
             println!("serious problem!");
         }
        print!(" {:6.3}", my_time/$hash_time);
    }};
}

const USIZE_SIZES: &[usize] = &[20,30,50,100,200,1000,2000,5000,10000,20000,30000,
                                100000, 200_000, 300_000, 500_000,
                                1_000_000];
const USIZE_MAXES: &[usize] = &[254, 10000, 100_000, 10_000_000_000];

macro_rules! bench_all_contains {
    (usize, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>12}", "contains", "max");
        print!("{:^8}", "fnvhash");
        print!("{:^8}", "hash");
        print!("{:^8}", "set");
        print!("{:^8}", "btree");
        print!("{:^8}", "tinyset");
        print!("{:^8}", "set64");
        println!();
        for size in (1..15).chain(USIZE_SIZES.iter().map(|&x|x).filter(|&x|x<$maxsz)) {
            println!("size: {:5}",size);
            for mx in USIZE_MAXES.iter().map(|&x|x).filter(|&x| x>=2*size) {
                print!("{:12}",mx);
                let (total_true, hash_time, _)
                    = bench_contains!(FnvHashSet::<usize>::default(), usize, size, mx, $iters);

                bench_c!(FnvHashSet::<usize>::default(), usize, size, mx, $iters,
                         hash_time, total_true);
                bench_c!(HashSet::<usize>::default(), usize, size, mx, $iters,
                         hash_time, total_true);
                bench_c!(Set::<usize>::new(), usize, size, mx, $iters,
                         hash_time, total_true);
                bench_c!(BTreeSet::<usize>::new(), usize, size, mx, $iters,
                         hash_time, total_true);
                bench_c!(TinySet::<usize>::new(), usize, size, mx, $iters,
                         hash_time, total_true);
                bench_c!(Set64::<usize>::new(), usize, size, mx, $iters,
                         hash_time, total_true);
                println!();
            }
        }
    }};
    (i8, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>6}", "contains", "size");
        print!("{:^8}", "fnvhash");
        print!("{:^8}", "hash");
        print!("{:^8}", "set");
        print!("{:^8}", "btree");
        print!("{:^8}", "tinyset");
        print!("{:^8}", "set64");
        println!();
        for size in (1..15).chain(USIZE_SIZES.iter().map(|&x|x).filter(|&x|x<$maxsz)) {
            let mx = 100;
            print!("{:6}",size);
            let (total_true, hash_time, _)
                = bench_contains!(FnvHashSet::<i8>::default(), i8, size, mx, $iters);

            bench_c!(FnvHashSet::<i8>::default(), i8, size, mx, $iters, hash_time, total_true);
            bench_c!(HashSet::<i8>::default(), i8, size, mx, $iters, hash_time, total_true);
            bench_c!(Set::<i8>::new(), i8, size, mx, $iters, hash_time, total_true);
            bench_c!(VecSet::<i8>::new(), i8, size, mx, $iters, hash_time, total_true);
            bench_c!(BTreeSet::<i8>::new(), i8, size, mx, $iters, hash_time, total_true);
            bench_c!(SmallSet::<i8>::new(), i8, size, mx, $iters, hash_time, total_true);
            bench_c!(Set64::<i8>::new(), i8, size, mx, $iters, hash_time, total_true);
            println!();
        }
    }};
    ($item: ident, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "contains", "size");
        print!("{:^8}", "fnvhash");
        print!("{:^8}", "hash");
        print!("{:^8}", "set");
        print!("{:^8}", "vecset");
        print!("{:^8}", "btree");
        print!("{:^8}", "smallset");
        print!("{:^8}", "tinyset");
        print!("{:^8}", "set64");
        println!();
        for size in (1..15).chain([20,30,50,100,1000].iter().map(|&x|x)
                                  .filter(|&x|x<$maxsz)) {
            print!("{:5}",size);

            let (total_true, hash_time, _)
                = bench_contains!(FnvHashSet::<$item>::default(), $item, size,
                                  2*size, $iters);

            bench_c!(FnvHashSet::<$item>::default(), $item, size, 2*size, $iters,
                     hash_time, total_true);
            bench_c!(HashSet::<$item>::default(), $item, size, 2*size, $iters,
                     hash_time, total_true);
            bench_c!(Set::<$item>::new(), $item, size, 2*size, $iters,
                     hash_time, total_true);
            bench_c!(VecSet::<$item>::new(), $item, size, 2*size, $iters,
                     hash_time, total_true);
            bench_c!(BTreeSet::<$item>::new(), $item, size, 2*size, $iters,
                     hash_time, total_true);
            bench_c!(SmallSet::<$item>::new(), $item, size, 2*size, $iters,
                     hash_time, total_true);
            bench_c!(TinySet::<$item>::new(), $item, size, 2*size, $iters,
                     hash_time, total_true);
            bench_c!(Set64::<$item>::new(), $item, size, 2*size, $iters,
                     hash_time, total_true);
            println!();
        }
    }};
}

macro_rules! bench_remove_insert {
    ($set: expr, $item: ident, $size: expr, $iters: expr) => {{
        let (mut set, my_stack) =
            initialize!($set, $item, $size, 2*$size);
        let mut rng = XorShiftRng::from_seed([$size as u32,$iters as u32,1,1000]);
        let my_time = time_me!({
            let i = $item::rand(&mut rng)%(2*$size as $item);
            if set.remove(&i) { set.insert(i); }
        }, $iters);
        (my_time, my_stack)
    }};
}

macro_rules! bench_ri {
    ($set: expr, $item: ident, $size: expr, $iters: expr, $hash_time: expr) => {{
        let (my_time, my_stack)
            = bench_remove_insert!($set, $item, $size, $iters);
        print!(" {:6.3}", my_time/$hash_time);
    }};
}

macro_rules! bench_all_remove_insert {
    (usize, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "remove/ins", "size");
        print!("{:^8}", "fnvhash");
        print!("{:^8}", "hash");
        print!("{:^8}", "set");
        print!("{:^8}", "btree");
        print!("{:^8}", "tinyset");
        print!("{:^8}", "set64");
        println!();
        for size in (1..15).chain(USIZE_SIZES.iter().map(|&x|x).filter(|&x|x<$maxsz)) {
            print!("{:5}",size);
            let (hash_time, _)
                = bench_remove_insert!(FnvHashSet::<usize>::default(), usize, size, $iters);

            bench_ri!(FnvHashSet::<usize>::default(), usize, size, $iters, hash_time);
            bench_ri!(HashSet::<usize>::default(), usize, size, $iters, hash_time);
            bench_ri!(Set::<usize>::new(), usize, size, $iters, hash_time);
            bench_ri!(BTreeSet::<usize>::new(), usize, size, $iters, hash_time);
            bench_ri!(TinySet::<usize>::new(), usize, size, $iters, hash_time);
            bench_ri!(Set64::<usize>::new(), usize, size, $iters, hash_time);
            println!();
        }
    }};
    (i8, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "remove/ins", "size");
        print!("{:^8}", "fnvhash");
        print!("{:^8}", "hash");
        print!("{:^8}", "set");
        print!("{:^8}", "btree");
        print!("{:^8}", "tinyset");
        print!("{:^8}", "set64");
        println!();
        for size in (1..15).chain(USIZE_SIZES.iter().map(|&x|x).filter(|&x|x<$maxsz)) {
            print!("{:5}",size);
            let (hash_time, _)
                = bench_remove_insert!(FnvHashSet::<i8>::default(), i8, size, $iters);

            bench_ri!(FnvHashSet::<i8>::default(), i8, size, $iters, hash_time);
            bench_ri!(HashSet::<i8>::default(), i8, size, $iters, hash_time);
            bench_ri!(Set::<i8>::new(), i8, size, $iters, hash_time);
            bench_ri!(VecSet::<i8>::new(), i8, size, $iters, hash_time);
            bench_ri!(BTreeSet::<i8>::new(), i8, size, $iters, hash_time);
            bench_ri!(SmallSet::<i8>::new(), i8, size, $iters, hash_time);
            bench_ri!(Set64::<i8>::new(), i8, size, $iters, hash_time);
            println!();
        }
    }};
    ($item: ident, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "remove/ins", "size");
        print!("{:^8}", "fnvhash");
        print!("{:^8}", "hash");
        print!("{:^8}", "set");
        print!("{:^8}", "vecset");
        print!("{:^8}", "btree");
        print!("{:^8}", "smallset");
        print!("{:^8}", "tinyset");
        print!("{:^8}", "set64");
        println!();
        for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)
                                  .filter(|&x|x<$maxsz)) {
            print!("{:5}",size);
            let (hash_time, _)
                = bench_remove_insert!(FnvHashSet::<$item>::default(), $item, size, $iters);

            bench_ri!(FnvHashSet::<$item>::default(), $item, size, $iters, hash_time);
            bench_ri!(HashSet::<$item>::default(), $item, size, $iters, hash_time);
            bench_ri!(Set::<$item>::new(), $item, size, $iters, hash_time);
            bench_ri!(VecSet::<$item>::new(), $item, size, $iters, hash_time);
            bench_ri!(BTreeSet::<$item>::new(), $item, size, $iters, hash_time);
            bench_ri!(SmallSet::<$item>::new(), $item, size, $iters, hash_time);
            bench_ri!(TinySet::<$item>::new(), $item, size, $iters, hash_time);
            bench_ri!(Set64::<$item>::new(), $item, size, $iters, hash_time);
            println!();
        }
    }};
}

fn main() {
    let iters = 10000000;

    println!("\n==============\n    i8\n==============");
    let maxsz = 64;
    bench_all_contains!(i8, iters, maxsz);
    bench_all_remove_insert!(i8, iters, maxsz);

    let maxsz = 10*iters;
    println!("\n==============\n    usize\n==============");
    bench_all_contains!(usize, iters, maxsz);
    bench_all_remove_insert!(usize, iters, maxsz);

    println!("\n==============\n    u8\n==============");
    let maxsz = 250;
    bench_all_contains!(u8, iters, maxsz);
    bench_all_remove_insert!(u8, iters, maxsz);

    let maxsz = 10*iters;
    println!("\n==============\n    u32\n==============");
    bench_all_contains!(u32, iters, maxsz);
    bench_all_remove_insert!(u32, iters, maxsz);
}

fn duration_to_f64(t: Duration) -> f64 {
    t.as_secs() as f64 + (t.subsec_nanos() as f64)*1e-9
}
