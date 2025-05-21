// //!Implementation of [`TaskManager`]
// use super::TaskControlBlock;
// use crate::sync::UPSafeCell;
// use alloc::collections::VecDeque;
// use alloc::sync::Arc;
// use lazy_static::*;
// ///A array of `TaskControlBlock` that is thread-safe
// pub struct TaskManager {
//     ready_queue: VecDeque<Arc<TaskControlBlock>>,
// }

// /// A simple FIFO scheduler.
// impl TaskManager {
//     ///Creat an empty TaskManager
//     pub fn new() -> Self {
//         Self {
//             ready_queue: VecDeque::new(),
//         }
//     }
//     /// Add process back to ready queue
//     pub fn add(&mut self, task: Arc<TaskControlBlock>) {
//         self.ready_queue.push_back(task);
//     }
//     /// Take a process out of the ready queue
//     pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
//         self.ready_queue.pop_front()
//     }
// }

// lazy_static! {
//     /// TASK_MANAGER instance through lazy_static!
//     pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
//         unsafe { UPSafeCell::new(TaskManager::new()) };
// }

// /// Add process to ready queue
// pub fn add_task(task: Arc<TaskControlBlock>) {
//     //trace!("kernel: TaskManager::add_task");
//     TASK_MANAGER.exclusive_access().add(task);
// }

// /// Take a process out of the ready queue
// pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
//     //trace!("kernel: TaskManager::fetch_task");
//     TASK_MANAGER.exclusive_access().fetch()
// }


//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
//use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;

use core::cmp::Ordering;
use crate::config::BIG_STRIDE ;
use alloc::collections::binary_heap::BinaryHeap;


///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    //ready_queue: VecDeque<Arc<TaskControlBlock>>,
    stride_queue: BinaryHeap<StrideTask>,
}

///A array of `StrideTask`
pub struct StrideTask {
    task: Arc<TaskControlBlock>,
    pass: usize,
}

/// Stride 
impl TaskManager {
    ///Creat an empty queue
    pub fn new() -> Self {
        Self {
            // ready_queue: VecDeque::new(),
            // 定义大根堆
            stride_queue: BinaryHeap::new(),
        }
    }
    /// Add process back to stride queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        // self.ready_queue.push_back(task);
        self.stride_queue.push(StrideTask::new(task));
        
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // self.ready_queue.pop_front()
        if let Some(mut stride_task) = self.stride_queue.pop() {
            stride_task.stride_add();
            let task = Arc::clone(&stride_task.task);
            Some(task)
        } else {
            None
        }
        
    }
}


impl StrideTask {
    pub fn new(task: Arc<TaskControlBlock>) -> Self {
        let priority = task.inner_exclusive_access().priority;
        let pass = BIG_STRIDE / priority;
        Self {
            task,
            pass,
        }
    }
    pub fn stride_add(&mut self) {
        self.task.inner_exclusive_access().stride += self.pass;
    }
}



lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}


// 大根堆
impl PartialEq for StrideTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.inner_exclusive_access().stride == other.task.inner_exclusive_access().stride
    }
}
impl Eq for StrideTask {}
impl PartialOrd for StrideTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for StrideTask {
    fn cmp(&self, other: &Self) -> Ordering {
        other.task.inner_exclusive_access().stride.cmp(&self.task.inner_exclusive_access().stride)
    }
}