use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a position in a log
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LogOffset(pub u64);

impl LogOffset {
    pub const ZERO: LogOffset = LogOffset(0);
    pub const MAX: LogOffset = LogOffset(u64::MAX);

    #[inline]
    pub fn new(offset: u64) -> Self {
        LogOffset(offset)
    }

    #[inline]
    pub fn next(&self) -> Self {
        LogOffset(self.0.saturating_add(1))
    }

    #[inline]
    pub fn prev(&self) -> Option<Self> {
        if self.0 > 0 {
            Some(LogOffset(self.0 - 1))
        } else {
            None
        }
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for LogOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for LogOffset {
    fn from(offset: u64) -> Self {
        LogOffset(offset)
    }
}

/// Represents a range of offsets [start, end)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OffsetRange {
    pub start: LogOffset,
    pub end: LogOffset,
}

impl OffsetRange {
    pub fn new(start: LogOffset, end: LogOffset) -> Self {
        OffsetRange { start, end }
    }

    pub fn contains(&self, offset: LogOffset) -> bool {
        offset >= self.start && offset < self.end
    }

    pub fn len(&self) -> u64 {
        if self.end.0 >= self.start.0 {
            self.end.0 - self.start.0
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

