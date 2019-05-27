#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    NoReason = 0,
    NoResponse = 1,
    Forbidden = 2,
    Declined = 3,
    NotFound = 4,
    NotAnswered = 5,
    Busy = 6,
    UnsupportedContent = 7,
    IOError = 8,
    DoNotDisturb = 9,
    // TODO - more
}

impl From<Reason> for u32 {
    fn from(r: Reason) -> u32 {
        r as u32
    }
}
