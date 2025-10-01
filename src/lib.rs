                                                                              
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

use core::ptr::{addr_of, addr_of_mut};

const VGA: *mut u8 = 0xb8000 as *mut u8;
const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 25;

const REG_CTRL: u32 = 0x00000;
const REG_STATUS: u32 = 0x00008;
const REG_ICR: u32 = 0x000C0;
const REG_IMS: u32 = 0x000D0;
const REG_RCTL: u32 = 0x00100;
const REG_TCTL: u32 = 0x00400;
const REG_TIPG: u32 = 0x00410;
const REG_RDBAL: u32 = 0x02800;
const REG_RDBAH: u32 = 0x02804;
const REG_RDLEN: u32 = 0x02808;
const REG_RDH: u32 = 0x02810;
const REG_RDT: u32 = 0x02818;
const REG_TDBAL: u32 = 0x03800;
const REG_TDBAH: u32 = 0x03804;
const REG_TDLEN: u32 = 0x03808;
const REG_TDH: u32 = 0x03810;
const REG_TDT: u32 = 0x03818;
const REG_RAL: u32 = 0x05400;
const REG_RAH: u32 = 0x05404;

const TX_DESC_BASE: u32 = 0x200000;
const RX_DESC_BASE: u32 = 0x201000;
const TX_BUF_BASE: u32 = 0x202000;
const RX_BUF_BASE: u32 = 0x210000;

const NUM_TX_DESC: usize = 8;
const NUM_RX_DESC: usize = 8;

#[repr(C, packed)]
struct TxDesc {
    addr: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

#[repr(C, packed)]
struct RxDesc {
    addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

struct E1000 {
    base: u32,
    line: usize,
    tx_tail: usize,
    rx_tail: usize,
    mac: [u8; 6],
}

impl E1000 {
    fn new() -> Self {
        E1000 { 
            base: 0, 
            line: 0,
            tx_tail: 0,
            rx_tail: 0,
            mac: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56],
        }
    }

    fn scroll_screen(&mut self) {
        unsafe {
            for line in 0..(SCREEN_HEIGHT - 1) {
                for col in 0..SCREEN_WIDTH {
                    let src = VGA.add((line + 1) * SCREEN_WIDTH * 2 + col * 2);
                    let dst = VGA.add(line * SCREEN_WIDTH * 2 + col * 2);
                    *dst = *src;
                    *dst.add(1) = *src.add(1);
                }
            }
            for col in 0..SCREEN_WIDTH {
                let addr = VGA.add((SCREEN_HEIGHT - 1) * SCREEN_WIDTH * 2 + col * 2);
                *addr = b' ';
                *addr.add(1) = 0x07;
            }
        }
        self.line = SCREEN_HEIGHT - 1;
    }

    fn print(&mut self, msg: &[u8]) {
        if self.line >= SCREEN_HEIGHT {
            self.scroll_screen();
        }
        
        let colors = [0x74u8, 0x76, 0x7e, 0x72, 0x73, 0x71, 0x75];
        for i in 0..msg.len().min(SCREEN_WIDTH) {
            unsafe {
                *VGA.add(self.line * SCREEN_WIDTH * 2 + i * 2) = msg[i];
                *VGA.add(self.line * SCREEN_WIDTH * 2 + i * 2 + 1) = colors[i % colors.len()];
            }
        }
        self.line += 1;
    }

    fn write_reg(&self, reg: u32, val: u32) {
        unsafe {
            let addr = (self.base + reg) as *mut u32;
            core::ptr::write_volatile(addr, val);
            core::arch::asm!("mfence", options(nostack, preserves_flags));
        }
    }

    fn read_reg(&self, reg: u32) -> u32 {
        unsafe {
            let addr = (self.base + reg) as *const u32;
            let val = core::ptr::read_volatile(addr);
            core::arch::asm!("mfence", options(nostack, preserves_flags));
            val
        }
    }

    fn delay(&self, count: u32) {
        for _ in 0..count {
            unsafe { 
                core::arch::asm!("pause", options(nomem, nostack, preserves_flags)); 
            }
        }
    }

    fn find_device(&mut self) -> bool {
        self.print(b"Searching for E1000...");
        
        for bus in 0..8 {
            for slot in 0..32 {
                let vendor_device = pci_read(bus, slot, 0, 0);
                if vendor_device == 0xFFFFFFFF || vendor_device == 0 { 
                    continue; 
                }
                
                let vendor = vendor_device & 0xFFFF;
                let device = (vendor_device >> 16) & 0xFFFF;
                
                if vendor == 0x8086 && (device == 0x100E || device == 0x100F || device == 0x10D3) {
                    let cmd = pci_read(bus, slot, 0, 0x04);
                    pci_write(bus, slot, 0, 0x04, cmd | 0x07);
                    
                    let bar0 = pci_read(bus, slot, 0, 0x10);
                    self.base = bar0 & 0xFFFFFFF0;
                    
                    self.print(b"E1000 found!");
                    return true;
                }
            }
        }
        
        self.print(b"E1000 not found!");
        false
    }

    fn init(&mut self) {
        self.print(b"Init: Disable IRQ");
        self.write_reg(REG_IMS, 0);
        self.delay(1000);
        
        self.print(b"Init: Clear ICR");
        self.read_reg(REG_ICR);
        
        self.print(b"Init: Reset");
        self.write_reg(REG_CTRL, 0x04000000);
        
        self.print(b"Init: Wait for reset");
        for _ in 0..100000 {
            self.delay(10);
        }
        
        self.print(b"Init: Post-reset clear");
        self.read_reg(REG_ICR);
        self.write_reg(REG_IMS, 0);
        
        self.print(b"Init: Setup RX descriptors");
        unsafe {
            let rx_base = RX_DESC_BASE as *mut RxDesc;
            for i in 0..NUM_RX_DESC {
                let desc = rx_base.add(i);
                let buf_addr = (RX_BUF_BASE + (i as u32 * 2048)) as u64;
                
                core::ptr::write_volatile(addr_of_mut!((*desc).addr), buf_addr);
                core::ptr::write_volatile(addr_of_mut!((*desc).length), 0);
                core::ptr::write_volatile(addr_of_mut!((*desc).checksum), 0);
                core::ptr::write_volatile(addr_of_mut!((*desc).status), 0);
                core::ptr::write_volatile(addr_of_mut!((*desc).errors), 0);
                core::ptr::write_volatile(addr_of_mut!((*desc).special), 0);
            }
        }
        
        self.print(b"Init: Configure RX registers");
        self.write_reg(REG_RDBAL, RX_DESC_BASE);
        self.write_reg(REG_RDBAH, 0);
        self.write_reg(REG_RDLEN, (NUM_RX_DESC * 16) as u32);
        self.write_reg(REG_RDH, 0);
        self.write_reg(REG_RDT, (NUM_RX_DESC - 1) as u32);
        self.rx_tail = NUM_RX_DESC - 1;
        
        self.print(b"Init: Set MAC address");
        // MAC: 52:54:00:12:34:56
        self.write_reg(REG_RAL, 0x12005452);
        self.write_reg(REG_RAH, 0x00005634 | 0x80000000); // AV bit set
        
        self.print(b"Init: Enable receiver");
        // EN=1, UPE=1, MPE=1, BAM=1, BSIZE=2048, SECRC=1
        self.write_reg(REG_RCTL, 0x00000002 | 0x00000008 | 0x00000010 | 0x00008000 | 0x04000000 | (1 << 26));
        
        self.print(b"Init: Setup TX descriptors");
        unsafe {
            let tx_base = TX_DESC_BASE as *mut TxDesc;
            for i in 0..NUM_TX_DESC {
                let desc = tx_base.add(i);
                let buf_addr = (TX_BUF_BASE + (i as u32 * 2048)) as u64;
                
                core::ptr::write_volatile(addr_of_mut!((*desc).addr), buf_addr);
                core::ptr::write_volatile(addr_of_mut!((*desc).length), 0);
                core::ptr::write_volatile(addr_of_mut!((*desc).cso), 0);
                core::ptr::write_volatile(addr_of_mut!((*desc).cmd), 0);
                core::ptr::write_volatile(addr_of_mut!((*desc).status), 1);
                core::ptr::write_volatile(addr_of_mut!((*desc).css), 0);
                core::ptr::write_volatile(addr_of_mut!((*desc).special), 0);
            }
        }
        
        self.print(b"Init: Configure TX registers");
        self.write_reg(REG_TDBAL, TX_DESC_BASE);
        self.write_reg(REG_TDBAH, 0);
        self.write_reg(REG_TDLEN, (NUM_TX_DESC * 16) as u32);
        self.write_reg(REG_TDH, 0);
        self.write_reg(REG_TDT, 0);
        self.write_reg(REG_TIPG, 0x00702008);
        
        self.print(b"Init: Enable transmitter");
        self.write_reg(REG_TCTL, 0x00000002 | 0x00000008 | (15 << 4) | (63 << 12));
        
        self.tx_tail = 0;
        
        self.print(b"Completed init");
    }

    fn send_ping(&mut self, seq: u16) {
        self.print(b"Building ICMP packet");
        
        let buf = (TX_BUF_BASE + (self.tx_tail as u32 * 2048)) as *mut u8;
        let mut off = 0;
        
        unsafe {
            // Ethernet: destination MAC, source MAC, type
            for _ in 0..6 { *buf.add(off) = 0xFF; off += 1; }
            for i in 0..6 { *buf.add(off) = self.mac[i]; off += 1; }
            *buf.add(off) = 0x08; off += 1; // IPv4
            *buf.add(off) = 0x00; off += 1;
            
            // IP header
            let ip_start = off;
            *buf.add(off) = 0x45; off += 1; // Version 4, IHL 5
            *buf.add(off) = 0x00; off += 1; // DSCP/ECN
            *buf.add(off) = 0x00; off += 1; // Total length (high)
            *buf.add(off) = 0x54; off += 1; // Total length (low) = 84 bytes
            *buf.add(off) = 0x00; off += 1; // ID (high)
            *buf.add(off) = 0x01; off += 1; // ID (low)
            *buf.add(off) = 0x00; off += 1; // Flags/fragment (high)
            *buf.add(off) = 0x00; off += 1; // Flags/fragment (low)
            *buf.add(off) = 0x40; off += 1; // TTL = 64
            *buf.add(off) = 0x01; off += 1; // Protocol = ICMP
            let ip_csum_off = off;
            *buf.add(off) = 0x00; off += 1; // Checksum placeholder
            *buf.add(off) = 0x00; off += 1;

            // Source IP: 10.0.2.15 (QEMU default)
            *buf.add(off) = 0x0a; off += 1;
            *buf.add(off) = 0x00; off += 1;
            *buf.add(off) = 0x02; off += 1;
            *buf.add(off) = 0x0f; off += 1;

            // Destination IP: 10.0.2.2 (QEMU gateway)
            *buf.add(off) = 0x0a; off += 1;
            *buf.add(off) = 0x00; off += 1;
            *buf.add(off) = 0x02; off += 1;
            *buf.add(off) = 0x02; off += 1;
            
            let ip_csum = checksum(core::slice::from_raw_parts(buf.add(ip_start), 20));
            *buf.add(ip_csum_off) = (ip_csum >> 8) as u8;
            *buf.add(ip_csum_off + 1) = ip_csum as u8;
            
            // ICMP header
            let icmp_start = off;
            *buf.add(off) = 0x08; off += 1; // Echo request
            *buf.add(off) = 0x00; off += 1; // Code
            let icmp_csum_off = off;
            *buf.add(off) = 0x00; off += 1; // Checksum placeholder
            *buf.add(off) = 0x00; off += 1;
            *buf.add(off) = 0x00; off += 1; // ID (high)
            *buf.add(off) = 0x01; off += 1; // ID (low)
            *buf.add(off) = (seq >> 8) as u8; off += 1; // Sequence (high)
            *buf.add(off) = seq as u8; off += 1; // Sequence (low)
            
            // 56 bytes payload
            for i in 0..56 {
                *buf.add(off) = (0x20 + i) as u8;
                off += 1;
            }
            
            let icmp_csum = checksum(core::slice::from_raw_parts(buf.add(icmp_start), 64));
            *buf.add(icmp_csum_off) = (icmp_csum >> 8) as u8;
            *buf.add(icmp_csum_off + 1) = icmp_csum as u8;
            
            self.print(b"Packet build, sending");
            
            // TX descriptor
            let tx_desc = (TX_DESC_BASE as *mut TxDesc).add(self.tx_tail);
            core::ptr::write_volatile(addr_of_mut!((*tx_desc).length), off as u16);
            core::ptr::write_volatile(addr_of_mut!((*tx_desc).cmd), 0x0B); // EOP | IFCS | RS
            core::ptr::write_volatile(addr_of_mut!((*tx_desc).status), 0);
            
            core::arch::asm!("mfence", options(nostack, preserves_flags));
            
            let new_tail = (self.tx_tail + 1) % NUM_TX_DESC;
            self.write_reg(REG_TDT, new_tail as u32);
            self.tx_tail = new_tail;
            
            // Transmission
            let mut timeout = 0;
            loop {
                if core::ptr::read_volatile(addr_of!((*tx_desc).status)) & 1 != 0 {
                    self.print(b"TX Complete");
                    break;
                }
                self.delay(100);
                timeout += 1;
                if timeout > 10000 {
                    self.print(b"TX Timeout");
                    break;
                }
            }
        }
    }

    fn check_reply(&mut self) -> bool {
        unsafe {
            let next_rx = (self.rx_tail + 1) % NUM_RX_DESC;
            let rx_desc = (RX_DESC_BASE as *mut RxDesc).add(next_rx);
            
            let status = core::ptr::read_volatile(addr_of!((*rx_desc).status));
            if status & 1 != 0 {
                let len = core::ptr::read_volatile(addr_of!((*rx_desc).length)) as usize;
                let buf = (RX_BUF_BASE + (next_rx as u32 * 2048)) as *const u8;
                
                self.print(b"RX: Aquired packet");
                
                let is_reply = len > 34 && 
                               *buf.add(23) == 0x01 &&
                               *buf.add(34) == 0x00;
                
                if is_reply {
                    self.print(b"RX: ICMP echo reply");
                } else {
                    self.print(b"RX: Not an echo reply");
                }
                
                // Reset descriptor
                core::ptr::write_volatile(addr_of_mut!((*rx_desc).status), 0);
                core::ptr::write_volatile(addr_of_mut!((*rx_desc).length), 0);
                
                core::arch::asm!("mfence", options(nostack, preserves_flags));
                
                self.write_reg(REG_RDT, next_rx as u32);
                self.rx_tail = next_rx;
                
                return is_reply;
            }
        }
        false
    }
}

fn outl(port: u16, val: u32) {
    unsafe { core::arch::asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack)); }
}

fn inl(port: u16) -> u32 {
    let ret: u32;
    unsafe { core::arch::asm!("in eax, dx", in("dx") port, out("eax") ret, options(nomem, nostack)); }
    ret
}

fn pci_read(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let addr = 0x80000000u32 | ((bus as u32) << 16) | ((slot as u32) << 11) 
        | ((func as u32) << 8) | ((offset as u32) & 0xFC);
    outl(0xCF8, addr);
    inl(0xCFC)
}

fn pci_write(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
    let addr = 0x80000000u32 | ((bus as u32) << 16) | ((slot as u32) << 11) 
        | ((func as u32) << 8) | ((offset as u32) & 0xFC);
    outl(0xCF8, addr);
    outl(0xCFC, val);
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    
    while i < data.len() - 1 {
        sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !sum as u16
}

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