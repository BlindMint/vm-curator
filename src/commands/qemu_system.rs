//! QEMU system emulator utilities
//!
//! Provides utilities for checking QEMU availability and capabilities.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Get QEMU version information
pub fn get_qemu_version(emulator: &str) -> Result<String> {
    let output = Command::new(emulator)
        .arg("--version")
        .output()
        .context("Failed to get QEMU version")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().next().unwrap_or("Unknown").to_string())
}

/// Check if QEMU emulator is available
pub fn is_emulator_available(emulator: &str) -> bool {
    Command::new("which")
        .arg(emulator)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// List available QEMU emulators on the system
pub fn list_available_emulators() -> Vec<String> {
    let emulators = [
        "qemu-system-x86_64",
        "qemu-system-i386",
        "qemu-system-ppc",
        "qemu-system-m68k",
        "qemu-system-arm",
        "qemu-system-aarch64",
    ];

    emulators
        .iter()
        .filter(|e| is_emulator_available(e))
        .map(|e| e.to_string())
        .collect()
}

/// Check KVM availability
pub fn is_kvm_available() -> bool {
    Path::new("/dev/kvm").exists()
}

/// Get supported display backends for a QEMU emulator
///
/// Runs `<emulator> -display help` and parses the output to get
/// the list of supported display backends (e.g., gtk, sdl, spice-app, vnc).
pub fn get_supported_displays(emulator: &str) -> Vec<String> {
    let output = match Command::new(emulator).args(["-display", "help"]).output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    // QEMU prints display backends to stdout (or sometimes stderr)
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };

    let mut displays = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        // Skip empty lines and header lines
        if trimmed.is_empty() || trimmed.starts_with("Available") || trimmed.contains(':') {
            continue;
        }
        // Each display backend is typically listed on its own line
        let backend = trimmed.split_whitespace().next().unwrap_or("");
        if !backend.is_empty() {
            displays.push(backend.to_string());
        }
    }

    displays
}

/// Check if a SPICE viewer application is available in PATH
///
/// Checks for `remote-viewer` (from virt-viewer package) or `virt-viewer`.
pub fn is_spice_viewer_available() -> bool {
    for viewer in &["remote-viewer", "virt-viewer"] {
        if Command::new("which")
            .arg(viewer)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

/// Get KVM module info
pub fn get_kvm_info() -> Option<String> {
    if !is_kvm_available() {
        return None;
    }

    let output = Command::new("lsmod").output().ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("kvm_intel") || line.starts_with("kvm_amd") {
            return Some(line.split_whitespace().next()?.to_string());
        }
    }

    Some("kvm".to_string())
}

/// Information about available network backends
#[derive(Debug, Clone)]
pub struct NetworkCapabilities {
    pub passt_available: bool,
    pub bridge_helper_path: Option<PathBuf>,
    pub bridge_helper_configured: bool,
    pub system_bridges: Vec<String>,
    pub allowed_bridges: Vec<String>,
}

/// Classify a bridge by likely usage so the UI can present safer guidance.
pub fn classify_bridge(name: &str) -> &'static str {
    if name.starts_with("virbr") {
        "private/libvirt bridge"
    } else if name.starts_with("br-") {
        "container/custom bridge"
    } else {
        "host/LAN bridge"
    }
}

/// Whether a bridge is a good default candidate for isolated lab use.
pub fn is_lab_friendly_bridge(name: &str) -> bool {
    name.starts_with("virbr")
}

/// Filter allowed bridges to those that are most likely private/internal lab bridges.
pub fn lab_friendly_bridges(bridges: &[String]) -> Vec<String> {
    bridges
        .iter()
        .filter(|bridge| is_lab_friendly_bridge(bridge))
        .cloned()
        .collect()
}

/// Detect all available networking capabilities
pub fn detect_network_capabilities() -> NetworkCapabilities {
    let passt_available = is_passt_available();
    let bridge_helper_path = find_bridge_helper();
    let bridge_helper_configured = bridge_helper_path
        .as_ref()
        .map(|p| is_bridge_helper_configured(p))
        .unwrap_or(false);
    let system_bridges = list_system_bridges();
    let allowed_bridges = if bridge_helper_configured {
        list_allowed_bridges(&system_bridges)
    } else {
        Vec::new()
    };

    NetworkCapabilities {
        passt_available,
        bridge_helper_path,
        bridge_helper_configured,
        system_bridges,
        allowed_bridges,
    }
}

/// Check if passt binary is available
fn is_passt_available() -> bool {
    Command::new("passt")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Find qemu-bridge-helper binary
fn find_bridge_helper() -> Option<PathBuf> {
    let paths = [
        "/usr/lib/qemu/qemu-bridge-helper",
        "/usr/libexec/qemu-bridge-helper",
        "/usr/libexec/qemu/qemu-bridge-helper",
    ];

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Check if bridge helper has setuid or CAP_NET_ADMIN
fn is_bridge_helper_configured(path: &Path) -> bool {
    // Check setuid bit
    if let Ok(metadata) = std::fs::metadata(path) {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if mode & 0o4000 != 0 {
            return true;
        }
    }

    // Check capabilities via getcap
    if let Ok(output) = Command::new("getcap").arg(path).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("cap_net_admin") {
            return true;
        }
    }

    false
}

/// List bridges currently on the system
fn list_system_bridges() -> Vec<String> {
    let output = match Command::new("ip")
        .args(["-o", "link", "show", "type", "bridge"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut bridges = Vec::new();
    for line in stdout.lines() {
        // Format: "N: bridgename: <FLAGS> ..."
        if let Some(name) = line.split(':').nth(1) {
            let name = name.trim();
            if !name.is_empty() {
                bridges.push(name.to_string());
            }
        }
    }
    bridges
}

fn list_allowed_bridges(system_bridges: &[String]) -> Vec<String> {
    let bridge_conf = match std::fs::read_to_string("/etc/qemu/bridge.conf") {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };

    parse_allowed_bridges(&bridge_conf, system_bridges)
}

fn parse_allowed_bridges(bridge_conf: &str, system_bridges: &[String]) -> Vec<String> {
    let mut allowed = Vec::new();
    for line in bridge_conf.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("allow ") {
            let bridge = rest.trim();
            if bridge == "all" {
                return system_bridges.to_vec();
            }

            if system_bridges.iter().any(|candidate| candidate == bridge)
                && !allowed.iter().any(|candidate| candidate == bridge)
            {
                allowed.push(bridge.to_string());
            }
        }
    }

    allowed
}

#[cfg(test)]
mod tests {
    use super::{classify_bridge, lab_friendly_bridges, parse_allowed_bridges};

    #[test]
    fn test_parse_allowed_bridges_filters_to_existing_bridges() {
        let system_bridges = vec!["virbr0".to_string(), "virbr1".to_string()];
        let bridge_conf = "# comment\nallow virbr0\nallow virbr2\n";

        let allowed = parse_allowed_bridges(bridge_conf, &system_bridges);
        assert_eq!(allowed, vec!["virbr0".to_string()]);
    }

    #[test]
    fn test_parse_allowed_bridges_supports_allow_all() {
        let system_bridges = vec!["virbr0".to_string(), "virbr1".to_string()];
        let allowed = parse_allowed_bridges("allow all\n", &system_bridges);
        assert_eq!(allowed, system_bridges);
    }

    #[test]
    fn test_classify_bridge() {
        assert_eq!(classify_bridge("virbr0"), "private/libvirt bridge");
        assert_eq!(classify_bridge("br-f37fef"), "container/custom bridge");
        assert_eq!(classify_bridge("br0"), "host/LAN bridge");
    }

    #[test]
    fn test_lab_friendly_bridges_prefers_virbr() {
        let bridges = vec![
            "virbr0".to_string(),
            "br0".to_string(),
            "virbr1".to_string(),
            "br-deadbeef".to_string(),
        ];
        assert_eq!(
            lab_friendly_bridges(&bridges),
            vec!["virbr0".to_string(), "virbr1".to_string()]
        );
    }
}
