//                                _______                _______                 
//  __  __   ___   .--.   _..._   \  ___ `'.             \  ___ `'.              
// |  |/  `.'   `. |__| .'     '.  ' |--.\  \             ' |--.\  \             
// |   .-.  .-.   '.--..   .-.   . | |    \  '            | |    \  '            
// |  |  |  |  |  ||  ||  '   '  | | |     |  '    __     | |     |  '    __     
// |  |  |  |  |  ||  ||  |   |  | | |     |  | .:--.'.   | |     |  | .:--.'.   
// |  |  |  |  |  ||  ||  |   |  | | |     ' .'/ |   \ |  | |     ' .'/ |   \ |  
// |  |  |  |  |  ||  ||  |   |  | | |___.' /' `" __ | |  | |___.' /' `" __ | |  
// |__|  |__|  |__||__||  |   |  |/_______.'/   .'.''| | /_______.'/   .'.''| |  
//                     |  |   |  |\_______|/   / /   | |_\_______|/   / /   | |_ 
//                     |  |   |  |             \ \._,\ '/             \ \._,\ '/ 
//                     '--'   '--'              `--'  `"               `--'  `"  

#![no_std]

mod vga;
mod pci;
mod net;
mod drivers;

use drivers::e1000::E1000;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let mut nic = E1000::new();
    
    nic.print(b"Mindada is awesome!");
    nic.print(b"");
    
    if nic.find_device() {
        nic.init();
        nic.print(b"");
        nic.print(b"-== Starting ping loop ==-");
        nic.print(b"Pinging 10.0.2.2 (gateway)");
        nic.print(b"");
        
        let mut seq = 1u16;
        loop {
            nic.print(b"----");
            nic.print(b"Sending ping...");
            nic.send_ping(seq);
            
            nic.print(b"Waiting for reply...");
            let mut got_reply = false;
            
            for _ in 0..50000 {
                if nic.check_reply() {
                    got_reply = true;
                    break;
                }
                nic.delay(10);
            }
            
            if got_reply {
                nic.print(b"Success: Got reply!");
            } else {
                nic.print(b"TimeOut: No reply ):");
            }
            
            seq = seq.wrapping_add(1);
            
            for _ in 0..100000 {
                nic.delay(10);
            }
        }
    }
    
    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
}
