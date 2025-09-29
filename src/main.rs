#![no_std]
#![no_main]

use core::panic::PanicInfo;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let msg = b"Mindada is awesome!";
    
    // Rainbow colours on white background: 0x7X where X is the colour
    let rainbow_colors = [0x74, 0x76, 0x7e, 0x72, 0x73, 0x71, 0x75];
    
    unsafe {
        for (i, &byte) in msg.iter().enumerate() {
            let offset = i * 2;
            let color_index = i % rainbow_colors.len();
            
            *VGA_BUFFER.offset(offset as isize) = byte;
            *VGA_BUFFER.offset((offset + 1) as isize) = rainbow_colors[color_index];
        }
    }
    
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}