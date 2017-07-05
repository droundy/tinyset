extern crate david_allocator;
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
    ($map: expr, $item: ident, $num: expr, $mx: expr) => {{
        let mut rng = XorShiftRng::from_seed([$num as u32,$num as u32,3,4]);
        let before = david_allocator::net_allocation();
        let mut map = $map;
        if $num > 0 {
            while map.len() < $num {
                map.insert($item::rand(&mut rng) % ($mx as $item),
                           $item::rand(&mut rng) % ($mx as $item));
                map.remove(& ($item::rand(&mut rng) % ($mx as $item)));
            }
        }
        let net_alloced = david_allocator::net_allocation() - before;
        let len_stack = std::mem::size_of_val(&map);
        (map, len_stack, net_alloced)
    }};
}

macro_rules! bench_remove_insert {
    ($map: expr, $item: ident, $size: expr, $mx: expr, $iters: expr) => {{
        let (mut map, my_stack, my_size) = initialize!($map, $item, $size, $mx);
        let mut rng = XorShiftRng::from_seed([$size as u32,$iters as u32,1,1000]);
        let my_time = time_me!({
            let i = $item::rand(&mut rng)%(2*$size as $item);
            let j = $item::rand(&mut rng)%(2*$size as $item);
            if map.remove(&i).is_some() { map.insert(i, j); }
        }, $iters);
        print!(" {:6.1} ({:5.1}/{:5.1})",
               my_time*1e9/$iters as f64,
               ((my_stack+my_size) as f64/$size as f64),
               (my_size as f64/$size as f64));
        (my_time, my_stack, my_size)
    }};
}

macro_rules! bench_all_insert_remove {
    ($item: ident, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "ins/rem", "size");
        print!("{:^8}( tot / heap)", "fnv");
        print!("{:^8}( tot / heap)", "hash");
        print!("{:^8}( tot / heap)", "btree");
        print!("{:^8}( tot / heap)", "order");
        print!("{:^8}( tot / heap)", "flat");
        println!();
        for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)
                                  .filter(|&x|x<$maxsz)) {
            print!("{:5}",size);

            bench_remove_insert!(FnvHashMap::<$item,$item>::default(), $item, size,
                                 2*size, $iters);
            bench_remove_insert!(HashMap::<$item,$item>::default(), $item, size,
                                 2*size, $iters);
            bench_remove_insert!(BTreeMap::<$item,$item>::default(), $item, size,
                                 2*size, $iters);
            bench_remove_insert!(OrderMap::<$item,$item>::default(), $item, size,
                                 2*size, $iters);
            bench_remove_insert!(FlatMap::<$item,$item>::default(), $item, size,
                                 2*size, $iters);
            println!();
        }
    }};
}

fn main() {
    let iters = 10000000;

    let maxsz = 10*iters;
    println!("\n==============\n    usize\n==============");
    bench_all_insert_remove!(usize, iters, maxsz);
}

fn duration_to_f64(t: Duration) -> f64 {
    t.as_secs() as f64 + (t.subsec_nanos() as f64)*1e-9
}
