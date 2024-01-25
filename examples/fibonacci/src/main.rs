#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    core::hint::black_box(fib(1000));
    0
}

#[inline]
fn fib(i: usize) {
    let (mut a, mut b) = (0, 1);
    for _ in 0..i {
        // use inline assembly so it does not get optimized out
        unsafe {
            core::arch::asm!(
                "mv {tmp}, {b}",
                "add {b}, {a}, {b}",
                "mv {a}, {tmp}",
                a = inout(reg) a,
                b = inout(reg) b,
                tmp = out(reg) _,
            )
        };
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
