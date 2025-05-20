use crate::task::{current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::mm::{MapPermission, VirtAddr, translated_refmut};
use crate::task::{check_user_addr_range, get_syscall_count, insert_framed_area, free_framed_area};
// use crate::timer::{get_time_us};
use crate::config::{PAGE_SIZE};
use crate::timer::get_time_us;
// use crate::mm::PageTable;
use crate::task::change_program_brk;
use crate::task::{check_vpn_range, check_vpn_range_munmap};


#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
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
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time"); 
    let time_us = get_time_us();
    let token = current_user_token();

    if let Some(phy_addr) = translated_refmut::<TimeVal>(token, _ts){
        *phy_addr = TimeVal {
            sec: time_us / 1_000_000,
            usec: time_us % 1_000_000,
        };
        return 0
    }

    -1
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            if !check_user_addr_range(
                _id, 
                MapPermission::R | MapPermission::U
            ) {
                return -1;
            }

            // translate_read(_id)
            let ptr = _id as *mut u8;
            let token = current_user_token();
            if let Some(pa) = translated_refmut::<u8>(token, ptr){
                unsafe { pa.read_volatile() as isize }
            }
            -1

        },
        1 => {
            if !check_user_addr_range(
                _id as usize, 
                MapPermission::W | MapPermission::U
            ) {
                return -1;
            }

            let value = (_data & 0xFF) as u8;
            //translate_write(_id, value);
            let ptr = _id as *mut u8;
            let token = current_user_token();
            if let Some(pa) = translated_refmut::<u8>(token, ptr){
                unsafe { pa.write_volatile(value); }
                return 0
            }
            -1
        },
        2 =>{
            let num = get_syscall_count(_id);
            num
        }
        _ => -1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if (_port & !0x7) != 0 || (_port & 0x7) == 0 {
        return -1;
    }
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    let r = (_port & 0x1) != 0;
    let w = (_port & 0x2) != 0;
    let x = (_port & 0x4) != 0;
    let mut perm = MapPermission::U;
    if r {
        perm |= MapPermission::R;
    }
    if w {
        perm |= MapPermission::W;
    }
    if x {
        perm |= MapPermission::X;
    }
    if _len == 0 {
        return 0;
    }

    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);

    if check_vpn_range(start_va, end_va){
        insert_framed_area(
            start_va,
            end_va,
            perm,
        );
        return 0;
    }

    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);

    if check_vpn_range_munmap(start_va, end_va){
        free_framed_area(
            start_va,
            end_va,
        );
        return 0;
    }
    -1
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
