use fifo::Queue;
use gamedon_bus::{MemoryBus, PpuState};

mod drawer;
mod fifo;
mod pixel;

/// The number of dots per scanline.
const NUM_DOTS_PER_LINE: u16 = 456;
/// The number of scanlines per frame.
const NUM_LINES_PER_FRAME: u8 = 154;
/// The height of the viewport in pixels.
const VIEWPORT_HEIGHT: u8 = 144;
/// The number of dots (in time) per OAM scan state.
const DOTS_PER_OAM_SCAN: u16 = 80;

#[derive(Debug, Clone, Default)]
pub struct Ppu {
    dot_index: u16,
    pixel_queue: Queue<16, u8>,
}

impl Ppu {
    pub fn step(&mut self, bus: &mut MemoryBus) {
        let mode = bus.get_lcd_mode();

        match mode {
            PpuState::HBlank => self.handle_hblank(bus),
            PpuState::VBlank => self.handle_vblank(bus),
            PpuState::OamScan => self.handle_oam_scan(bus),
            PpuState::HDraw => todo!(),
        }
        self.dot_index += 1;
    }

    const fn handle_vblank(&mut self, bus: &mut MemoryBus) {
        if self.dot_index < NUM_DOTS_PER_LINE {
            return;
        }

        bus.increment_scanline();

        // Enter OAM scan if number of scanlines per frame is reached
        if bus.get_scanline() >= NUM_LINES_PER_FRAME {
            bus.set_lcd_mode(PpuState::OamScan);
            // Reset frame

            // Increment frame counter
        }

        // Reset the dot index
        self.dot_index = 0;
    }

    const fn handle_hblank(&mut self, bus: &mut MemoryBus) {
        if self.dot_index < NUM_DOTS_PER_LINE {
            return;
        }
        bus.increment_scanline();

        // Enter VBlank if past screen
        if bus.get_scanline() >= VIEWPORT_HEIGHT {
            bus.set_lcd_mode(PpuState::VBlank);
        }
        // Otherwise enter OAM scan
        else {
            bus.set_lcd_mode(PpuState::OamScan);
        }

        // Reset the dot index
        self.dot_index = 0;
    }

    const fn handle_oam_scan(&mut self, bus: &mut MemoryBus) {
        if self.dot_index == 0 {
            // Load sprites that are on current scanline
        }

        if self.dot_index < DOTS_PER_OAM_SCAN {
            return;
        }
        // Enter HDraw at the end of OAM scan
        bus.set_lcd_mode(PpuState::HDraw);
    }

    const fn handle_hdraw(&mut self, bus: &mut MemoryBus) {
        let viewport_x = bus.get_viewport_x();
        let viewport_y = bus.get_viewport_y();
        let current_scanline = bus.get_scanline();
    }
}
