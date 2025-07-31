use gamedon_bus::MemoryBus;

/// The size of a tile in pixels.
const TILE_SIZE_IN_PIXELS: u8 = 8;

#[derive(Debug, Clone, Copy, Default)]
enum State {
    /// Get tile.
    #[default]
    GetTile,
    /// Get tile data low.
    GetTileDataLow,
    /// Get tile data high.
    GetTileDataHigh,
    /// Sleep.
    Sleep,
    /// Push.
    Push,
}

pub(crate) struct Drawer {
    /// The number of pixels rendered in the current scanline.
    num_rendered: u8,
    /// The number of pixels fetched in the current scanline.
    num_fetched: u8,
    /// The state.
    state: State,
}

impl Drawer {
    pub(crate) fn tick(&mut self, bus: &mut MemoryBus) {
        let viewport_x = bus.get_viewport_x();
        let viewport_y = bus.get_viewport_y();
        let current_scanline = bus.get_scanline();

        // The x-coordinate with respect to the background map i.e. the 256x256 pixel map
        let map_x = self.num_fetched.wrapping_add(viewport_x);
        // The y-coordinate with respect to the background map i.e. the 256x256 pixel map
        let map_y = current_scanline.wrapping_add(viewport_y);

        // The horizontal position in tile coordinates
        let tile_map_x = map_x / TILE_SIZE_IN_PIXELS;
        // The vertical position in tile coordinates
        let tile_map_y = map_y / TILE_SIZE_IN_PIXELS;

        // The current tile line i.e. 0-7 for 8x8 sprites and 0-15 for 8x16 sprites
        let current_tile_line = map_y % TILE_SIZE_IN_PIXELS;

        // TODO(pavyamsiri): Gamecrab is does this every other dot as otherwise it doesn't render properly.
        // Fetch
    }

    pub(crate) fn fetch(&mut self, bus: &mut MemoryBus) {
        match self.state {
            State::GetTile => {
                if bus.get_background_and_window_enable() {
                    let (tile_index, is_window_tile) =
                        PixelFetcher::fetch_background_or_window_tile_index(
                            bus,
                            tile_map_x,
                            tile_map_y,
                            self.window_line,
                            self.pixels_fetched,
                        );

                    self.background_window_fetch_buffer.tile_map_index = tile_index;
                    self.is_window_tile = is_window_tile;
                }
            }
            State::GetTileDataLow => todo!(),
            State::GetTileDataHigh => todo!(),
            State::Sleep => todo!(),
            State::Push => todo!(),
        }
    }
}
