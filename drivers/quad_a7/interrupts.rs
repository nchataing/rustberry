use mmio;
use quad_a7;
use bcm2708;

const IRQ_SOURCE_BASE : usize = quad_a7::PERIPHERAL_BASE + 0x60;
const FIQ_SOURCE_BASE : usize = quad_a7::PERIPHERAL_BASE + 0x70;

const GPU_INTERRUPT_ROUTING : *mut u32 = (quad_a7::PERIPHERAL_BASE + 0x0C) as *mut u32;

pub fn init()
{
    bcm2708::interrupts::init();
}

pub fn handle_irq()
{
    let proc_id = quad_a7::get_core_id() as usize;
    let irq_source_reg = (IRQ_SOURCE_BASE + 4*proc_id) as *const u32;
    let irq_source = unsafe { mmio::read(irq_source_reg) };

    for mailbox_id in 0 .. 4
    {
        if irq_source & (1 << (mailbox_id + 4)) != 0
        {
            quad_a7::mailbox::handle_interrupt(mailbox_id);
        }
    }
    if irq_source & (1 << 8) != 0
    {
        bcm2708::interrupts::handle_irq();
    }
}

pub fn handle_fiq()
{
    let proc_id = quad_a7::get_core_id() as usize;
    let fiq_source_reg = (FIQ_SOURCE_BASE + 4*proc_id) as *const u32;
    let fiq_source = unsafe { mmio::read(fiq_source_reg) };

    for mailbox_id in 0 .. 4
    {
        if fiq_source & (1 << (mailbox_id + 4)) != 0
        {
            quad_a7::mailbox::handle_interrupt(mailbox_id);
        }
    }
    if fiq_source & (1 << 8) != 0
    {
        bcm2708::interrupts::handle_fiq();
    }
}

pub fn set_gpu_interrupt_routing(irq_core_id: u8, fiq_core_id: u8)
{
    assert!(irq_core_id < 4);
    assert!(fiq_core_id < 4);

    let val = (irq_core_id | (fiq_core_id << 2)) as u32;
    unsafe
    {
        mmio::write(GPU_INTERRUPT_ROUTING, val);
    }
}

