// TODO - fixup namespaces

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeypadEvent {
    KeyPress(char),
    // LongPress/etc?
    #[doc(hidden)]
    _Extensible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeypadMode {
    /// Buffer digits until '#' to produce a PhoneNumber
    WaitForUserDial,
    /// Each key is treated as a DTMF event, also buffered
    /// for history/logging
    Dtmf,
}

pub struct KeypadBuffer {
    mode: KeypadMode,
    data: String,
}

impl KeypadBuffer {
    pub fn new() -> Self {
        KeypadBuffer {
            mode: KeypadMode::WaitForUserDial,
            data: String::new(),
        }
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// True if event meets criteria for the given mode
    pub fn push(&mut self, mode: KeypadMode, event: KeypadEvent) -> bool {
        if mode != self.mode {
            self.clear();
        }

        self.mode = mode;

        match self.mode {
            KeypadMode::WaitForUserDial => {
                if let KeypadEvent::KeyPress(c) = event {
                    if c == '#' {
                        true
                    } else {
                        self.data.push(c);
                        false
                    }
                } else {
                    false
                }
            }
            KeypadMode::Dtmf => {
                if let KeypadEvent::KeyPress(c) = event {
                    self.data.push(c);
                    true
                } else {
                    false
                }
            }
        }
    }
}
