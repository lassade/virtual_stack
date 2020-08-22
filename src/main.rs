#![feature(test)]
#![feature(asm)]
#![feature(ptr_offset_from)]

extern crate test;

use std::mem::MaybeUninit;
use test::black_box;

mod complex;
mod simple;

fn main() {
    const N: usize = 100_000;
    // simple::exec();
    // println!("simple done!");

    println!("virtualized stack");
    complex::rec(N);
    println!("done!");
    println!("-----------------------------------");
    println!("normal stack");
    rec(N);
    println!("done!");
}

#[inline(never)]
pub fn rec(n: usize) {
    black_box(unsafe { MaybeUninit::<[u8; 256]>::uninit().assume_init() });
    if n != 0 {
        black_box(rec(n - 1))
    }
}
