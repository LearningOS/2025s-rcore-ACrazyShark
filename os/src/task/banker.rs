use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

/// Banker
pub struct Banker {
    // 可利用资源向量 Available: resource_id, count
    pub available: BTreeMap<usize, u8>,
    // 分配矩阵 Allocation: resource_id * <thread_id, counts>
    pub allocation: BTreeMap<usize, BTreeMap<usize, u8>>,
    // 需求矩阵 Need: thread_id * <resource_id, count>
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
    /// 分配资源
    pub fn add_available(&mut self, resource_id: usize, count: usize) {
        self.available.entry(resource_id).and_modify(|v| *v += count as u8);
    }

    /// 拒绝访问该线程
    pub fn bak_available(&mut self, thread_id: usize) {
        self.available
            .entry(thread_id)
            .and_modify(|v| *v -= 1);
    }



    /// 分配资源到线程
    pub fn add_allocation(&mut self, thread_id: usize, resource_id: usize, count: usize) {
        self.allocation
            .entry(resource_id)
            .or_default()
            .entry(thread_id)
            .and_modify(|v| *v += count as u8);
            //.or_insert(count as u8);
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
            .entry(thread_id)
            .or_default()
            .entry(resource_id)
            .and_modify(|v| *v += count as u8);
            // .or_insert(count as u8);
    }

    /// 拒绝访问该线程
    pub fn bak_need(&mut self, thread_id: usize, resource_id: usize) {
        self.need
            .entry(thread_id)
            .or_default()
            .entry(resource_id)
            .and_modify(|v| *v -= 1);
    }


    /// 安全检查算法
    pub fn is_unsafe(&self) -> bool {
        let mut thread_ids = Vec::new();
        for &thread_id in self.allocation.keys() {
            if !thread_ids.contains(&thread_id) {
                thread_ids.push(thread_id);
            }
        }



        for &thread_id in self.need.keys() {
            if !thread_ids.contains(&thread_id) {
                thread_ids.push(thread_id);
            }
        }
        let mut work = self.available.clone();
        let mut finish = vec![false; thread_ids.len()];
        let mut found = true;
        while found {
            found = false;
            for (idx, &thread_id) in thread_ids.iter().enumerate() {
                if !finish[idx] {
                    let mut can_allocate = true;
                    
                    if let Some(thread_need) = self.need.get(&thread_id) {
                        for (&res_id, &need_count) in thread_need {
                            let available_count = *work.get(&res_id).unwrap_or(&0);
                            if need_count > available_count {
                                can_allocate = false;
                                break;
                            }
                        }
                    }
                    
                    if can_allocate {
                        if let Some(thread_alloc) = self.allocation.get(&thread_id) {
                            for (&res_id, &alloc_count) in thread_alloc {
                                *work.entry(res_id).or_insert(0) += alloc_count;
                            }
                        }
                        finish[idx] = true;
                        found = true;
                    }
                }
            }
        }
        

        finish.iter().any(|&x| !x)
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