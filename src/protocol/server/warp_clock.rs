/// TODO: is this correct? In CTR, certain tracks have a "warp clock" power-up that changes
/// the race rules. This packet broadcasts the current warp clock state
/// during a race.
///
/// TODO: Table
use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct WarpClock {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(pad_bits_after = "2")]
    _pad: (),

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub warp_clock: u8,
}
