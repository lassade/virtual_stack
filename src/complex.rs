use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;
use std::ptr::copy_nonoverlapping;
use test::black_box;

#[derive(Copy, Clone)]
struct Slab {
    sp: *const u8,
    heap: *mut u8,
    size: usize,
}

struct Rec {}

impl Rec {
    #[inline(always)]
    fn __rec_inner(n: usize, s: Slab) {
        black_box(unsafe { MaybeUninit::<[u8; 256]>::uninit().assume_init() });
        if n != 0 {
            black_box(Self::rec(n - 1, Some(s)))
        }
    }

    #[inline(never)]
    fn rec(n: usize, s: Option<Slab>) {
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
                asm!("nop");
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

            // asm!(
            //     "push {0}",
            //     "push {1}",
            //     "push {2}",
            //     in(reg) __s.sp, in(reg) __s.heap, in(reg) __s.size,
            // );
        }

        // Call function
        __v = Self::__rec_inner(n, __s);

        unsafe {
            // asm!(
            //     "pop {2}",
            //     "pop {1}",
            //     "pop {0}",
            //     out(reg) __s.sp, out(reg) __s.heap, out(reg) __s.size,
            // );

            // Stack `Slab` changed!
            if __discard {
                // Restore to previous `Slab` or the frame in the program stack
                asm!(
                    "nop", // Make the sp reg not the same as the return value reg
                    "mov rsp, {sp}",
                    //v = in(reg) __v,
                    sp = in(reg) __s.sp
                );

                let layout = Layout::from_size_align_unchecked(__s.size, 32);
                dealloc(__s.heap, layout);
            }
        }

        // Return result
        __v
    }
}

pub fn rec(n: usize) {
    black_box(Rec::rec(n, None));
}
