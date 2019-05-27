// TODO - fixup namespaces

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeypadEvent {
    Key(char),
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

    // TODO - return bool? method for ref to data
    // user clears()?
    pub fn push(&mut self, mode: KeypadMode, event: KeypadEvent) -> Option<String> {
        if mode != self.mode {
            self.data.clear();
        }

        self.mode = mode;

        match self.mode {
            KeypadMode::WaitForUserDial => {
                if let KeypadEvent::Key(c) = event {
                    if c == '#' {
                        Some(self.data.clone())
                    } else {
                        self.data.push(c);
                        None
                    }
                } else {
                    None
                }
            }
            KeypadMode::Dtmf => {
                if let KeypadEvent::Key(c) = event {
                    if c == '#' {
                        Some(self.data.clone())
                    } else {
                        self.data.push(c);
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
}
