use drivers::mmio;

const INTERRUPT_BASE : usize = (mmio::PERIPHERAL_BASE + 0xB000);

const IRQ_BASIC_PENDING : *const u32 = (INTERRUPT_BASE + 0x200) as *const u32;
const IRQ_GPU1_PENDING  : *const u32 = (INTERRUPT_BASE + 0x204) as *const u32;
const IRQ_GPU2_PENDING  : *const u32 = (INTERRUPT_BASE + 0x208) as *const u32;

const FIQ_CONTROL : *mut u32 = (INTERRUPT_BASE + 0x20C) as *mut u32;

const IRQ_GPU1_ENABLE  : *mut u32 = (INTERRUPT_BASE + 0x210) as *mut u32;
const IRQ_GPU2_ENABLE  : *mut u32 = (INTERRUPT_BASE + 0x214) as *mut u32;
const IRQ_BASIC_ENABLE : *mut u32 = (INTERRUPT_BASE + 0x218) as *mut u32;

const IRQ_GPU1_DISABLE  : *mut u32 = (INTERRUPT_BASE + 0x21C) as *mut u32;
const IRQ_GPU2_DISABLE  : *mut u32 = (INTERRUPT_BASE + 0x220) as *mut u32;
const IRQ_BASIC_DISABLE : *mut u32 = (INTERRUPT_BASE + 0x224) as *mut u32;

fn disabled_irq_handler()
{
    panic!("Disabled IRQ recieved")
}

fn disabled_fiq_handler()
{
    panic!("Disabled FIQ recieved")
}

static mut IRQ_HANDLERS : [fn(); 72] = [disabled_irq_handler; 72];
static mut FIQ_HANDLER : fn() = disabled_fiq_handler;

pub fn init()
{
    unsafe
    {
        // Disable all IRQ's
        mmio::write(IRQ_GPU1_DISABLE, 0xffff_ffff);
        mmio::write(IRQ_GPU2_DISABLE, 0xffff_ffff);
        mmio::write(IRQ_BASIC_DISABLE, 0xff);

        // Also disable FIQ
        mmio::write(FIQ_CONTROL, 0);

        // Enable interrupts on ARM side
        asm!("cpsie if" :::: "volatile");
    }
}

pub fn register_irq(id: u32, handler: fn())
{
    assert!(id < 72);

    let id = id as usize;
    unsafe
    {
        IRQ_HANDLERS[id] = handler;

        if id < 32
        {
            mmio::write(IRQ_GPU1_ENABLE, 1 << id);
        }
        else if id < 64
        {
            mmio::write(IRQ_GPU2_ENABLE, 1 << (id-32));
        }
        else
        {
            mmio::write(IRQ_BASIC_ENABLE, 1 << (id-64));
        }
    }
}

pub fn unregister_irq(id: u32)
{
    assert!(id < 72);

    let id = id as usize;
    unsafe
    {
        IRQ_HANDLERS[id] = disabled_irq_handler;

        if id < 32
        {
            mmio::write(IRQ_GPU1_DISABLE, 1 << id);
        }
        else if id < 64
        {
            mmio::write(IRQ_GPU2_DISABLE, 1 << (id-32));
        }
        else
        {
            mmio::write(IRQ_BASIC_DISABLE, 1 << (id-64));
        }
    }
}

#[no_mangle]
pub unsafe extern fn irq_handler()
{
    let gpu1_irq = mmio::read(IRQ_GPU1_PENDING);
    for i in 0..32
    {
        if gpu1_irq & (1 << i) != 0
        {
            IRQ_HANDLERS[i]();
        }
    }

    let gpu2_irq = mmio::read(IRQ_GPU2_PENDING);
    for i in 32..64
    {
        if gpu2_irq & (1 << (i-32)) != 0
        {
            IRQ_HANDLERS[i]();
        }
    }

    let basic_irq = mmio::read(IRQ_BASIC_PENDING);
    for i in 64..72
    {
        if basic_irq & (1 << (i-64)) != 0
        {
            IRQ_HANDLERS[i]();
        }
    }
}

pub fn register_fiq(id: u32, handler: fn())
{
    assert!(id < 72);

    unsafe
    {
        mmio::write(FIQ_CONTROL, 0);
        FIQ_HANDLER = handler;
        mmio::write(FIQ_CONTROL, id | (1 << 7));
    }
}

pub fn unregister_fiq()
{
    unsafe
    {
        mmio::write(FIQ_CONTROL, 0);
        FIQ_HANDLER = disabled_fiq_handler;
    }
}

#[no_mangle]
pub unsafe extern fn fiq_handler()
{
    FIQ_HANDLER()
}
