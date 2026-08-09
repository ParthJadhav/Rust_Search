use super::SearchBuilder;
use ignore::DirEntry;
use std::{
    cmp::Ordering,
    fs::Metadata,
    panic::{RefUnwindSafe, UnwindSafe},
    sync::Arc,
    time::SystemTime,
};

/// Custom filter fn to expose the dir entry directly.
///
/// Prefer passing closures directly to [`FilterExt::custom_filter`]. This alias
/// remains available for callers that want to name a non-capturing filter
/// function explicitly.
pub type FilterFn = fn(&DirEntry) -> bool;

type CustomFilter =
    Arc<dyn Fn(&DirEntry) -> bool + Send + Sync + UnwindSafe + RefUnwindSafe + 'static>;

#[derive(Clone)]
pub enum FilterType {
    Created(Ordering, SystemTime),
    Modified(Ordering, SystemTime),
    FileSize(Ordering, u64),
    Custom(CustomFilter),
}

impl FilterType {
    pub(crate) fn custom<F>(f: F) -> Self
    where
        F: Fn(&DirEntry) -> bool + Send + Sync + UnwindSafe + RefUnwindSafe + 'static,
    {
        Self::Custom(Arc::new(f))
    }

    const fn requires_metadata(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }

    fn apply(&self, dir: &DirEntry, metadata: Option<&Metadata>) -> bool {
        match self {
            Self::Created(cmp, time) => metadata
                .and_then(|metadata| metadata.created().ok())
                .is_some_and(|created| created.cmp(time) == *cmp),
            Self::Modified(cmp, time) => metadata
                .and_then(|metadata| metadata.modified().ok())
                .is_some_and(|modified| modified.cmp(time) == *cmp),
            Self::FileSize(cmp, size_in_bytes) => {
                metadata.is_some_and(|metadata| metadata.len().cmp(size_in_bytes) == *cmp)
            }
            Self::Custom(filter) => filter(dir),
        }
    }
}

/// Apply all result filters while reusing a single metadata lookup.
pub fn matches_all(dir: &DirEntry, filters: &[FilterType]) -> bool {
    let metadata = filters
        .iter()
        .any(FilterType::requires_metadata)
        .then(|| dir.metadata().ok())
        .flatten();

    filters
        .iter()
        .all(|filter| filter.apply(dir, metadata.as_ref()))
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

impl From<FileSize> for u64 {
    fn from(size: FileSize) -> Self {
        use self::FileSize::{Byte, Gigabyte, Kilobyte, Megabyte, Terabyte};

        match size {
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
    /// Files smaller than the supplied [`FileSize`].
    fn file_size_smaller(self, size: FileSize) -> Self;
    /// Files equal to the supplied [`FileSize`].
    fn file_size_equal(self, size: FileSize) -> Self;
    /// Files greater than the supplied [`FileSize`].
    fn file_size_greater(self, size: FileSize) -> Self;
    /// Custom filter that exposes the [`DirEntry`] directly.
    /// ```rust
    /// use rust_search::{SearchBuilder, FilterExt};
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .custom_filter(|dir| dir.metadata().is_ok_and(|metadata| metadata.is_file()))
    ///     .build()
    ///     .collect();
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
        self.filter(Custom(Arc::new(f)))
    }
}
