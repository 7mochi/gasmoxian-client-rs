/// Sent during race to sync the finish countdown timer.
///
/// TODO: Table
use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct FinishTimer {
    #[deku(pad_bytes_before = "1")]
    #[deku(bits = "6", ctx = "deku::ctx::Order::Lsb0")]
    pub finish_timer: u8,

    #[deku(pad_bits_after = "2")]
    _pad: (),
}
