pub fn is_printable(b: u8) -> bool {
    b >= 0x20 && b <= 0x7e
}

pub fn must_be_escaped(b: u8) -> bool {
    b == b'"' || b == b'\\'
}
