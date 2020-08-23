# Virtual Stack

*The stack will never overflow if you have enough physical memory!*

[![](https://github.com/lassade/virtual_stack/workflows/Build/badge.svg)](https://github.com/lassade/virtual_stack/actions?query=workflow%3ABuild)

## Concept

Uses some assembly trickery to change the `RSP` (stack register) to point to
a custom heap allocated memory address, and when it is nearly full allocate a
new slab of data on the heap to be used as an stack.

**Debug and release modes have different stack requirements**, been the debug mode
much more memory hungry. This lib will use 4 times more memory on debug mode in an
attempt to make it work, but you may need to increase the amount of memory manually.

### Only `x86_64` and requires rust nightly

Although this crate only has a `x86_64` implementation the same principles
can be used for other architectures.

### Where's how to use it

```rust
use virtual_stack::*;
// Recursive function
struct Fib;

impl Recursive<usize, usize> for Fib {
    // You may also need to change the constants `SIZE`, `LEFT` and `COPY`
    // to better control the stack properties

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
