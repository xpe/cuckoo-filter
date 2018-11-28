#![feature(refcell_replace_swap)]

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Filter {
    /// Fingerprint bit length
    finger_bits: u8,

    /// Number of buckets
    num_buckets: u32,

    /// Number of entries per bucket
    num_entries: u8,

    /// Max swaps
    max_swaps: u8,

    /// Bucket type
    bucket_type: BucketType,

    /// Buckets
    buckets: RefCell<Buckets>,

    /// Entries used (occupied)
    used: RefCell<u64>,

    /// Random number generator
    rng: RefCell<ThreadRng>,
}

#[derive(Debug)]
enum BucketType {
    U8,
    U16
}

#[derive(Debug)]
enum Buckets {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

#[derive(Debug)]
pub struct Config {
    /// Fingerprint bit length
    pub finger_bits: u8,

    /// Number of buckets
    pub num_buckets: u32,

    /// Number of entries per bucket
    pub num_entries: u8,

    /// Max swaps
    pub max_swaps: u8,
}

impl Filter{
    pub fn new(c: &Config) -> Result<Filter, ()> {
        match Filter::init_buckets(c.num_buckets, c.num_entries, c.finger_bits) {
            Ok((buckets, bucket_type)) => {
                Ok(Filter {
                    finger_bits: c.finger_bits,
                    num_buckets: c.num_buckets,
                    num_entries: c.num_entries,
                    max_swaps: c.max_swaps,
                    bucket_type,
                    buckets: RefCell::new(buckets),
                    used: RefCell::new(0),
                    rng: RefCell::new(thread_rng()),
                })
            }
            Err(_) => Err(()),
        }
    }
}

impl Filter {
    pub fn used(&self) -> u64 {
        *self.used.borrow_mut()
    }

    pub fn capacity(&self) -> u64 {
        self.num_buckets as u64 * self.num_entries as u64
    }

    pub fn load_factor(&self) -> f64 {
        self.used() as f64 / self.capacity() as f64
    }

    pub fn bits(&self) -> u64 {
        self.capacity() * self.finger_bits as u64
    }
}

impl Filter {
    pub fn insert<T: ?Sized + Hash>(&self, x: &T) -> Result<u8, u8> where T: Debug {
        let result = match self.bucket_type {
            BucketType::U8 => self.insert_u8(x),
            BucketType::U16 => self.insert_u16(x),
        };
        if result.is_ok() {
            self.used.replace_with(|&mut x| x + 1);
        }
        result
    }
}

impl Filter {
    fn insert_u8<T: ?Sized + Hash>(&self, x: &T) -> Result<u8, u8> {
        let (finger, idx_1) = self.finger8_index(x);

        // Try to place fingerprint in empty entry
        if self.try_insert_u8(idx_1, finger) {
            return Ok(0);
        }
        let idx_2 = self.index(&finger);
        if self.try_insert_u8(idx_2, finger) {
            return Ok(0);
        }

        // Must relocate existing items
        let mut rng = self.rng.borrow_mut();
        let mut idx = *([idx_1, idx_2].choose(&mut *rng).unwrap());
        let mut finger = finger;
        for swaps in 1 ..= self.max_swaps {
            let entry = rng.gen_range(0, self.num_entries);
            finger = self.swap_u8(idx, entry, finger);
            idx = self.index(&finger);
            if self.try_insert_u8(idx, finger) {
                return Ok(swaps);
            }
        }
        return Err(self.max_swaps);
    }

    fn insert_u16<T: ?Sized + Hash>(&self, x: &T) -> Result<u8, u8> {
        let (finger, idx_1) = self.finger16_index(x);

        // Try to place fingerprint in empty entry
        if self.try_insert_u16(idx_1, finger) {
            return Ok(0);
        }
        let idx_2 = self.index(&finger);
        if self.try_insert_u16(idx_2, finger) {
            return Ok(0);
        }

        // Must relocate existing items
        let mut rng = self.rng.borrow_mut();
        let mut idx = *([idx_1, idx_2].choose(&mut *rng).unwrap());
        let mut finger = finger;
        for swaps in 1 ..= self.max_swaps {
            let entry = rng.gen_range(0, self.num_entries);
            finger = self.swap_u16(idx, entry, finger);
            idx = self.index(&finger);
            if self.try_insert_u16(idx, finger) {
                return Ok(swaps);
            }
        }
        return Err(self.max_swaps);    }
}

impl Filter {
    fn try_insert_u8(&self, bucket: u32, finger: u8) -> bool {
        match *self.buckets.borrow_mut() {
            Buckets::U8(ref mut vec) => {
                let entries = self.num_entries as usize;
                let start = bucket as usize * entries;
                for i in start .. (start + entries) {
                    if vec[i] == 0 {
                        vec[i] = finger;
                        return true;
                    }
                }
            },
            _ => unimplemented!(),
        }
        false
    }

    fn try_insert_u16(&self, index: u32, finger: u16) -> bool {
        match *self.buckets.borrow_mut() {
            Buckets::U16(ref mut vec) => {
                let entries = self.num_entries as usize;
                let start = index as usize * entries;
                for i in start .. (start + entries) {
                    if vec[i] == 0 {
                        vec[i] = finger;
                        return true;
                    }
                }
            },
            _ => unimplemented!(),
        }
        false
    }
}

impl Filter {
    fn swap_u8(&self, index: u32, entry: u8, finger: u8) -> u8 {
        match *self.buckets.borrow_mut() {
            Buckets::U8(ref mut vec) => {
                let i = index as usize * self.num_entries as usize + entry as usize;
                let x = vec[i];
                vec[i] = finger;
                x
            },
            _ => unimplemented!(),
        }
    }

    fn swap_u16(&self, index: u32, entry: u8, finger: u16) -> u16 {
        match *self.buckets.borrow_mut() {
            Buckets::U16(ref mut vec) => {
                let i = index as usize * self.num_entries as usize + entry as usize;
                let x = vec[i];
                vec[i] = finger;
                x
            },
            _ => unimplemented!(),
        }
    }
}

impl Filter {
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        let entries = self.num_entries as usize;
        match *self.buckets.borrow() {
            Buckets::U8(ref vec) => {
                let n = vec.len();
                for (i, x) in vec.iter().enumerate() {
                    if i % entries == 0 {
                        s.push_str(&format!("{:3} [", i / entries));
                    }
                    s.push_str(&format!(" {:3} ", x));  // 2 ^ 8 requires 3 digits
                    if i % entries == entries - 1 {
                        if i == n - 1 {
                            s.push_str("]");
                        } else {
                            s.push_str("]\n");
                        }
                    }
                }
            },
            Buckets::U16(ref vec) => {
                let n = vec.len();
                for (i, x) in vec.iter().enumerate() {
                    if i % entries == 0 {
                        s.push_str(&format!("{:3} [", i / entries));
                    }
                    s.push_str(&format!(" {:5} ", x)); // 2 ^ 16 requires 5 digits
                    if i % entries == entries - 1 {
                        if i == n - 1 {
                            s.push_str("]");
                        } else {
                            s.push_str("]\n");
                        }
                    }
                }
            }
        }
        s
    }
}

impl Filter {
    fn init_buckets(num_buckets: u32, num_entries: u8, finger_bits: u8)
        -> Result<(Buckets, BucketType), ()> {
        let n = num_buckets as usize * num_entries as usize;
        if finger_bits == 8 {
            Ok((Buckets::U8(vec![0u8; n]), BucketType::U8))
        } else if finger_bits == 16 {
            Ok((Buckets::U16(vec![0u16; n]), BucketType::U16))
        } else {
            Err(())
        }
    }
}

impl Filter {
    /// Hashes an arbitrary value and returns (fingerprint, index).
    /// Fingerprint cannot be 0.
    fn finger8_index<T: ?Sized + Hash>(&self, x: &T) -> (u8, u32) {
        let h = hash64(x);
        let finger = ((h >> 32) % 255) as u8 + 1u8;
        let index = (h as u32) % self.num_buckets;
        (finger, index)
    }

    /// Hashes an arbitrary value and returns (fingerprint, index).
    /// Fingerprint cannot be 0.
    fn finger16_index<T: ?Sized + Hash>(&self, x: &T) -> (u16, u32) {
        let h = hash64(x);
        let finger = ((h >> 32) % 65535) as u16 + 1u16;
        let index = (h as u32) % self.num_buckets;
        (finger, index)
    }

    /// Hashes an arbitrary value.
    fn index<T: ?Sized + Hash>(&self, x: &T) -> u32 {
        (hash64(x) as u32) % self.num_buckets
    }
}

/// Hashes an arbitrary value.
fn hash64<T: ?Sized + Hash>(x: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}
