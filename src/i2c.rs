use derive_more::{From, Into};

pub const ADDRESS: u8 = 0b110_0101;

bitfield::bitfield! {
    /// Defined in Table 7-8 (Command Code Definition)
    #[derive(Clone, Copy, PartialEq, Eq, From, Into, defmt::Format)]
    pub struct CommandCode(u8);
    impl Debug;
    u8;
    pub mode, set_mode: 7;
    pub offset, set_offset: 6, 0;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Block,
    Byte,
}

impl CommandCode {
    pub fn new(op_code: OpCode, offset: u8) -> Self {
        let mut i = Self(0);

        i.set_mode(op_code == OpCode::Byte);
        i.set_offset(offset);

        i
    }
}
