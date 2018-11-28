use cuckoo_filter::{Config, Filter};
use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric};

pub fn main() {
    println!("Cuckoo Filter");
    let mut rng = thread_rng();
    run_experiment(&mut rng);
}

fn run_experiment<R>(rng: &mut R) where R: Rng {
    let config = Config {
        finger_bits: 16,    //    16      8      8     8     8
        num_buckets: 10000, // 20000  20000  10000  5000  4000
        num_entries: 100,   //    50     50    100   200   250
        max_swaps: 99,
    };
    println!("Experiment : config={:?}", config);
    match Filter::new(&config) {
        Ok(f) => {
            let n = 990000;
            let mut words = words(rng, n);
            rng.shuffle(&mut words);
            let mut summary = Summary::new(config.max_swaps as usize + 1);
            for (i, word) in words.iter().enumerate() {
                let (status, swaps) = insert(&f, word);
                summary.update(i, status, swaps);
            }
            println!("load_factor   : {}", f.load_factor());
            println!("bits          : {}", f.bits());
            println!("bits_per_key  : {}", f.bits() as f64 / n as f64);
            summary.print_status();
        }
        Err(_) => {
            println!("Bucket type does not have enough bits");
        }
    }
}

struct Summary {
    success: u64,
    failure: u64,
    swaps: u64,
    first_failure: usize,
    swap_histogram: Vec<usize>,
}

impl Summary {
    fn new(bins: usize) -> Summary {
        Summary {
            success: 0,
            failure: 0,
            swaps: 0,
            first_failure: 0,
            swap_histogram: vec![0; bins],
        }
    }

    fn update(&mut self, i: usize, status: bool, swaps: u64) {
        self.swaps += swaps;
        self.swap_histogram[swaps as usize] += 1;
        if status {
            self.success += 1;
        } else {
            if self.failure == 0 {
                self.first_failure = i;
            }
            self.failure += 1;
        }
    }

    fn print_status(&self) {
        println!("first_failure : {}", self.first_failure);
        println!("success       : {:8}", self.success);
        println!("failure       : {:8}", self.failure);
        println!("swaps         : {:8}", self.swaps);
        for (i, x) in self.swap_histogram.iter().enumerate() {
            println!("{:2} {:8}", i, *x);
        }
    }
}

fn insert(f: &Filter, x: &str) -> (bool, u64) {
    match f.insert(x) {
        Ok(swaps) => {
            // println!("{:20}   success   {:2} swaps", x, swaps);
            (true, swaps as u64)
        },
        Err(swaps) => {
            // println!("{:20}   failure   {:2} swaps", x, swaps);
            (false, swaps as u64)
        }
    }
}

fn words<R>(rng: &mut R, n: usize) -> Vec<String> where R: Rng {
    let mut vec = Vec::with_capacity(n);
    for i in 0 .. n {
        let s = rand_string(rng, 4);
        vec.push(format!("{}_{}", s, i));
    }
    vec
}

fn rand_string<R>(rng: &mut R, k: usize) -> String where R: Rng {
    rng.sample_iter(&Alphanumeric).take(k).collect()
}

