use super::mailbox;

#[repr(C, align(128))]
#[derive(Debug)]
pub struct FbData {
    width: u32,
    height: u32,
    virtual_width: u32,
    virtual_height: u32,
    pitch: u32,
    depth: u32,
    x_offset: u32,
    y_offset: u32,
    pointer: *mut u8,
    size: u32,
}

pub fn init(w: u32, h: u32) -> FbData {
    let mut data = FbData {
        width: w,
        height: h,
        virtual_width: w,
        virtual_height: h,
        pitch: 0,
        depth: 24,
        x_offset: 0,
        y_offset: 0,
        pointer: 0 as *mut u8,
        size: 0,
    };

    mailbox::send(1, &mut data);

    let foo = mailbox::receive(1);
    if foo != Some(0) {
        panic!("Failed to init framebuffer")
    }

    data
}
