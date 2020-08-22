use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;

#[inline(never)]
fn fib(n: usize) -> usize {
    // big stack allocation
    test::black_box(unsafe { MaybeUninit::<[u8; 180]>::uninit().assume_init() });

    if (n == 0) || (n == 1) {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

// ! FIXME: Broken
pub fn exec() {
    const SIZE: usize = 2 * 1024 * 1024;
    let layout = Layout::from_size_align(SIZE, 8).unwrap();
    let stack_ptr = unsafe { alloc(layout) };

    let vsp = unsafe { stack_ptr.add(SIZE) };
    let mut sp: *const usize = std::ptr::null();

    unsafe {
        asm!(
            // "nop",
            // "nop",
            "mov {sp}, rsp",
            "mov rsp, {vsp}",
            // "nop",
            // "nop",
            sp = out(reg) sp,
            vsp = in(reg) vsp,
        );
    }

    fib(40);

    unsafe {
        asm!(
            "mov rsp, {sp}",
            sp = in(reg) sp,
        );
    }

    unsafe {
        dealloc(stack_ptr, layout);
    }
}
