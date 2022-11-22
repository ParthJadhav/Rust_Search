use std::{cmp::Ordering, time::SystemTime};

use ignore::DirEntry;

/// check if `dir`: [DirEntry] is created before `t`: [SystemTime]
pub fn created_before(dir: &DirEntry, t: SystemTime) -> bool {
    filter(FilterType::Created(t), dir, Ordering::Less)
}
/// check if `dir`: [DirEntry] is created at `t`: [SystemTime]
pub fn created_at(dir: &DirEntry, t: SystemTime) -> bool {
    filter(FilterType::Created(t), dir, Ordering::Equal)
}
/// check if `dir`: [DirEntry] is created after `t`: [SystemTime]
pub fn created_after(dir: &DirEntry, t: SystemTime) -> bool {
    filter(FilterType::Created(t), dir, Ordering::Greater)
}

/// check if `dir`: [DirEntry] is modified before `t`: [SystemTime]
pub fn modified_before(dir: &DirEntry, t: SystemTime) -> bool {
    filter(FilterType::Modified(t), dir, Ordering::Less)
}
/// check if `dir`: [DirEntry] is modified at `t`: [SystemTime]
pub fn modified_at(dir: &DirEntry, t: SystemTime) -> bool {
    filter(FilterType::Modified(t), dir, Ordering::Equal)
}
/// check if `dir`: [DirEntry] is modified after `t`: [SystemTime]
pub fn modified_after(dir: &DirEntry, t: SystemTime) -> bool {
    filter(FilterType::Modified(t), dir, Ordering::Greater)
}

/// check if `dir`: [DirEntry] is smaller than `size_in_bytes`: [u64]
pub fn file_size_smaller(dir: &DirEntry, size_in_bytes: u64) -> bool {
    filter(FilterType::FileSize(size_in_bytes), dir, Ordering::Less)
}
/// check if `dir`: [DirEntry] is equal to `size_in_bytes`: [u64]
pub fn file_size_equals(dir: &DirEntry, size_in_bytes: u64) -> bool {
    filter(FilterType::FileSize(size_in_bytes), dir, Ordering::Equal)
}
/// check if `dir`: [DirEntry] is greater than `size_in_bytes`: [u64]
pub fn file_size_greater(dir: &DirEntry, size_in_bytes: u64) -> bool {
    filter(FilterType::FileSize(size_in_bytes), dir, Ordering::Greater)
}

/// convert kilobytes to bytes
pub fn kb(kb: f64) -> u64 {
    (kb * 1024_f64) as u64
}
/// convert megabytes to bytes
pub fn mb(mb: f64) -> u64 {
    (mb * 1024_u64.pow(2) as f64) as u64
}
/// convert gigabytes to bytes
pub fn gb(gb: f64) -> u64 {
    (gb * 1024_u64.pow(3) as f64) as u64
}

enum FilterType {
    Created(SystemTime),
    Modified(SystemTime),
    FileSize(u64),
}

fn filter(typ: FilterType, dir: &DirEntry, ord: Ordering) -> bool {
    if let Ok(m) = dir.metadata() {
        match typ {
            FilterType::Created(t) => {
                if let Ok(c) = m.created() {
                    return c.cmp(&t) == ord;
                }
            }
            FilterType::Modified(t) => {
                if let Ok(c) = m.modified() {
                    return c.cmp(&t) == ord;
                }
            }
            FilterType::FileSize(size) => {
                let len = m.len();
                return len.cmp(&size) == ord;
            }
        }
    }
    false
}
