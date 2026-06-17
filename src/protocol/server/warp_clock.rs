use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct WarpClock {
    // === BYTE 0 ===
    // 1. Consumimos los 4 bits bajos del tipo de mensaje
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    // 2. Saltamos los 2 bits intermedios (bits 4 y 5) que no se usan en este paquete
    #[deku(pad_bits_after = "2")]
    _pad: (),

    // 3. Leemos los últimos 2 bits altos (bits 6 y 7) que son tu warp_clock
    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub warp_clock: u8,
}
