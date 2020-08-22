use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;
use std::ptr::copy_nonoverlapping;

#[derive(Copy, Clone)]
pub struct Slab {
    sp: *const u8,
    heap: *mut u8,
    size: usize,
}

struct Fib {}

impl Fib {
    fn new() -> Self {
        Self {}
    }

    #[inline(never)]
    fn __calc_inner(&mut self, n: usize, s: Slab) -> usize {
        // big stack allocation
        test::black_box(unsafe { MaybeUninit::<[u8; 180]>::uninit().assume_init() });

        let v;
        if (n == 0) || (n == 1) {
            v = n
        } else {
            v = self.calc(n - 1, Some(s)) + self.calc(n - 2, Some(s));
        }
        v
    }

    #[inline(never)]
    fn calc(&mut self, n: usize, s: Option<Slab>) -> usize {
        let __v; // Return value
        let mut __s; // Allocated or inherited stack slab,
        let mut __discard = false; // Owns the stack `Slab` thus need to dealloc it at the end
        let copy = 512; // TODO: figure out how many stack bytes we need to copy over when allocating a new stack `Slab`

        unsafe {
            // Save current stack pointer
            let mut sp: *const u8 = std::ptr::null();
            asm!("mov {sp}, rsp", sp = out(reg) sp,);

            if let Some(slab) = s {
                // Block still active
                let offset = sp.offset_from(slab.heap).abs() as usize;
                if offset < 1024 {
                    // Running out of space allocate a new block with more
                    let size = slab.size * 2;
                    let layout = Layout::from_size_align_unchecked(size, 32);
                    let heap = alloc(layout);
                    __discard = true;

                    __s = Slab { sp, heap, size };

                    let mut vsp = heap.add(size - copy);
                    // Copy the current stack frame to the new stack before activating it
                    copy_nonoverlapping(sp, vsp, copy);
                    // Activate block
                    asm!("mov rsp, {vsp}", vsp = in(reg) vsp,);
                } else {
                    __s = slab;
                }
            } else {
                let size = 64 * 1024;
                let layout = Layout::from_size_align_unchecked(size, 32);
                let heap = alloc(layout);

                __discard = true;
                __s = Slab { sp, heap, size };

                let mut vsp = heap.add(size - copy);
                // Copy the current stack frame to the new stack before activating it
                copy_nonoverlapping(sp, vsp, copy);
                // Activate stack slab
                asm!("mov rsp, {vsp}", vsp = in(reg) vsp,);
            }
        }

        // Call function
        __v = self.__calc_inner(n, __s);

        unsafe {
            // Stack `Slab` changed!
            if __discard {
                let layout = Layout::from_size_align_unchecked(__s.size, 32);
                dealloc(__s.heap, layout);

                // Restore to previous `Slab` or the frame in the program stack
                asm!(
                    //"mov {v}, {v}", // Make the sp reg not the same as the return value reg
                    "mov rsp, {sp}",
                    //v = in(reg) __v,
                    sp = in(reg) __s.sp
                );
            }
        }

        // Return result
        __v
    }
}

pub fn exec() {
    let f = test::black_box(Fib::new().calc(40, None));
    println!("fib(40): {}", f);
}
