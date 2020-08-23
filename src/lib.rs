#![feature(test)]
#![feature(asm)]
#![feature(ptr_offset_from)]

#[cfg(test)]
extern crate test;

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::copy_nonoverlapping;

/// Virtual stack frame
#[derive(Copy, Clone)]
pub struct StackFrame {
    sp: *const u8,
    heap: *mut u8,
    size: usize,
}

/// Implements a recursive function with configurable virtual stack behavior
///
/// ```rust
/// use virtual_stack::*;
///
/// // Recursive function
/// struct Fib;
///
/// impl Recursive<usize, usize> for Fib {
///     fn call(n: usize, s: Option<StackFrame>) -> usize {
///         if (n == 0) || (n == 1) {
///             n
///         } else {
///             // Recursive call, stack never overflows, but you can run out memory!
///             Fib::recursive_call(n - 1, s) + Fib::recursive_call(n - 2, s)
///         }
///     }
/// }
///
/// fn main() {
///     // Frist call always uses the program stack
///     Fib::call(10, None);
/// }
/// ```
pub trait Recursive<T, R = ()> {
    /// Start size of the virtualized stack
    ///
    /// A bigger stack prevents allocations as the code reuses the current stack,
    /// while a smaller stack will need less memory.
    const SIZE: usize = 16 * 1024;
    /// Number of bytes left in the stack to be considered full enough
    /// to need allocate another one
    ///
    /// Must be always bigger than `Self::COPY`, bigger values wastes more memory
    /// but lower values may lead to stack overflow
    const LEFT: usize = 1024;
    /// Copy this amount of bytes from the previous stack when allocating a
    /// new one.
    ///
    /// Provide a sensible amount that match your function need to run.
    const COPY: usize = 512;

    /// Main function logic, uses the previous virtual stack frame or none in case
    /// of the default program stack;
    ///
    /// **NOTE** Any recursive call must use `recursive_call` instead
    fn call(arg: T, s: Option<StackFrame>) -> R;
}

/// Make call while managing the program stack to avoid stack overflow,
/// you will only runout of physical memory!
pub unsafe trait Caller<T, R> {
    fn recursive_call(arg: T, s: Option<StackFrame>) -> R;
}

/// Recursive virtual stack caller for `x86_64`
/// *NOTE* On debug mode it will need much more memory to work
#[cfg(target_arch = "x86_64")]
unsafe impl<T, R, V: Recursive<T, R>> Caller<T, R> for V {
    #[inline(never)]
    fn recursive_call(arg: T, s: Option<StackFrame>) -> R {
        let __v; // Return value
        let mut __s; // Allocated or inherited stack slab,
        let mut __discard = false; // Owns the stack `StackFrame` thus need to dealloc it at the end

        unsafe {
            // Save current stack pointer
            let sp: *const u8;
            asm!("mov {sp}, rsp", sp = out(reg) sp,);

            if let Some(slab) = s {
                // Block still active
                let offset = sp.offset_from(slab.heap).abs() as usize;
                if offset < memory_multiplayer(Self::LEFT) {
                    // Running out of space allocate a new block with more
                    let size = slab.size * 2;
                    let layout = Layout::from_size_align_unchecked(size, 32);
                    let heap = alloc(layout);
                    __discard = true;

                    __s = StackFrame { sp, heap, size };

                    let vsp = heap.add(size - memory_multiplayer(Self::COPY));
                    // Copy the current stack frame to the new stack before activating it
                    copy_nonoverlapping(sp, vsp, memory_multiplayer(Self::COPY));
                    // Activate block
                    asm!("mov rsp, {vsp}", vsp = in(reg) vsp,);
                } else {
                    __s = slab;
                }
            } else {
                let size = memory_multiplayer(Self::SIZE);
                let layout = Layout::from_size_align_unchecked(size, 32);
                let heap = alloc(layout);

                __discard = true;
                __s = StackFrame { sp, heap, size };

                let vsp = heap.add(size - memory_multiplayer(Self::COPY));
                // Copy the current stack frame to the new stack before activating it
                copy_nonoverlapping(sp, vsp, memory_multiplayer(Self::COPY));
                // Activate stack slab
                asm!("mov rsp, {vsp}", vsp = in(reg) vsp,);
            }
        }

        // Call function
        __v = Self::call(arg, Some(__s));

        unsafe {
            // Stack `StackFrame` changed!
            if __discard {
                // Restore to previous `StackFrame` or the frame in the program stack
                asm!("mov rsp, {sp}", sp = in(reg) __s.sp);

                let layout = Layout::from_size_align_unchecked(__s.size, 32);
                dealloc(__s.heap, layout);
            }
        }

        // Return result
        __v
    }
}

/// On debug mode it will automatically use 4 times more memory
/// for copy, allocate and left in the virtual stack
#[cfg(debug_assertions)]
const fn memory_multiplayer(n: usize) -> usize {
    n * 4
}

/// In release use the normal amount of memory
#[cfg(not(debug_assertions))]
const fn memory_multiplayer(n: usize) -> usize {
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::MaybeUninit;
    use test::black_box;

    struct Rec;

    impl Recursive<usize> for Rec {
        fn call(n: usize, s: Option<StackFrame>) -> () {
            black_box(unsafe { MaybeUninit::<[u8; 256]>::uninit().assume_init() });
            if n != 0 {
                black_box(Rec::recursive_call(n - 1, s))
            }
        }
    }

    struct Fib;

    impl Recursive<usize, usize> for Fib {
        fn call(n: usize, s: Option<StackFrame>) -> usize {
            black_box(unsafe { MaybeUninit::<[u8; 256]>::uninit().assume_init() });
            if (n == 0) || (n == 1) {
                n
            } else {
                Fib::recursive_call(n - 1, s) + Fib::recursive_call(n - 2, s)
            }
        }
    }

    #[test]
    fn deep() {
        const N: usize = 100_000;
        Rec::call(N, None);
    }

    #[test]
    fn fib() {
        Fib::call(40, None);
    }
}
