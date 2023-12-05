#![no_std]
#![no_main]


#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    loop_fn();
    0
}

fn loop_fn() {
    let mut a = 0;
    for i in 0..14 {
        // use inline assembly so it does not get optimized out
        unsafe {
            core::arch::asm!(
                "add {a}, {a}, {i}",
                a = inout(reg) a,
                i = in(reg) i,
            )
        };
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

