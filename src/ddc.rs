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

/// Parse `current value = <N>` from ddcutil getvcp output.
pub fn parse_current_value(output: &str) -> Result<u8> {
    // ddcutil output looks like:
    //   VCP code 0x10 (Brightness                    ): current value =   70, max value =  100
    // We look for "current value ="
    for line in output.lines() {
        if let Some(pos) = line.find("current value =") {
            let after = &line[pos + "current value =".len()..];
            // Take characters until comma or end
            let num_str: String = after.chars().take_while(|c| *c == ' ' || c.is_ascii_digit()).collect();
            let val: u8 = num_str.trim().parse().context("Failed to parse VCP value")?;
            return Ok(val);
        }
    }
    Err(anyhow!("Could not find 'current value' in ddcutil output"))
}
