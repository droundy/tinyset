extern crate david_allocator;
extern crate david_set;
extern crate rand;
extern crate smallset;
extern crate fnv;

use std::time::{Instant, Duration};

use rand::{XorShiftRng, SeedableRng, Rand};

use std::collections::BTreeSet;
use david_set::Set;
use david_set::VecSet;
use david_set::TinySet;
use fnv::FnvHashSet;

type SmallSet<T> = smallset::SmallSet<[T; 8]>;

macro_rules! initialize {
    ($set: expr, $item: ident, $num: expr) => {{
        let mut rng = XorShiftRng::from_seed([$num as u32,$num as u32,3,4]);
        let before = david_allocator::net_allocation();
        let before_total = david_allocator::total_allocation();
        let mut set = $set;
        if $num > 0 {
            while set.len() < $num {
                set.insert($item::rand(&mut rng) % (2*$num as $item));
                set.remove(& ($item::rand(&mut rng) % (2*$num as $item)));
            }
        }
        let net_alloced = david_allocator::net_allocation() - before;
        let total_alloced = david_allocator::total_allocation() - before_total;
        let len_stack = std::mem::size_of_val(&set);
        (set, len_stack, net_alloced, total_alloced)
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
    ($set: expr, $item: ident, $size: expr, $iters: expr) => {{
        let (set, my_stack, my_size, my_alloc) = initialize!($set, $item, $size);
        let mut total = 0;
        let mut i = 0;
        let my_time = time_me!({
            i = (i+1)%(2*$size as $item);
            if set.contains(&i) { total += 1; }
        }, $iters);
        (total, my_time, my_stack, my_size, my_alloc)
    }};
}

macro_rules! bench_all_contains {
    ($item: ident, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "contains", "size");
        print!("{:^8}( tot / heap)", "hash");
        print!("{:^8}( tot / heap)", "set");
        print!("{:^8}( tot / heap)", "vecset");
        print!("{:^8}( tot / heap)", "btree");
        print!("{:^8}( tot / heap)", "smallset");
        print!("{:^8}( tot / heap)", "tinyset");
        println!();
        for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)
                                  .filter(|&x|x<$maxsz)) {
            print!("{:5}",size);

            let (total_true, hash_time, _, _, _)
                = bench_contains!(FnvHashSet::<$item>::default(), $item, size, $iters);

            let (total, my_time, my_stack, my_size, _)
                = bench_contains!(FnvHashSet::<$item>::default(), $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (total, my_time, my_stack, my_size, _)
                = bench_contains!(Set::<$item>::new(), $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (total, my_time, my_stack, my_size, _)
                = bench_contains!(VecSet::<$item>::new(), $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (total, my_time, my_stack, my_size, _)
                = bench_contains!(BTreeSet::<$item>::new(), $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (total, my_time, my_stack, my_size, _)
                = bench_contains!(SmallSet::<$item>::new(), $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (total, my_time, my_stack, my_size, _)
                = bench_contains!(TinySet::<$item>::new(), $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            println!();
        }
    }};
}

macro_rules! bench_remove_insert {
    ($set: expr, $item: ident, $size: expr, $iters: expr) => {{
        let (mut set, my_stack, my_size, my_alloc) = initialize!($set, $item, $size);
        let mut i = 0;
        let my_time = time_me!({
            i = (i+1)%(2*$size as $item);
            if set.remove(&i) { set.insert(i); }
        }, $iters);
        (my_time, my_stack, my_size, my_alloc)
    }};
}

macro_rules! bench_all_remove_insert {
    ($item: ident, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "remove/ins", "size");
        print!("{:^8}( tot / heap)", "hash");
        print!("{:^8}( tot / heap)", "set");
        print!("{:^8}( tot / heap)", "vecset");
        print!("{:^8}( tot / heap)", "btree");
        print!("{:^8}( tot / heap)", "smallset");
        print!("{:^8}( tot / heap)", "tinyset");
        println!();
        for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)
                                  .filter(|&x|x<$maxsz)) {
            print!("{:5}",size);
            let (hash_time, _, _, _)
                = bench_remove_insert!(FnvHashSet::<$item>::default(), $item, size, $iters);

            let (my_time, my_stack, my_size, _)
                = bench_remove_insert!(FnvHashSet::<$item>::default(), $item, size, $iters);
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (my_time, my_stack, my_size, _)
                = bench_remove_insert!(Set::<$item>::new(), $item, size, $iters);
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (my_time, my_stack, my_size, _)
                = bench_remove_insert!(VecSet::<$item>::new(), $item, size, $iters);
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (my_time, my_stack, my_size, _)
                = bench_remove_insert!(BTreeSet::<$item>::new(), $item, size, $iters);
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (my_time, my_stack, my_size, _)
                = bench_remove_insert!(SmallSet::<$item>::new(), $item, size, $iters);
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            let (my_time, my_stack, my_size, _)
                = bench_remove_insert!(TinySet::<$item>::new(), $item, size, $iters);
            print!(" {:6.3} ({:5.1}/{:5.1})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/size as f64),
                   (my_size as f64/size as f64));

            println!();
        }
    }};
}

fn main() {
    let iters = 10000000;

    println!("\n==============\n    u8\n==============");
    let maxsz = 250;
    bench_all_contains!(u8, iters, maxsz);
    bench_all_remove_insert!(u8, iters, maxsz);

    let maxsz = 10*iters;
    println!("\n==============\n    u32\n==============");
    bench_all_contains!(u32, iters, maxsz);
    bench_all_remove_insert!(u32, iters, maxsz);

    println!("\n==============\n    usize\n==============");
    bench_all_contains!(usize, iters, maxsz);
    bench_all_remove_insert!(usize, iters, maxsz);
}

fn duration_to_f64(t: Duration) -> f64 {
    t.as_secs() as f64 + (t.subsec_nanos() as f64)*1e-9
}
