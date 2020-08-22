# Virtual Stack

The stack will never overflow if you have enough physical memory!

====

### *NOTE* Don't use this code on production, it's just a proof of concept! 

### *NOTE* Only `x86_64` requires rust nightly

Uses some assembly trickery to change the `RSP` (stack register) to point to
a custom heap allocated memory address, and when it is nearly full allocate a
new slab of data on the heap to be used as an stack.

Although this crate only has an `x86_64` implementation the same principles
can be used for other architectures.

Where's how to use:

```rust
use virtual_stack::*;
// Recursive function
struct Fib;

impl Recursive<usize, usize> for Fib {
    fn call(n: usize, s: Option<StackFrame>) -> usize {
        if (n == 0) || (n == 1) {
            n
        } else {
            // Recursive call, stack never overflows, but you can run out memory!
            Fib::recursive_call(n - 1, s) + Fib::recursive_call(n - 2, s)
        }
    }
}

fn main() {
    // Frist call always uses the program stack
    Fib::call(10, None);
}
```
