pub const BERT_MAGIC_NUMBER: u8 = 131;
pub const SMALL_INTEGER_EXT: u8 = 97;
pub const INTEGER_EXT: u8 = 98;
pub const FLOAT_EXT: u8 = 99;
pub const ATOM_EXT: u8 = 100;
pub const SMALL_ATOM_EXT: u8 = 115;
pub const SMALL_TUPLE_EXT: u8 = 104;
pub const LARGE_TUPLE_EXT: u8 = 105;
pub const NIL_EXT: u8 = 106;
pub const STRING_EXT: u8 = 107;
pub const LIST_EXT: u8 = 108;
pub const BINARY_EXT: u8 = 109;
pub const SMALL_BIG_EXT: u8 = 110;
pub const LARGE_BIG_EXT: u8 = 111;
pub const ATOM_UTF8_EXT: u8 = 118;
pub const SMALL_ATOM_UTF8_EXT: u8 = 119;
pub const NEW_FLOAT_EXT: u8 = 70;
pub const MAP_EXT: u8 = 116;

pub const DISK_LOG_MAGIC: u32 = 0x01020304;
pub const DISK_LOG_OPENED: u32 = 0x06070809;
pub const DISK_LOG_CLOSED: u32 = 0x63584d0b;
pub const DISK_LOG_TERM_MAGIC: u32 = 0x62574c41;

pub const VERSION: &str = "0.9.3";
