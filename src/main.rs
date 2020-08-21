#![feature(test)]
#![feature(asm)]
#![feature(ptr_offset_from)]

// use std::alloc::{alloc, dealloc, Layout};
// use std::mem::{ManuallyDrop, MaybeUninit};

extern crate test;

mod complex;
mod simple;

fn main() {
    // simple::exec();
    // println!("simple done!");
    complex::exec();
    println!("complex done!");
}

// #[inline(never)]
// fn fib(n: usize) -> usize {
//     // big stack allocation
//     test::black_box(unsafe { MaybeUninit::<[u8; 4096]>::uninit().assume_init() });

//     if (n == 0) || (n == 1) {
//         n
//     } else {
//         fib(n - 1) + fib(n - 2)
//     }
// }

// fn main() {
//     const SIZE: usize = 2 * 1024 * 1024;
//     let layout = Layout::from_size_align(SIZE, 8).unwrap();
//     let stack_ptr = unsafe { alloc(layout) };

//     let vsp = unsafe { stack_ptr.add(SIZE) };
//     let mut sp: *const usize = std::ptr::null();

//     unsafe {
//         asm!(
//             // "nop",
//             // "nop",
//             "mov {sp}, rsp",
//             "mov rsp, {vsp}",
//             // "nop",
//             // "nop",
//             sp = out(reg) sp,
//             vsp = in(reg) vsp,
//         );
//     }

//     fib(10);

//     unsafe {
//         asm!(
//             "mov rsp, {sp}",
//             sp = in(reg) sp,
//         );
//     }

//     unsafe {
//         dealloc(stack_ptr, layout);
//     }
// }

// fn main() {
//     unsafe {
//         let layout = Layout::from_size_align(64, 8).unwrap();
//         let stack_ptr = alloc(layout);

//         let vsp = stack_ptr.add(64);
//         let mut sp: *const usize = std::ptr::null();

//         asm!(
//             "mov {sp}, rsp",
//             "mov rsp, {vsp}",
//             "push {number}",
//             "mov rsp, {sp}",
//             sp = out(reg) sp,
//             vsp = in(reg) vsp,
//             number = const 5,
//         );

//         let v7 = *(stack_ptr as *const usize).add(7);
//         println!("{}", v7);

//         dealloc(stack_ptr, layout)
//     }
// }
