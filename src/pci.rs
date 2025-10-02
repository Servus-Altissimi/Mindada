fn outl(port: u16, val: u32) {
    unsafe { core::arch::asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack)); }
}

fn inl(port: u16) -> u32 {
    let ret: u32;
    unsafe { core::arch::asm!("in eax, dx", in("dx") port, out("eax") ret, options(nomem, nostack)); }
    ret
}

pub fn pci_read(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let addr = 0x80000000u32 | ((bus as u32) << 16) | ((slot as u32) << 11) 
        | ((func as u32) << 8) | ((offset as u32) & 0xFC);
    outl(0xCF8, addr);
    inl(0xCFC)
}

pub fn pci_write(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
    let addr = 0x80000000u32 | ((bus as u32) << 16) | ((slot as u32) << 11) 
        | ((func as u32) << 8) | ((offset as u32) & 0xFC);
    outl(0xCF8, addr);
    outl(0xCFC, val);
}
