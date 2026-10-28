use crate::self_update::UpdateError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub asset_name: &'static str,
    pub executable_suffix: &'static str,
}

pub fn current() -> Result<Target, UpdateError> {
    for (os, arch) in [
        ("linux", "x86_64"),
        ("linux", "aarch64"),
        ("windows", "x86_64"),
    ] {
        if (cfg!(target_os = "linux")
            && os == "linux"
            && ((cfg!(target_arch = "x86_64") && arch == "x86_64")
                || (cfg!(target_arch = "aarch64") && arch == "aarch64")))
            || (cfg!(target_os = "windows")
                && os == "windows"
                && cfg!(target_arch = "x86_64")
                && arch == "x86_64")
        {
            return for_target(os, arch);
        }
    }
    Err(UpdateError::UnsupportedTarget(format!(
        "{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    )))
}

pub fn for_target(os: &str, arch: &str) -> Result<Target, UpdateError> {
    match (os, arch) {
        ("linux", "x86_64") => Ok(Target {
            asset_name: "rudis-lite-linux-x86_64",
            executable_suffix: "",
        }),
        ("linux", "aarch64") => Ok(Target {
            asset_name: "rudis-lite-linux-aarch64",
            executable_suffix: "",
        }),
        ("windows", "x86_64") => Ok(Target {
            asset_name: "rudis-lite-windows-x86_64.exe",
            executable_suffix: ".exe",
        }),
        _ => Err(UpdateError::UnsupportedTarget(format!("{os}-{arch}"))),
    }
}
