use gamedon_bits::HwReg8;

#[derive(Debug, Clone, Copy, Default)]
struct Serial {
    /// 0xFF01: Serial data port.
    data: HwReg8,
    // 0xFF02: Serial data control.
    control: HwReg8,
}
