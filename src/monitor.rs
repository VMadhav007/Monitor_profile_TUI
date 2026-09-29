use anyhow::{Context, Result, anyhow};
use std::process::Command;

use crate::ddc;
use crate::preset::Preset;

/// Represents a connected DDC/CI monitor.
pub struct Monitor {
    pub display: u32,
    pub name: String,
    pub connection: String,
}

impl Monitor {
    /// Detect the first connected monitor using `ddcutil detect`.
    pub fn detect() -> Result<Monitor> {
        let output = Command::new("ddcutil")
            .arg("detect")
            .output()
            .context("Failed to run ddcutil — is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ddcutil detect failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_detect(&stdout)
    }

    /// Parse `ddcutil detect` output.
    fn parse_detect(output: &str) -> Result<Monitor> {
        let mut display: Option<u32> = None;
        let mut name = String::from("Unknown");
        let mut connection = String::from("Unknown");

        for line in output.lines() {
            let trimmed = line.trim();

            // "Display 1"
            if trimmed.starts_with("Display") && display.is_none() {
                if let Some(num_str) = trimmed.strip_prefix("Display") {
                    if let Ok(d) = num_str.trim().parse::<u32>() {
                        display = Some(d);
                    }
                }
            }

            // "Monitor:                Lenovo Group Limited"
            // or "Model:                 R24e-20"
            if let Some(rest) = trimmed.strip_prefix("Model:") {
                let model = rest.trim().to_string();
                if !model.is_empty() {
                    name = model;
                }
            }

            // Try manufacturer + model combo
            if let Some(rest) = trimmed.strip_prefix("Mfg id:") {
                let mfg = rest.trim().to_string();
                if name == "Unknown" && !mfg.is_empty() {
                    name = mfg;
                }
            }

            // "I2C bus:  /dev/i2c-5" or "DRM connector: card0-HDMI-A-1"
            if let Some(rest) = trimmed.strip_prefix("DRM connector:") {
                let conn = rest.trim();
                // Extract just the connector name like "HDMI-A-1"
                if let Some(dash_pos) = conn.find('-') {
                    connection = conn[dash_pos + 1..].to_string();
                } else {
                    connection = conn.to_string();
                }
            }
        }

        match display {
            Some(d) => Ok(Monitor {
                display: d,
                name,
                connection,
            }),
            None => Err(anyhow!("No DDC/CI monitor found")),
        }
    }
}

/// Read brightness (VCP code 0x10), returns 0–100.
pub fn get_brightness(display: u32) -> Result<u8> {
    let output = ddc::get_vcp(display, "10")?;
    ddc::parse_current_value(&output)
}

/// Set brightness (VCP code 0x10), value 0–100.
pub fn set_brightness(display: u32, value: u8) -> Result<()> {
    let clamped = value.min(100);
    ddc::set_vcp(display, "10", clamped)
}

/// Read current preset (VCP code 0xF9).
pub fn get_preset(display: u32) -> Result<Preset> {
    let output = ddc::get_vcp(display, "F9")?;
    let raw = ddc::parse_current_value(&output)?;
    Preset::from_vcp(raw).ok_or_else(|| anyhow!("Unknown preset value: 0x{:02X}", raw))
}

/// Set preset (VCP code 0xF9).
pub fn set_preset(display: u32, preset: Preset) -> Result<()> {
    ddc::set_vcp(display, "F9", preset.vcp_value())
}
