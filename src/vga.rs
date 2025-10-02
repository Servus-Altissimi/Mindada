const VGA: *mut u8 = 0xb8000 as *mut u8;
const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 25;

pub struct VgaWriter {
    line: usize,
}

impl VgaWriter {
    pub fn new() -> Self {
        VgaWriter { line: 0 }
    }

    pub fn scroll_screen(&mut self) {
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

    pub fn print(&mut self, msg: &[u8]) {
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

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn set_line(&mut self, line: usize) {
        self.line = line;
    }
}
