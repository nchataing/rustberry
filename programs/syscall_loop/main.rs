#![no_std]
#![feature(asm)]

extern crate rustberry_std as std;

static mut TABLE : [i32; 0x1000] = [0; 0x1000];

#[no_mangle]
pub extern fn main()
{
    loop
    {
        unsafe
        {
            TABLE[0] = 1;
            for i in 1 .. 0x1000
            {
                TABLE[i] = TABLE[i-1] * 2;
            }
            asm!("" :: "{r0}"(&TABLE) :: "volatile");

            asm!("svc 42" ::: "r0","r1","r2","r3","r12","lr","cc" : "volatile");
        }
    }
}
