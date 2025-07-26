use crate::Cpu;

/// The boot ROM to use.
#[derive(Debug, Clone, Copy)]
pub enum BootRom {
    // Monochrome models
    /// The boot ROM for the DMG-01 (Original Gameboy) model.
    Dmg,
    /// The boot ROM for the MGB (Gameboy Pocket) model.
    Mgb,
    // Super Game Boy
    /// The boot ROM for the SGB (Super Gameboy) model.
    Sgb,
    /// The boot ROM for the SGB2 (Super Gameboy 2) model.
    Sgb2,
    // Color models
    /// The boot ROM for the CGB (Gameboy Color) model.
    Cgb,
    // Debug
    /// The boot ROM register values to be compatible with the `gameboy-doctor` utility.
    Doctor,
}

impl Cpu {
    /// Set the state after the given boot `rom`.
    #[inline]
    pub fn boot(&mut self, rom: BootRom) {
        match rom {
            BootRom::Dmg => self.boot_dmg(),
            BootRom::Doctor => self.boot_doctor(),
            rom => {
                todo!("implement boot rom {rom:?}")
            }
        }
    }

    /// Set the state after the DMG boot rom has run.
    fn boot_dmg(&mut self) {
        // Set register flags
        self.registers.set_zero_flag(true);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);

        // Set 8-bit register values
        self.registers.set_a(0x01);
        self.registers.set_b(0x00);
        self.registers.set_c(0x13);
        self.registers.set_d(0x00);
        self.registers.set_e(0xD8);
        self.registers.set_h(0x01);
        self.registers.set_l(0x4D);

        // Set 16-bit register values
        self.registers.set_pc(0x0100);
        self.registers.set_sp(0xFFFE);
    }

    /// Set the state that `gameboy-doctor` is in after its boot rom.
    fn boot_doctor(&mut self) {
        // Set register flags
        self.registers.set_zero_flag(true);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(true);
        self.registers.set_carry_flag(true);

        // Set 8-bit register values
        self.registers.set_a(0x01);
        self.registers.set_b(0x00);
        self.registers.set_c(0x13);
        self.registers.set_d(0x00);
        self.registers.set_e(0xD8);
        self.registers.set_h(0x01);
        self.registers.set_l(0x4D);

        // Set 16-bit register values
        self.registers.set_pc(0x0100);
        self.registers.set_sp(0xFFFE);
    }
}
