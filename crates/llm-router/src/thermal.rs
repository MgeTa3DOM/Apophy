//! Thermal guard — surveillance température GPU.
//!
//! CLAUDE.md : GPU < 80°C sinon pause automatique et failover CPU.

use tracing::warn;

/// Lit la température GPU actuelle (°C).
///
/// Tente de lire depuis sysfs (Linux NVIDIA/AMD).
/// Retourne 0 si pas de GPU détecté (safe : pas de thermal limit).
pub fn read_gpu_temp() -> u32 {
    // NVIDIA: /sys/class/hwmon/hwmon*/temp*_input (millidegrees)
    if let Ok(temp) = try_read_nvidia_temp() {
        return temp;
    }

    // AMD: /sys/class/drm/card0/device/hwmon/hwmon*/temp1_input
    if let Ok(temp) = try_read_amd_temp() {
        return temp;
    }

    // Pas de GPU détecté — retourne 0 (aucune restriction thermique)
    0
}

/// Vérifie si le GPU est en zone safe.
pub fn is_gpu_safe(limit: u32) -> bool {
    let temp = read_gpu_temp();
    if temp >= limit {
        warn!(temp, limit, "GPU thermal limit exceeded — failover to CPU");
        return false;
    }
    true
}

fn try_read_nvidia_temp() -> Result<u32, ()> {
    // nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader
    let paths = glob_hwmon_temps();
    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(millideg) = content.trim().parse::<u32>() {
                return Ok(millideg / 1000);
            }
        }
    }
    Err(())
}

fn try_read_amd_temp() -> Result<u32, ()> {
    let path = "/sys/class/drm/card0/device/hwmon";
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let temp_path = entry.path().join("temp1_input");
            if let Ok(content) = std::fs::read_to_string(&temp_path) {
                if let Ok(millideg) = content.trim().parse::<u32>() {
                    return Ok(millideg / 1000);
                }
            }
        }
    }
    Err(())
}

fn glob_hwmon_temps() -> Vec<String> {
    let mut paths = Vec::new();
    let hwmon_base = "/sys/class/hwmon";
    if let Ok(entries) = std::fs::read_dir(hwmon_base) {
        for entry in entries.flatten() {
            let temp_path = entry.path().join("temp1_input");
            if temp_path.exists() {
                if let Some(s) = temp_path.to_str() {
                    paths.push(s.to_string());
                }
            }
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_gpu_temp_no_crash() {
        // Ne crash jamais, même sans GPU
        let temp = read_gpu_temp();
        assert!(temp < 200); // sanity check
    }

    #[test]
    fn is_gpu_safe_without_gpu() {
        // Sans GPU, temp = 0, donc toujours safe
        assert!(is_gpu_safe(80));
    }

    #[test]
    fn is_gpu_safe_with_limit_zero() {
        // Limite 0 = jamais safe (sauf si temp = 0 et pas de GPU)
        // Avec temp = 0, 0 >= 0 = true → not safe
        let safe = is_gpu_safe(0);
        assert!(!safe);
    }
}
