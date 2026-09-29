use anyhow::{Context, Result, anyhow};
use std::process::Command;

/// Execute `ddcutil getvcp <code>` and return the raw stdout.
pub fn get_vcp(display: u32, code: &str) -> Result<String> {
    let output = Command::new("ddcutil")
        .args(["getvcp", code, "--display", &display.to_string()])
        .output()
        .context("Failed to run ddcutil — is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ddcutil getvcp {} failed: {}", code, stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Execute `ddcutil setvcp <code> <value>` with --noverify.
pub fn set_vcp(display: u32, code: &str, value: u8) -> Result<()> {
    let output = Command::new("ddcutil")
        .args([
            "setvcp",
            code,
            &format!("{}", value),
            "--display",
            &display.to_string(),
            "--noverify",
        ])
        .output()
        .context("Failed to run ddcutil — is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "ddcutil setvcp {} {} failed: {}",
            code,
            value,
            stderr.trim()
        ));
    }

    Ok(())
}

/// Parse the current value from ddcutil getvcp output.
///
/// Handles two formats:
///   Standard:       "current value =   70, max value =  100"
///   Manufacturer:   "mh=0x00, ml=0x0a, sh=0x00, sl=0x0a"
pub fn parse_current_value(output: &str) -> Result<u8> {
    for line in output.lines() {
        // Standard format: "current value = N"
        if let Some(pos) = line.find("current value =") {
            let after = &line[pos + "current value =".len()..];
            let num_str: String = after.chars().take_while(|c| *c == ' ' || c.is_ascii_digit()).collect();
            let val: u8 = num_str.trim().parse().context("Failed to parse VCP value")?;
            return Ok(val);
        }

        // Manufacturer-specific format: "sl=0x0a"
        if let Some(pos) = line.find("sl=0x") {
            let hex_start = pos + "sl=0x".len();
            let hex_str: String = line[hex_start..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            let val = u8::from_str_radix(&hex_str, 16)
                .context("Failed to parse manufacturer-specific VCP hex value")?;
            return Ok(val);
        }
    }
    Err(anyhow!("Could not find current value in ddcutil output"))
}
