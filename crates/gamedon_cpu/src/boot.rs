use crate::Cpu;

#[derive(Debug, Clone, Copy)]
pub enum BootRom {
    // Monochrome models
    Dmg0,
    Dmg,
    Mgb,
    // Super Game Boy
    Sgb,
    Sgb2,
    // Color models
    Cgb0,
    Cgb,
    Agb0,
    Agb,
}

impl Cpu {
    pub fn boot(&mut self, rom: BootRom) {
        match rom {
            BootRom::Dmg => self.boot_dmg(),
            _ => {
                // TODO(pavyamsiri): Implement boot roms
                todo!()
            }
        }
    }

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
}
