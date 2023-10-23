//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    mm::page_table::translated_pa,
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_start_time,
        get_syscall_counter, get_task_status, mmap, munmap, suspend_current_and_run_next,
        TaskStatus,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let tz = get_time_us();
    let ts_pa = translated_pa(current_user_token(), ts as *const u8) as *mut TimeVal;

    unsafe {
        (*ts_pa) = TimeVal {
            sec: tz / 1000000,
            usec: tz % 1000000,
        }
    }

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");

    let ti_pa = translated_pa(current_user_token(), ti as *const u8) as *mut TaskInfo;

    unsafe {
        (*ti_pa).status = get_task_status();
        (*ti_pa).time = (get_time_us() - get_start_time()) / 1000;
        (*ti_pa).syscall_times = get_syscall_counter();
    }

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if start & 0xfff != 0 {
        return -1;
    } else if port & !0x7 != 0 {
        return -1;
    } else if port & 0x7 == 0 {
        return -1;
    } else {
        mmap(start, len, port)
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if start & 0xfff != 0 {
        return -1;
    } else {
        munmap(start, len)
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
