use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Special {
    #[deku(pad_bytes_before = "1")]
    #[deku(bytes = "1")]
    pub gamemodes: [bool; 18],
}
