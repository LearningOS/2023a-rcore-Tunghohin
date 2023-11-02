use core::ops::{AddAssign, SubAssign};

use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;

/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );

    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        process_inner.mutex_available[id] = 1;
        for i in process_inner.mutex_need.iter_mut() {
            i[id] = 0;
        }
        for i in process_inner.mutex_allocated.iter_mut() {
            i[id] = 0;
        }
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_available.push(1);
        for i in process_inner.mutex_need.iter_mut() {
            i.push(0);
        }
        for i in process_inner.mutex_allocated.iter_mut() {
            i.push(0);
        }
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());

    let thread_id = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    process_inner.mutex_need[thread_id][mutex_id].add_assign(1);

    if process_inner.deadlock_detect {
        let thread_count = process_inner.semaphore_need.len();
        let mut finished = vec![false; thread_count];
        let mut work = process_inner.mutex_available.clone();

        let mut releaseable = true;
        while releaseable {
            releaseable = false;
            for thread_id in 0..thread_count {
                if !finished[thread_id] {
                    let mut flag = false;
                    for mutex_index in 0..work.len() {
                        if process_inner.mutex_need[thread_id][mutex_index] > work[mutex_index] {
                            flag = true;
                            break;
                        }
                    }
                    if !flag {
                        finished[thread_id] = true;
                        work.iter_mut().enumerate().for_each(|(idx, available)| {
                            *available += process_inner.mutex_allocated[thread_id][idx];
                        });
                        releaseable = true;
                    }
                }
            }
        }
        for status in finished {
            if status == false {
                process_inner.mutex_need[thread_id][mutex_id].sub_assign(1);
                return -0xDEAD;
            }
        }
    }

    process_inner.mutex_need[thread_id][mutex_id].sub_assign(1);
    process_inner.mutex_available[mutex_id].sub_assign(1);
    process_inner.mutex_allocated[thread_id][mutex_id].add_assign(1);

    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());

    let thread_id = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    process_inner.mutex_available[mutex_id].add_assign(1);
    process_inner.mutex_allocated[thread_id][mutex_id].sub_assign(1);

    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        process_inner.semaphore_available[id] = res_count;
        for i in process_inner.semaphore_need.iter_mut() {
            i[id] = 0;
        }
        for i in process_inner.semaphore_allocated.iter_mut() {
            i[id] = 0;
        }
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_available.push(res_count);
        for i in process_inner.semaphore_need.iter_mut() {
            i.push(0);
        }
        for i in process_inner.semaphore_allocated.iter_mut() {
            i.push(0);
        }

        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();

    let thread_id = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    process_inner.semaphore_available[sem_id].add_assign(1);
    process_inner.semaphore_allocated[thread_id][sem_id].sub_assign(1);

    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let thread_id = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    process_inner.semaphore_need[thread_id][sem_id].add_assign(1);
    if process_inner.deadlock_detect {
        let thread_count = process_inner.semaphore_need.len();
        let mut finished = vec![false; thread_count];
        let mut work = process_inner.semaphore_available.clone();

        let mut releaseable = true;
        while releaseable {
            releaseable = false;
            for thread_id in 0..thread_count {
                if !finished[thread_id] {
                    let mut flag = false;
                    for semaphore_idx in 0..work.len() {
                        if process_inner.semaphore_need[thread_id][semaphore_idx]
                            > work[semaphore_idx]
                        {
                            flag = true;
                            break;
                        }
                    }
                    if !flag {
                        finished[thread_id] = true;
                        work.iter_mut().enumerate().for_each(|(idx, available)| {
                            *available += process_inner.semaphore_allocated[thread_id][idx];
                        });
                        releaseable = true;
                    }
                }
            }
        }
        for status in finished {
            if status == false {
                process_inner.semaphore_need[thread_id][sem_id].sub_assign(1);
                return -0xDEAD;
            }
        }
    }

    drop(process_inner);
    sem.down();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.semaphore_need[thread_id][sem_id].sub_assign(1);
    process_inner.semaphore_available[sem_id].sub_assign(1);
    process_inner.semaphore_allocated[thread_id][sem_id].add_assign(1);
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    if enabled == 0 {
        current_process().inner_exclusive_access().deadlock_detect = false;
        0
    } else if enabled == 1 {
        current_process().inner_exclusive_access().deadlock_detect = true;
        0
    } else {
        -1
    }
}
