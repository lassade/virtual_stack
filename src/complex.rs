use std::alloc::{alloc, dealloc, Layout};
use std::mem::MaybeUninit;
use std::ptr::copy_nonoverlapping;

struct Block {
    sp: *const u8,
    heap: *mut u8,
    size: usize,
}

struct VirtualStack {
    blocks: Vec<Block>,
}

impl VirtualStack {
    #[inline(always)]
    fn new() -> Self {
        Self { blocks: vec![] }
    }
}

struct Fib {
    virtual_stack: VirtualStack,
}

impl Fib {
    fn new() -> Self {
        Self {
            virtual_stack: VirtualStack::new(),
        }
    }

    #[inline(never)]
    fn __calc_code(&mut self, n: usize) -> usize {
        println!("n: {}", n);

        // big stack allocation
        test::black_box(unsafe { MaybeUninit::<[u8; 180]>::uninit().assume_init() });

        if (n == 0) || (n == 1) {
            n
        } else {
            self.calc(n - 1) + self.calc(n - 2)
        }
    }

    #[inline(never)]
    fn calc(&mut self, n: usize) -> usize {
        // Should the block be discarded at the end?
        let __v;
        let mut __discard = false;
        let mut __sp: *const u8 = std::ptr::null();

        unsafe {
            // Save current stack pointer
            asm!("mov {sp}, rsp", sp = out(reg) __sp,);

            let __stack = &mut self.virtual_stack;
            if let Some(block) = __stack.blocks.pop() {
                // Block still active
                let offset = __sp.offset_from(block.heap).abs() as usize;
                println!("offset: {}", offset);
                if offset < 1024 {
                    println!("change block");

                    // Running out of space allocate a new block with more
                    let size = block.size * 2;
                    let layout = Layout::from_size_align_unchecked(size, 8);
                    let heap = alloc(layout);
                    __discard = true;

                    __stack.blocks.push(block);
                    __stack.blocks.push(Block {
                        sp: __sp,
                        heap,
                        size,
                    });

                    // Activate block
                    let vsp = heap.add(size);
                    asm!("mov rsp, {vsp}", vsp = in(reg) vsp);
                } else {
                    // Block still has some space left
                    __stack.blocks.push(block);
                }
            } else {
                let size = 4096;
                let layout = Layout::from_size_align_unchecked(size, 8);
                let heap = alloc(layout);

                __discard = true;
                __stack.blocks.push(Block {
                    sp: __sp,
                    heap,
                    size,
                });

                // Activate block
                let vsp = heap.add(size);
                asm!("mov rsp, {vsp}", vsp = in(reg) vsp,);
            }
        }

        // Call function
        __v = self.__calc_code(n);

        unsafe {
            // Restore stack pointer
            asm!("mov rsp, {sp}", sp = in(reg) __sp,);

            // Discard block
            if __discard {
                let __stack = &mut self.virtual_stack;
                let block = __stack.blocks.pop().unwrap();
                let layout = Layout::from_size_align_unchecked(block.size, 8);
                dealloc(block.heap, layout);
            }
        }

        // Return result
        __v
    }
}

pub fn exec() {
    test::black_box(Fib::new().calc(10));
}
