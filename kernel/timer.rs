use alloc::BinaryHeap;
use drivers::system_timer;
use drivers::system_timer::SystemTimer;
use process::ProcessState;
use scheduler;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Trigger {
    time: u32,
    pid: usize,
}

struct TimerHandler {
    next_wakeups: BinaryHeap<Trigger>,
    wakeup_trigger: Option<Trigger>,
}

static mut TIMER_HANDLER: Option<TimerHandler> = None;

pub fn init() {
    unsafe {
        TIMER_HANDLER = Some(TimerHandler {
            next_wakeups: BinaryHeap::new(),
            wakeup_trigger: None,
        });
    }
}

fn timer_callback() {
    let timer_handler = unsafe { TIMER_HANDLER.as_mut().unwrap() };

    let trigger = timer_handler.wakeup_trigger.unwrap();
    if let Some(process) = scheduler::get_process(trigger.pid) {
        if process.state == ProcessState::WaitingTimer {
            process.state = ProcessState::Runnable;
            scheduler::resume_process(trigger.pid)
        } else {
            warn!("Wakeup signal on already runnable process")
        }
    }

    match timer_handler.next_wakeups.pop() {
        Some(trig) => {
            system_timer::set_trigger_time(SystemTimer::Timer1, trig.time);
            timer_handler.wakeup_trigger = Some(trig);
        }
        None => {
            system_timer::unregister_callback(SystemTimer::Timer1);
            timer_handler.wakeup_trigger = None;
        }
    }

    system_timer::clear_irq(SystemTimer::Timer1);
}

pub fn add_wakeup_event(pid: usize, micro_secs: u32) {
    let timer_handler = unsafe { TIMER_HANDLER.as_mut().unwrap() };

    let time = system_timer::get_time_low().wrapping_add(micro_secs);
    let trig = Trigger { pid, time };
    match timer_handler.wakeup_trigger {
        Some(other_trig) => {
            if other_trig.time <= time {
                timer_handler.next_wakeups.push(trig);
            } else {
                timer_handler.wakeup_trigger = Some(trig);
                system_timer::set_trigger_time(SystemTimer::Timer1, time);
                timer_handler.next_wakeups.push(other_trig.clone());
            }
        }
        None => {
            timer_handler.wakeup_trigger = Some(trig);
            system_timer::set_trigger_time(SystemTimer::Timer1, time);
            system_timer::register_callback(SystemTimer::Timer1, timer_callback);
        }
    }
}
