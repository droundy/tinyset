extern crate tinyset;
extern crate rand;
extern crate fnv;
extern crate ordermap;
extern crate flat_map;

use std::time::{Instant, Duration};

use rand::{XorShiftRng, SeedableRng, Rand};

use std::collections::BTreeMap;
use std::collections::HashMap;
use fnv::FnvHashMap;
use ordermap::OrderMap;
use flat_map::FlatMap;
use tinyset::TinyMap;
use tinyset::{Map64, Map6464};

macro_rules! time_me {
    ($fn: expr, $num: expr) => {{
        let now = Instant::now();
        for _ in 0..$num {
            $fn;
        }
        duration_to_f64(now.elapsed())
    }};
}

macro_rules! initialize {
    ($map: expr, $item: ident, $vexpr: expr, $num: expr, $mx: expr) => {{
        let mut rng = XorShiftRng::from_seed([$num as u32,$num as u32,3,4]);
        let mut map = $map;
        if $num > 0 {
            while map.len() < $num {
                map.insert($item::rand(&mut rng) % ($mx as $item), $vexpr);
                map.remove(& ($item::rand(&mut rng) % ($mx as $item)));
            }
        }
        let len_stack = std::mem::size_of_val(&map);
        (map, len_stack)
    }};
}

macro_rules! bench_remove_insert {
    ($map: expr, $item: ident, $v: expr, $size: expr, $mx: expr, $iters: expr) => {{
        let (mut map, my_stack) = initialize!($map, $item, $v, $size, $mx);
        let mut rng = XorShiftRng::from_seed([$size as u32,$iters as u32,1,1000]);
        let my_time = time_me!({
            // let i = $item::rand(&mut rng)%(2*$size as $item);
            // if let Some(e) = map.remove(&i) { map.insert(i, e); }
            let i = $item::rand(&mut rng)%(2*$size as $item);
            if let Some(e) = map.remove(&i) {
                let mut j: $item = 0;
                while map.get(&j).is_some() {
                    j = $item::rand(&mut rng)%(2*$size as $item);
                }
                map.insert(j, e);
            }
        }, $iters);
        print!(" {:6.1}", my_time*1e9/$iters as f64);
        (my_time, my_stack)
    }};
}

macro_rules! bench_all_insert_remove {
    ($item: ident, $vty: ty, $v: expr, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>6}", "ins/rem", "size");
        print!("{:>7}", "fnv");
        print!("{:>7}", "hash");
        print!("{:>7}", "btree");
        print!("{:>7}", "order");
        print!("{:>7}", "flat");
        print!("{:>7}", "tiny");
        print!("{:>7}", "map64");
        print!("{:>7}", "ma6464");
        println!();
        for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)
                                  .filter(|&x|x<$maxsz)) {
            print!("{:6}",size);

            bench_remove_insert!(FnvHashMap::<$item,$vty>::default(), $item, $v, size,
                                 2*size, $iters);
            bench_remove_insert!(HashMap::<$item,$vty>::default(), $item, $v, size,
                                 2*size, $iters);
            bench_remove_insert!(BTreeMap::<$item,$vty>::default(), $item, $v, size,
                                 2*size, $iters);
            bench_remove_insert!(OrderMap::<$item,$vty>::default(), $item, $v, size,
                                 2*size, $iters);
            bench_remove_insert!(FlatMap::<$item,$vty>::default(), $item, $v, size,
                                 2*size, $iters);
            bench_remove_insert!(TinyMap::<$item,$vty>::new(), $item, $v, size,
                                 2*size, $iters);
            bench_remove_insert!(Map64::<$item,$vty>::new(), $item, $v, size,
                                 2*size, $iters);
            // bench_remove_insert!(Map6464::<$item,$vty>::new(), $item, $v, size,
            //                      2*size, $iters);
            println!();
        }
    }};
}

fn main() {
    let iters = 10000000;

    let maxsz = 10*iters;
    println!("\n==============\n    u64, &str\n==============");
    bench_all_insert_remove!(u64, &str, &"hello world", iters, maxsz);

    let maxsz = 10*iters;
    println!("\n==============\n    usize,usize\n==============");
    bench_all_insert_remove!(usize, usize, 5, iters, maxsz);

    let maxsz = 120;
    println!("\n==============\n    u8, [u8;128]\n==============");
    bench_all_insert_remove!(u8, [u8;128], [3;128], iters, maxsz);
}

fn duration_to_f64(t: Duration) -> f64 {
    t.as_secs() as f64 + (t.subsec_nanos() as f64)*1e-9
}
