use crate::process::{ChildEvent, Process, ProcessState, RegisterContext};
use crate::sparse_vec::SparseVec;
use crate::system_control;
use crate::timer;
use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use drivers::core_timer;

type Pid = usize;

struct Scheduler {
    process_table: SparseVec<Box<Process>>,
    run_queue: VecDeque<Pid>,
    current_pid: Option<Pid>,
    active: bool,
}

static mut SCHEDULER: Option<Scheduler> = None;

pub fn init() {
    unsafe {
        SCHEDULER = Some(Scheduler {
            process_table: SparseVec::new(),
            run_queue: VecDeque::new(),
            current_pid: None,
            active: false,
        });
    }
    timer::init();
}

fn schedule_timer_handler() {
    plan_scheduling();
    core_timer::set_remaining_time(core_timer::Virtual, 10_000_000);
}

pub fn start() -> ! {
    core_timer::register_callback(core_timer::Virtual, schedule_timer_handler, false);
    core_timer::set_enabled(core_timer::Virtual, true);
    core_timer::set_remaining_time(core_timer::Virtual, 10_000_000);
    plan_scheduling();
    unsafe {
        system_control::set_mode(system_control::ProcessorMode::System);
        asm!("svc 0" ::: "lr" : "volatile");
    }
    panic!("Scheduler did not start")
}

extern "C" {
    // This assembly function waits indefinitely
    fn idle() -> !;
}

pub fn check_schedule(active_ctx: &mut RegisterContext) {
    // Wait end of syscall for rescheduling
    if active_ctx.psr & 0b11111 == system_control::ProcessorMode::Supervisor as u32 {
        return;
    }

    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };

    if scheduler.active {
        scheduler.active = false;
        print!(".");

        if let Some(pid) = scheduler.current_pid {
            let current_process = &mut scheduler.process_table[pid];

            current_process.save_context(active_ctx);
            if current_process.state == ProcessState::Runnable {
                scheduler.run_queue.push_back(pid);
            }
        }

        scheduler.current_pid = scheduler.run_queue.pop_front();
        match scheduler.current_pid {
            Some(pid) => {
                let next_active_process = &mut scheduler.process_table[pid];
                next_active_process.restore_context(active_ctx);
            }
            None => {
                active_ctx.pc = idle as *const u32;
                active_ctx.psr = system_control::ProcessorMode::System as u32;
            }
        }
    }
}

pub fn plan_scheduling() {
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
    scheduler.active = true;
}

pub fn add_process(process: Box<Process>) -> Pid {
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };

    let pid = scheduler.process_table.insert(process);
    scheduler.process_table[pid].pid = pid;

    if scheduler.process_table[pid].state == ProcessState::Runnable {
        scheduler.run_queue.push_back(pid);
    }

    pid
}

pub fn get_process<'a>(pid: Pid) -> Option<&'a mut Process> {
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
    scheduler.process_table.get_mut(pid).map(|x| &mut **x)
}

pub fn current_pid() -> Option<Pid> {
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
    scheduler.current_pid
}

pub fn current_process<'a>() -> Option<&'a mut Process> {
    get_process(current_pid()?)
}

pub fn remove_process(pid: Pid) -> Option<Box<Process>> {
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };

    let killed_process = scheduler.process_table.remove(pid)?;
    if killed_process.state == ProcessState::Runnable {
        suspend_process(pid);
    }
    if scheduler.current_pid == Some(pid) {
        scheduler.current_pid = None;
    }

    // Reattach all children to process 0 (init)
    for child_pid in &killed_process.children_pid {
        scheduler.process_table[*child_pid].parent_pid = 0
    }

    Some(killed_process)
}

pub fn suspend_process(pid: Pid) {
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };

    if scheduler.current_pid == Some(pid) {
        plan_scheduling(); // check_schedule will stop the current process
    } else {
        let run_queue = scheduler
            .run_queue
            .drain(..)
            .filter(|x| *x != pid)
            .collect();
        scheduler.run_queue = run_queue;
    }
}

pub fn resume_process(pid: Pid) {
    let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
    scheduler.run_queue.push_back(pid);
}

pub fn send_child_event(reciever_pid: Pid, ev: ChildEvent) {
    if let Some(reciever) = get_process(reciever_pid) {
        if reciever.state == ProcessState::WaitingChildren {
            reciever.regs.r0 = ev.pid as u32;
            reciever.regs.r1 = ev.exit_code;
            reciever.state = ProcessState::Runnable;
            resume_process(reciever_pid);
        } else {
            reciever.child_events.push(ev);
        }
    }
}
