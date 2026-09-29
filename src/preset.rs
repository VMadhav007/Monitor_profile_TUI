use std::fmt;

/// Monitor preset modes exposed via DDC/CI VCP code 0xF9.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Standard,
    FPS1,
    FPS2,
    Racing,
    RTS,
    Game1,
    Game2,
}

impl Preset {
    /// All presets in display order.
    pub const ALL: [Preset; 7] = [
        Preset::Standard,
        Preset::FPS1,
        Preset::FPS2,
        Preset::Racing,
        Preset::RTS,
        Preset::Game1,
        Preset::Game2,
    ];

    /// VCP value sent to the monitor via ddcutil.
    pub fn vcp_value(self) -> u8 {
        match self {
            Preset::Standard => 0x0A,
            Preset::FPS1 => 0x01,
            Preset::FPS2 => 0x02,
            Preset::Racing => 0x03,
            Preset::RTS => 0x04,
            Preset::Game1 => 0x05,
            Preset::Game2 => 0x06,
        }
    }

    /// Convert a VCP value back to a Preset.
    pub fn from_vcp(value: u8) -> Option<Preset> {
        match value {
            0x0A => Some(Preset::Standard),
            0x01 => Some(Preset::FPS1),
            0x02 => Some(Preset::FPS2),
            0x03 => Some(Preset::Racing),
            0x04 => Some(Preset::RTS),
            0x05 => Some(Preset::Game1),
            0x06 => Some(Preset::Game2),
            _ => None,
        }
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Preset::Standard => write!(f, "Standard"),
            Preset::FPS1 => write!(f, "FPS1"),
            Preset::FPS2 => write!(f, "FPS2"),
            Preset::Racing => write!(f, "Racing"),
            Preset::RTS => write!(f, "RTS"),
            Preset::Game1 => write!(f, "Game 1"),
            Preset::Game2 => write!(f, "Game 2"),
        }
    }
}
