use easybench::{bench_gen_env, bench_power_scaling};
use rand::Rng;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn mem_used<T>(f: impl Fn() -> T) -> (T, usize) {
    let reg = Region::new(&GLOBAL);
    let v = f();
    let stats = reg.change();
    let total = stats.bytes_allocated as isize - stats.bytes_deallocated as isize + stats.bytes_reallocated;
    if total > 0 {
        (v, std::mem::size_of::<T>() + total as usize)
    } else {
        (v, std::mem::size_of::<T>())
    }
}

fn bench_sets(density: f64, num_elements: usize) {
    assert!(density <= 1.0);
    let mut gen = move || {
        let mut rng = rand::thread_rng();
        let mx = (num_elements as f64/density) as u64 + 1;
        let mut set = tinyset::SetU64::new();
        while set.len() < num_elements {
            set.insert(rng.gen_range(0, mx));
        }
        (rng.gen_range(0, mx), set)
    };
    let mut gen_hashset = move || {
        let mut rng = rand::thread_rng();
        let mx = (num_elements as f64/density) as u64 + 1;
        let mut set = std::collections::HashSet::new();
        while set.len() < num_elements {
            set.insert(rng.gen_range(0, mx));
        }
        (rng.gen_range(0, mx), set)
    };
    let mut gen_tiny = move || {
        let mut rng = rand::thread_rng();
        let mx = (num_elements as f64/density) as u64 + 1;
        let mut set = tinyset::Set64::new();
        while set.len() < num_elements {
            set.insert(rng.gen_range(0, mx));
        }
        (rng.gen_range(0, mx), set)
    };

    let mut sz_total = 0.0;
    for _ in 0..1000 {
        sz_total += gen().1.mem_used() as f64;
    }
    sz_total /= 1000.0;
    println!("\n{}, {}: {:.1} bytes {:>9} {:>9} {:>9}", density, num_elements, sz_total,
             "this", "tiny", "std");
    println!("{:>18}: {:5.0}ns {:5.0}ns {:5.0}ns", ".len()",
             bench_gen_env(&mut gen, |(_,set)| {
                 set.len()
             }).ns_per_iter,
             bench_gen_env(&mut gen_tiny, |(_,set)| {
                 set.len()
             }).ns_per_iter,
             bench_gen_env(&mut gen_hashset, |(_,set)| {
                 set.len()
             }).ns_per_iter,
    );
    println!("{:>18}: {:5.0}ns {:5.0}ns {:5.0}ns", ".contains(ran)",
             bench_gen_env(gen, |(idx,set)| {
                 set.contains(*idx)
             }).ns_per_iter,
             bench_gen_env(gen_tiny, |(idx,set)| {
                 set.contains(*idx)
             }).ns_per_iter,
             bench_gen_env(gen_hashset, |(idx,set)| {
                 set.contains(idx)
             }).ns_per_iter,
    );
    println!("{:>18}: {:5.0}ns {:5.0}ns {:5.0}ns", ".remove(ran)",
             bench_gen_env(gen, |(idx,set)| {
                 set.remove(*idx)
             }).ns_per_iter,
             bench_gen_env(gen_tiny, |(idx,set)| {
                 set.remove(idx)
             }).ns_per_iter,
             bench_gen_env(gen_hashset, |(idx,set)| {
                 set.remove(idx)
             }).ns_per_iter,
    );
    println!("{:>18}: {:5.0}ns {:5.0}ns {:5.0}ns", ".insert(ran)",
             bench_gen_env(gen, |(idx,set)| {
                 set.insert(*idx)
             }).ns_per_iter,
             bench_gen_env(gen_tiny, |(idx,set)| {
                 set.insert(*idx)
             }).ns_per_iter,
             bench_gen_env(gen_hashset, |(idx,set)| {
                 set.insert(*idx)
             }).ns_per_iter,
    );
}

fn bench_collect(density: f64) {
    assert!(density <= 1.0);
    println!("\ncollect {:5}:{:>12} {:>13} {:>13} {:>13}", density,
             "setu64", "setu32", "tiny", "std");
    for sz in 0..10 {
        let mut gen = move || {
            let mut rng = rand::thread_rng();
            let mx = (sz as f64/density) as u64 + 1;
            let mut vec = Vec::new();
            while vec.iter().cloned().collect::<tinyset::SetU64>().len() < sz {
                vec.push(rng.gen_range(0, mx));
            }
            vec
        };
        let mut gen32 = move || {
            let mut rng = rand::thread_rng();
            let mx = (sz as f64/density) as u32 + 1;
            let mut vec = Vec::new();
            while vec.iter().cloned().collect::<tinyset::SetU32>().len() < sz {
                vec.push(rng.gen_range(0, mx));
            }
            vec
        };
        println!("{:>11}: {:8.0}ns    {:8.0}ns    {:8.0}ns    {:8.0}ns", sz,
                 bench_gen_env(&mut gen, |v| {
                     v.iter().cloned().collect::<tinyset::SetU64>().len()
                 }).ns_per_iter,
                 bench_gen_env(&mut gen32, |v| {
                     v.iter().cloned().collect::<tinyset::SetU32>().len()
                 }).ns_per_iter,
                 bench_gen_env(&mut gen, |v| {
                     v.iter().cloned().collect::<tinyset::Set64<_>>().len()
                 }).ns_per_iter,
                 bench_gen_env(&mut gen, |v| {
                     v.iter().cloned().collect::<std::collections::HashSet<_>>().len()
                 }).ns_per_iter,
        );
    }
    let mut gen = move |sz| {
        let mut rng = rand::thread_rng();
        let mx = (sz as f64/density) as u64 + 1;
        let mut vec = Vec::new();
        while vec.iter().cloned().collect::<tinyset::SetU64>().len() < sz {
            vec.push(rng.gen_range(0, mx));
            }
        vec
    };
    let mut gen32 = move |sz| {
        let mut rng = rand::thread_rng();
        let mx = (sz as f64/density) as u32 + 1;
        let mut vec = Vec::new();
        while vec.iter().cloned().collect::<tinyset::SetU32>().len() < sz {
            vec.push(rng.gen_range(0, mx));
            }
        vec
    };
    println!("{:>11}: {:8.0} {:8.0} {:8.0} {:8.0}", ".collect()",
             bench_power_scaling(&mut gen, |v| {
                 v.iter().cloned().collect::<tinyset::SetU64>().len()
             }, 10).scaling,
             bench_power_scaling(&mut gen32, |v| {
                 v.iter().cloned().collect::<tinyset::SetU32>().len()
             }, 10).scaling,
             bench_power_scaling(&mut gen, |v| {
                 v.iter().cloned().collect::<tinyset::Set64<_>>().len()
             }, 10).scaling,
             bench_power_scaling(&mut gen, |v| {
                 v.iter().cloned().collect::<std::collections::HashSet<_>>().len()
             }, 10).scaling,
    );
}

fn bench_funcs<O>(name: &str,
                  density: f64,
                  func32: impl Copy + Fn(&mut tinyset::SetU32) -> O,
                  func32b: impl Copy + Fn(&mut tinyset::setu32b::SetU32) -> O,
                  funchash32: impl Copy + Fn(&mut std::collections::HashSet<u32>) -> O,
                  func64: impl Copy + Fn(&mut tinyset::SetU64) -> O,
                  oldtiny: impl Copy + Fn(&mut tinyset::Set64<u64>) -> O,
                  funchash: impl Copy + Fn(&mut std::collections::HashSet<u64>) -> O) {
    assert!(density <= 1.0);
    println!("\n{:<5}{:5}:{:>11} {:>13} {:>13} {:>13} {:>13} {:>13}", name, density,
             "setu32", "setu32b", "std32", "setu64", "tiny", "std64");
    for sz in 0..10 {
        let gen = move || {
            let mut rng = rand::thread_rng();
            let mx = (sz as f64/density) as u64 + 1;
            let mut vec = Vec::new();
            while vec.iter().cloned().collect::<tinyset::SetU64>().len() < sz {
                vec.push(rng.gen_range(0, mx));
            }
            vec
        };
        println!("{:>9}:{:8.0}ns/{:<3.0} {:7.0}ns/{:<3.0} {:7.0}ns/{:<3.0} {:7.0}ns/{:<3.0} {:7.0}ns/{:<3.0} {:7.0}ns/{:<3.0}",
                 sz,
                 bench_gen_env(|| { gen().iter().cloned()
                                    .map(|x| x as u32).collect::<tinyset::SetU32>() },
                               func32).ns_per_iter,
                 (0..100).map(|_| mem_used(|| gen().iter().cloned()
                                           .map(|x| x as u32).collect::<tinyset::SetU32>()).1).sum::<usize>() as f64/100.0,
                 bench_gen_env(|| { gen().iter().cloned()
                                    .map(|x| x as u32).collect::<tinyset::setu32b::SetU32>() },
                               func32b).ns_per_iter,
                 (0..100).map(|_| mem_used(|| gen().iter().cloned()
                                           .map(|x| x as u32).collect::<tinyset::setu32b::SetU32>()).1).sum::<usize>() as f64/100.0,
                 bench_gen_env(|| { gen().iter().cloned()
                                    .map(|x| x as u32)
                                    .collect::<std::collections::HashSet<_>>() },
                               funchash32).ns_per_iter,
                 (0..100).map(|_| mem_used(|| gen().iter().cloned()
                                           .map(|x| x as u32)
                                           .collect::<std::collections::HashSet<_>>()).1).sum::<usize>() as f64/100.0,
                 bench_gen_env(|| { gen().iter().cloned().collect::<tinyset::SetU64>() },
                               func64).ns_per_iter,
                 (0..100).map(|_| mem_used(|| gen().iter().cloned().collect::<tinyset::SetU64>()).1).sum::<usize>() as f64/100.0,
                 bench_gen_env(|| { gen().iter().cloned().collect::<tinyset::Set64<_>>() },
                               oldtiny).ns_per_iter,
                 (0..100).map(|_| mem_used(|| gen().iter().cloned().collect::<tinyset::Set64<_>>()).1).sum::<usize>() as f64/100.0,
                 bench_gen_env(|| { gen().iter().cloned().collect::<std::collections::HashSet<_>>() },
                               funchash).ns_per_iter,
                 (0..100).map(|_| mem_used(|| gen().iter().cloned().collect::<std::collections::HashSet<_>>()).1).sum::<usize>() as f64/100.0,
        );
    }
    let gen = move |sz| {
        let mut rng = rand::thread_rng();
        let mx = (sz as f64/density) as u64 + 1;
        let mut vec = Vec::new();
        while vec.iter().cloned().collect::<tinyset::SetU64>().len() < sz {
            vec.push(rng.gen_range(0, mx));
        }
        vec
    };
    println!("{:>10}: {:8.0} {:8.0} {:8.0} {:8.0} {:8.0} {:8.0}", "scaling",
             bench_power_scaling(
                 |sz| {
                     gen(sz).iter().cloned().map(|x| x as u32).collect::<tinyset::SetU32>()
                 }, func32, 10).scaling,
             bench_power_scaling(
                 |sz| {
                     gen(sz).iter().cloned().map(|x| x as u32).collect::<tinyset::setu32b::SetU32>()
                 }, func32b, 10).scaling,
             bench_power_scaling(
                 |sz| {
                     gen(sz).iter().map(|&x| x as u32).collect::<std::collections::HashSet<_>>()
                 }, funchash32, 10).scaling,
             bench_power_scaling(
                 |sz| {
                     gen(sz).iter().cloned().collect::<tinyset::SetU64>()
                 }, func64, 10).scaling,
             bench_power_scaling(
                 |sz| {
                     gen(sz).iter().cloned().collect::<tinyset::Set64<u64>>()
                 }, oldtiny, 10).scaling,
             bench_power_scaling(
                 |sz| {
                     gen(sz).iter().cloned().collect::<std::collections::HashSet<_>>()
                 }, funchash, 10).scaling,
    );
}

fn bench_min(density: f64) {
    bench_funcs("min", density,
                |s| { s.iter().min().map(|x| x as u64) },
                |s| { s.iter().min().map(|x| x as u64) },
                |s| { s.iter().min().map(|x| *x as u64) },
                |s| { s.iter().min() },
                |s| { s.iter().min() },
                |s| { s.iter().cloned().min() },
    );
}

fn bench_sum(density: f64) {
    bench_funcs("sum", density,
                |s| { s.iter().sum::<u32>() as u64 },
                |s| { s.iter().sum::<u32>() as u64 },
                |s| { s.iter().sum::<u32>() as u64 },
                |s| { s.iter().sum::<u64>() },
                |s| { s.iter().sum::<u64>() },
                |s| { s.iter().cloned().sum::<u64>() },
    );
}


fn bench_scaling(density: f64, min: usize) {
    assert!(density <= 1.0);
    let mut gen = move |num_elements| {
        let mut rng = rand::thread_rng();
        let mx = (num_elements as f64/density) as u64 + 1;
        let mut set = tinyset::SetU64::new();
        while set.len() < num_elements {
            set.insert(rng.gen_range(0, mx));
        }
        (rng.gen_range(0, mx), set)
    };
    let mut gen_hashset = move |num_elements| {
        let mut rng = rand::thread_rng();
        let mx = (num_elements as f64/density) as u64 + 1;
        let mut set = std::collections::HashSet::new();
        while set.len() < num_elements {
            set.insert(rng.gen_range(0, mx));
        }
        (rng.gen_range(0, mx), set)
    };
    let mut gen_tiny = move |num_elements| {
        let mut rng = rand::thread_rng();
        let mx = (num_elements as f64/density) as u64 + 1;
        let mut set = tinyset::Set64::new();
        while set.len() < num_elements {
            set.insert(rng.gen_range(0, mx));
        }
        (rng.gen_range(0, mx), set)
    };

    println!("\n{},      {:5}-: {:>13} {:>13} {:>13}", density, min,
             "this", "tiny", "std");
    println!("{:>18}: {:5.0} {:5.0} {:5.0}", ".len()",
             bench_power_scaling(&mut gen, |(_,set)| {
                 set.len()
             }, min).scaling,
             bench_power_scaling(&mut gen_tiny, |(_,set)| {
                 set.len()
             }, min).scaling,
             bench_power_scaling(&mut gen_hashset, |(_,set)| {
                 set.len()
             }, min).scaling,
    );
    println!("{:>18}: {:5.0} {:5.0} {:5.0}", ".contains(ran)",
             bench_power_scaling(gen, |(idx,set)| {
                 set.contains(*idx)
             }, min).scaling,
             bench_power_scaling(gen_tiny, |(idx,set)| {
                 set.contains(*idx)
             }, min).scaling,
             bench_power_scaling(gen_hashset, |(idx,set)| {
                 set.contains(idx)
             }, min).scaling,
    );
    println!("{:>18}: {:5.0} {:5.0} {:5.0}", ".remove(ran)",
             bench_power_scaling(gen, |(idx,set)| {
                 set.remove(*idx)
             }, min).scaling,
             bench_power_scaling(gen_tiny, |(idx,set)| {
                 set.remove(idx)
             }, min).scaling,
             bench_power_scaling(gen_hashset, |(idx,set)| {
                 set.remove(idx)
             }, min).scaling,
    );
    println!("{:>18}: {:5.0} {:5.0} {:5.0}", ".insert(ran)",
             bench_power_scaling(gen, |(idx,set)| {
                 set.insert(*idx)
             }, min).scaling,
             bench_power_scaling(gen_tiny, |(idx,set)| {
                 set.insert(*idx)
             }, min).scaling,
             bench_power_scaling(gen_hashset, |(idx,set)| {
                 set.insert(*idx)
             }, min).scaling,
    );
}

fn main() {

    bench_sum(0.001);
    bench_sum(0.05);
    bench_sum(0.5);
    bench_sum(0.8);

    bench_min(0.001);
    bench_min(0.05);
    bench_min(0.5);
    bench_min(0.8);

    bench_collect(0.001);
    bench_collect(0.05);
    bench_collect(0.5);
    bench_collect(0.8);

    bench_scaling(0.05, 8);
    bench_scaling(0.5, 8);
    bench_scaling(0.8, 8);

    for sz in  0..10 {
        bench_sets(0.05, sz);
    }

    for sz in  0..10 {
        bench_sets(0.5, sz);
    }
}
