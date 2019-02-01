pub fn read_u16(block: &[u8], pos: usize) -> u16 {
    (block[pos] as u16) + (block[pos + 1] as u16) * 0x100
}

pub fn read_u32(block: &[u8], pos: usize) -> u32 {
    (block[pos] as u32)
        + (block[pos + 1] as u32) * 0x100
        + (block[pos + 2] as u32) * 0x10000
        + (block[pos + 3] as u32) * 0x1000000
}

pub fn write_u16(block: &mut [u8], pos: usize, word: u16) {
    block[pos] = (word >> 8) as u8;
    block[pos + 1] = word as u8;
}

pub fn write_u32(block: &mut [u8], pos: usize, word: u32) {
    block[pos] = (word >> 24) as u8;
    block[pos + 1] = (word >> 16) as u8;
    block[pos + 2] = (word >> 8) as u8;
    block[pos + 3] = word as u8;
}
