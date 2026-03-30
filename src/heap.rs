use std::collections::{BinaryHeap, HashSet};
use std::cmp::Reverse;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct HeapNode {
    next_run: chrono::DateTime<chrono::Utc>,
    task_id: Uuid,
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.next_run.cmp(&other.next_run)
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TaskHeap {
    heap: BinaryHeap<Reverse<HeapNode>>,
    index: HashSet<Uuid>,
}

impl TaskHeap {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            index: HashSet::new(),
        }
    }

    /// push: 保证 task_id 互斥，已存在则跳过
    pub fn push(&mut self, task_id: Uuid, next_run: chrono::DateTime<chrono::Utc>) -> bool {
        if self.index.contains(&task_id) {
            return false;
        }
        
        self.index.insert(task_id);
        self.heap.push(Reverse(HeapNode { next_run, task_id }));
        true
    }

    /// remove: 延迟删除，只从 index 移除
    pub fn remove(&mut self, task_id: &Uuid) -> bool {
        self.index.remove(task_id)
    }

    /// pop_due: 返回到期且有效的任务
    pub fn pop_due(&mut self, now: chrono::DateTime<chrono::Utc>) -> Vec<Uuid> {
        let mut due = Vec::new();

        while let Some(Reverse(node)) = self.heap.peek() {
            if !self.index.contains(&node.task_id) {
                self.heap.pop();
                continue;
            }

            if node.next_run > now {
                break;
            }

            let node = self.heap.pop().unwrap().0;
            self.index.remove(&node.task_id);
            due.push(node.task_id);
        }

        due
    }

    pub fn next_deadline(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        for Reverse(node) in self.heap.iter() {
            if self.index.contains(&node.task_id) {
                return Some(node.next_run);
            }
        }
        None
    }

    /// is_empty: 返回堆是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// len: 返回堆中元素个数
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.index.len()
    }
}

impl Default for TaskHeap {
    fn default() -> Self {
        Self::new()
    }
}