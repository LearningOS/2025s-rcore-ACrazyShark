use alloc::{collections::BTreeMap};

/// Banker
pub struct Banker {
    // 可利用资源向量 Available: resource_id, count
    pub available: BTreeMap<usize, u8>,
    // 分配矩阵 Allocation: resource_id * <thread_id, counts>
    pub allocation: BTreeMap<usize, BTreeMap<usize, u8>>,
    // 需求矩阵 Need: thread_id * <resource_id, count>
    // 需求矩阵 Need: resource_id * <thread_id, count>
    pub need: BTreeMap<usize, BTreeMap<usize, u8>>
}

impl Banker {
    /// new
    pub fn new() -> Self {
        Self {
            available: BTreeMap::new(),
            allocation: BTreeMap::new(),
            need: BTreeMap::new(),
        }
    }

    /// 分配资源总数 MAX
    pub fn set_max_available(&mut self, resource_id: usize, count: usize) {
        self.available.insert(resource_id, count as u8);
    }


    /// 设置线程资源需求
    pub fn add_available(&mut self, resource_id: usize, count: usize) {
        // self.available.entry(resource_id).and_modify(|v| *v += count as u8);
        *self.available.entry(resource_id).or_insert(0) += count as u8;
    }


    /// 拒绝访问该线程
    pub fn bak_available(&mut self, thread_id: usize) {
        //self.available
            // .entry(thread_id)
            // .and_modify(|v| *v -= 1);

        if let Some(count) = self.available.get_mut(&thread_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }


    /// 分配资源到线程
    pub fn add_allocation(&mut self, thread_id: usize, resource_id: usize, count: usize) {
        // self.allocation
        //     .entry(resource_id)
        //     .or_default()
        //     .entry(thread_id)
        //     .and_modify(|v| *v += count as u8);
            //.or_insert(count as u8);
        *self.allocation.entry(thread_id).or_default().entry(resource_id).or_insert(0) += count as u8;
    }


    /// 拒绝访问该线程
    pub fn bak_allocation(&mut self, thread_id: usize, resource_id: usize) {
        self.need
            .entry(thread_id)
            .or_default()
            .entry(resource_id)
            .and_modify(|v| *v -= 1);
    }

    /// 设置线程资源需求
    pub fn add_need(&mut self, thread_id: usize, resource_id: usize, count: usize) {
        self.need
            .entry(resource_id)
            .or_default()
            .entry(thread_id)
            .and_modify(|v| *v += count as u8);
            // .or_insert(count as u8);
    }

    /// 拒绝访问该线程
    pub fn bak_need(&mut self, thread_id: usize, resource_id: usize) {
        self.need
            .entry(resource_id)
            .or_default()
            .entry(thread_id)
            .and_modify(|v| *v -= 1);
    }

    /// 安全检测，判断是否是不安全的
    pub fn is_unsafe(&self) -> bool {
        let mut work = self.available.clone();
        // 判断是否还有资源，没有的话，就不需要分配了

         // 如果没有可用资源，直接返回不安全
        if self.available.values().all(|&v| v == 0) {
            return true;
        }

        let mut finish = BTreeMap::new();
        
        for &tid in self.need.keys() {
            finish.insert(tid, false);
        }
        
        for alloc_map in self.allocation.values() {
            for &tid in alloc_map.keys() {
                finish.entry(tid).or_insert(false);
            }
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            
            // Check each thread
            for (&tid, completed) in &mut finish {
                println!("work: {:?}", work);
                if *completed {
                    continue;
                }
                
                // Check if thread's needs can be satisfied
                let mut can_allocate = true;
                if let Some(need_map) = self.need.get(&tid) {
                    for (&rid, &need) in need_map {
                        if *work.get(&rid).unwrap_or(&0) < need {
                            can_allocate = false;
                            break;
                        }
                    }
                }
                
                if can_allocate {
                    // Release allocated resources
                    for (rid, alloc_map) in &self.allocation {
                        if let Some(&alloc) = alloc_map.get(&tid) {
                            *work.entry(*rid).or_insert(0) += alloc;
                        }
                    }
                    *completed = true;
                    changed = true;
                }
            }
        }
        
        // Check if any thread didn't complete
        finish.values().any(|&completed| !completed)
        
    }

}

impl Clone for Banker {
    fn clone(&self) -> Self {
        Self {
            available: self.available.clone(),
            allocation: self.allocation.iter().map(|(k, v)| (*k, v.clone())).collect(),
            need: self.need.iter().map(|(k, v)| (*k, v.clone())).collect(),
        }
    }
}