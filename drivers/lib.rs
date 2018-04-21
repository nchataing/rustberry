#![no_std]
#![feature(asm)]
#![allow(dead_code)]

#[macro_use] pub mod coproc_reg;
pub mod mmio;
mod bcm2708;
mod quad_a7;

pub use bcm2708::{uart, gpio, system_timer, video_core, random};
pub use quad_a7::{interrupts, get_core_id, mailbox, core_timer};
