use sparse_vec::SparseVec;
use process::{Process, ProcessState, RegisterContext};
use alloc::VecDeque;
use alloc::boxed::Box;
use system_control;
use drivers::core_timer;

type Pid = usize;

struct Scheduler
{
    process_table : SparseVec<Box<Process>>,
    run_queue : VecDeque<Pid>,
    current_pid : Option<Pid>,
    active : bool,
}

static mut SCHEDULER : Option<Scheduler> = None;

pub fn init()
{
    unsafe
    {
        SCHEDULER = Some(Scheduler { process_table: SparseVec::new(),
            run_queue: VecDeque::new(), current_pid: None,
            active: false});
    }
}

fn schedule_timer_handler()
{
    plan_scheduling();
    core_timer::set_remaining_time(core_timer::Virtual, 10_000_000);
}

pub fn start() -> !
{
    core_timer::register_callback(core_timer::Virtual, schedule_timer_handler, false);
    core_timer::set_enabled(core_timer::Virtual, true);
    core_timer::set_remaining_time(core_timer::Virtual, 10_000_000);
    plan_scheduling();
    unsafe
    {
        system_control::set_mode(system_control::ProcessorMode::System);
        asm!("svc 0" ::: "lr" : "volatile");
    }
    panic!("Scheduler did not start")
}

extern
{
    // This assembly function waits indefinitely
    fn idle() -> !;
}

pub fn check_schedule(active_ctx: &mut RegisterContext)
{
    // Wait end of syscall for rescheduling
    if active_ctx.psr & 0b11111 == system_control::ProcessorMode::Supervisor as u32
    {
        return;
    }

    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };

    if scheduler.active
    {
        scheduler.active = false;
        print!(".");

        match scheduler.current_pid
        {
            Some(pid) =>
            {
                let current_process = &mut scheduler.process_table[pid];

                current_process.save_context(active_ctx);
                if current_process.state == ProcessState::Runnable
                {
                    scheduler.run_queue.push_back(pid);
                }
            },
            None => ()
        }

        scheduler.current_pid = scheduler.run_queue.pop_front();
        match scheduler.current_pid
        {
            Some(pid) =>
            {
                let next_active_process = &mut scheduler.process_table[pid];
                next_active_process.restore_context(active_ctx);
            },
            None =>
            {
                active_ctx.pc = idle as *const u32;
                active_ctx.psr = system_control::ProcessorMode::System as u32;
            }
        }
    }
}

pub fn plan_scheduling()
{
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
    scheduler.active = true;
}

pub fn add_process(process: Box<Process>) -> Pid
{
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };

    let pid = scheduler.process_table.insert(process);
    scheduler.process_table[pid].pid = pid;

    if scheduler.process_table[pid].state == ProcessState::Runnable
    {
        scheduler.run_queue.push_back(pid);
    }

    pid
}
