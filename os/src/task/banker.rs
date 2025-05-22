use alloc::collections::{BTreeMap, BTreeSet};

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
        let mut work = self.available.clone();
        let mut finish = BTreeMap::new();

        // 获取所有涉及的线程ID
        let tids: BTreeSet<_> = self.need
            .keys()
            .chain(
                self.allocation.values()
                    .flat_map(|m| m.keys())
            )
            .copied()
            .collect();

        // 初始化完成状态
        for &tid in &tids {
            finish.insert(tid, false);
        }

        let mut found;
        loop {
            found = false;
            
            // 遍历所有线程
            for &tid in &tids {
                // 跳过已完成的线程
                if *finish.get(&tid).unwrap_or(&false) {
                    continue;
                }

                // 检查资源需求是否可满足
                let can_allocate = self.need.get(&tid)
                    .map(|need_map| 
                        need_map.iter().all(|(&rid, &need)| 
                            *work.get(&rid).unwrap_or(&0) >= need
                        )
                    )
                    .unwrap_or(true); // 无需求的线程视为可立即完成

                if can_allocate {
                    // 回收该线程持有的所有资源
                    for (rid, alloc_map) in &self.allocation {
                        if let Some(&alloc) = alloc_map.get(&tid) {
                            *work.entry(*rid).or_insert(0) += alloc;
                        }
                    }
                    finish.insert(tid, true);
                    found = true;
                }
            }

            if !found {
                break;
            }
        }

        // 检查是否有未完成的线程
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