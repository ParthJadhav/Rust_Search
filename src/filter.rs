use super::SearchBuilder;
use ignore::DirEntry;
use std::{cmp::Ordering, time::SystemTime};

/// custom filter fn to expose the dir entry directly
pub type FilterFn = fn(&DirEntry) -> bool;

#[derive(Clone, Copy)]
pub enum FilterType {
    Created(Ordering, SystemTime),
    Modified(Ordering, SystemTime),
    FileSize(Ordering, u64),
    Custom(FilterFn),
}

impl FilterType {
    pub fn apply(&self, dir: &DirEntry, filter_dirs: bool) -> bool {
        if let Ok(m) = dir.metadata() {
            if !filter_dirs && m.file_type().is_dir() {
                return true;
            }
            match self {
                Self::Created(cmp, time) => {
                    if let Ok(created) = m.created() {
                        return created.cmp(time) == *cmp;
                    }
                }
                Self::Modified(cmp, time) => {
                    if let Ok(modified) = m.modified() {
                        return modified.cmp(time) == *cmp;
                    }
                }
                Self::FileSize(cmp, size_in_bytes) => {
                    return m.len().cmp(size_in_bytes) == *cmp;
                }
                Self::Custom(f) => return f(dir),
            }
        }
        false
    }
}

/// enum to easily convert between `byte_sizes`
#[derive(Debug, Clone)]
pub enum FileSize {
    /// size in bytes
    Byte(u64),
    /// size in kilobytes
    Kilobyte(f64),
    /// size in megabytes
    Megabyte(f64),
    /// size in gigabytes
    Gigabyte(f64),
    /// size in terabytes
    Terabyte(f64),
}

// helper function for FileSize conversion
fn convert(b: f64, pow: u32) -> u64 {
    (b * 1024_u64.pow(pow) as f64) as u64
}

#[allow(clippy::from_over_into)]
impl Into<u64> for FileSize {
    fn into(self) -> u64 {
        use self::FileSize::{Byte, Gigabyte, Kilobyte, Megabyte, Terabyte};

        match self {
            Byte(b) => b,
            Kilobyte(b) => convert(b, 1),
            Megabyte(b) => convert(b, 2),
            Gigabyte(b) => convert(b, 3),
            Terabyte(b) => convert(b, 4),
        }
    }
}

/// import this trait to filter files
pub trait FilterExt {
    /// files created before `t`: [`SystemTime`]
    fn created_before(self, t: SystemTime) -> Self;
    /// files created at `t`: [`SystemTime`]
    fn created_at(self, t: SystemTime) -> Self;
    /// files created after `t`: [`SystemTime`]
    fn created_after(self, t: SystemTime) -> Self;
    /// files created before `t`: [`SystemTime`]
    fn modified_before(self, t: SystemTime) -> Self;
    /// files modified at `t`: [`SystemTime`]
    fn modified_at(self, t: SystemTime) -> Self;
    /// files modified after `t`: [`SystemTime`]
    fn modified_after(self, t: SystemTime) -> Self;
    /// files smaller than `size_in_bytes`: [usize]
    fn file_size_smaller(self, size: FileSize) -> Self;
    /// files equal to `size_in_bytes`: [usize]
    fn file_size_equal(self, size: FileSize) -> Self;
    /// files greater than `size_in_bytes`: [usize]
    fn file_size_greater(self, size: FileSize) -> Self;
    /// custom filter that exposes the [`DirEntry`] directly
    /// ```rust
    /// # use rust_search::FilterExt;
    /// # let builder = rust_search::SearchBuilder::default();
    /// builder.custom_filter(|dir| dir.metadata().unwrap().is_file());
    /// ```
    fn custom_filter(self, f: FilterFn) -> Self;
}

use FilterType::{Created, Custom, FileSize as FilterFileSize, Modified};
use Ordering::{Equal, Greater, Less};
impl FilterExt for SearchBuilder {
    fn created_before(self, t: SystemTime) -> Self {
        self.filter(Created(Less, t))
    }

    fn created_at(self, t: SystemTime) -> Self {
        self.filter(Created(Equal, t))
    }

    fn created_after(self, t: SystemTime) -> Self {
        self.filter(Created(Greater, t))
    }

    fn modified_before(self, t: SystemTime) -> Self {
        self.filter(Modified(Less, t))
    }

    fn modified_at(self, t: SystemTime) -> Self {
        self.filter(Modified(Equal, t))
    }

    fn modified_after(self, t: SystemTime) -> Self {
        self.filter(Modified(Greater, t))
    }

    fn file_size_smaller(self, size: FileSize) -> Self {
        self.filter(FilterFileSize(Less, size.into()))
    }

    fn file_size_equal(self, size: FileSize) -> Self {
        self.filter(FilterFileSize(Equal, size.into()))
    }

    fn file_size_greater(self, size: FileSize) -> Self {
        self.filter(FilterFileSize(Greater, size.into()))
    }
    fn custom_filter(self, f: FilterFn) -> Self {
        self.filter(Custom(f))
    }
}
