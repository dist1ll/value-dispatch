/*
 * Copyright (c) Adrian Alic <contact@alic.dev>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
#![feature(test)]

use std::hint::black_box;

use rand::{
    distributions::{Bernoulli, Uniform, WeightedIndex},
    prelude::Distribution,
    thread_rng,
};
use test::Bencher;

extern crate test;

/*
Distribution:
Rep:    1  2  3  4  5  6  8  9  10
Type0: 85 10  5  0  0  0  0  0   0
Type1:  0  0  0  5 20 20 20 20   5
*/
fn create_stream(size: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(size + 16);
    let mut rng = thread_rng();

    // possible sequence lengths
    let choices = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // weighted index according to distribution
    // let weights_t0 = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
    // let weights_t1 = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
    // let weights_t0 = [80, 15, 5, 0, 0, 0, 0, 0, 0, 0];
    // let weights_t1 = [0, 0, 0, 30, 25, 15, 10, 5, 15, 0];
    let weights_t0 = [100, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let weights_t1 = [0, 0, 0, 0, 0, 0, 0, 0, 0, 100];

    let dist0 = WeightedIndex::new(&weights_t0).unwrap();
    let dist1 = WeightedIndex::new(&weights_t1).unwrap();

    let range_0 = 10;
    let dist0_value = Uniform::new_inclusive(10u8, 10u8 + range_0);

    let coinflip_classes = Bernoulli::new(1f64 / range_0 as f64).unwrap();
    loop {
        // take this branch with 90% probability
        if !coinflip_classes.sample(&mut rng) {
            for _ in 0..choices[dist0.sample(&mut rng)] {
                // push a number from group 0
                result.push(dist0_value.sample(&mut rng));
            }
        } 
        // take this branch with 10% probability
        else {
            for _ in 0..choices[dist1.sample(&mut rng)] {
                result.push(1);
            }
        }

        if result.len() >= size {
            // remove trailing elements
            for _ in 0..(result.len() - size) {
                result.pop();
            }
            break;
        }
    }
    assert!(result.len() == size);
    result
}
#[bench]
fn no_dispatch_impl(b: &mut Bencher) {
    let stream = create_stream(1 << 18);
    b.iter(|| {
        let mut current = stream[0];
        let mut skip = 0;
        for &next in &stream[1..] {
            if current == next {
                skip += 1;
            } else {
                current = next;
                black_box(skip);
                skip = 0;
            }
        }
        black_box(current);
    });
}
#[bench]
fn dispatch_impl(b: &mut Bencher) {
    let stream = create_stream(1 << 18);
    b.iter(|| {
        let mut current = stream[0];
        let mut skip = 0;

        let mut i = 0;
        while i < stream.len() {
            let mut next = 0;
            if current == 1 {
                while i < stream.len() {
                    next = stream[i];
                    if next != 1 {
                        break;
                    }
                    skip += 1;
                    i += 1;
                }
                current = next;
                black_box(skip);
                skip = 0;
            } else {
                while i < stream.len() {
                    next = stream[i];
                    if current != next {
                        break;
                    }
                    skip += 1;
                    i += 1;
                }
                current = next;
                black_box(skip);
                skip = 0;
            }
        }
        black_box(current);
    });
}
