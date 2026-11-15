use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Copies this executable to a per-user location and registers it with the OS.
pub fn run() -> Result<()> {
    let exe = std::env::current_exe().context("cannot locate the mdview executable")?;
    #[cfg(target_os = "linux")]
    linux(&exe)?;
    #[cfg(target_os = "windows")]
    windows(&exe)?;
    #[cfg(target_os = "macos")]
    {
        let _ = exe;
        bail!("on macOS use cargo-bundle; see packaging/macos/README.md");
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = exe;
        bail!("unsupported operating system");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn windows(exe: &Path) -> Result<()> {
    use std::os::windows::process::CommandExt;
    let dir = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .context("LOCALAPPDATA is not set")?
        .join("Programs")
        .join("MD Viewer");
    std::fs::create_dir_all(&dir).context("cannot create install directory")?;

    let dest = dir.join("mdview.exe");
    if exe != dest {
        std::fs::copy(exe, &dest).context("cannot copy mdview.exe")?;
    }
    let template = include_str!("../packaging/windows/register.reg");
    let escaped = dest.display().to_string().replace('\\', "\\\\");
    let reg_file = std::env::temp_dir().join("mdview-install.reg");
    std::fs::write(&reg_file, template.replace(r"C:\\path\\to\\mdview.exe", &escaped))
        .context("cannot write registry file")?;
    let status = Command::new("reg")
        .arg("import")
        .arg(&reg_file)
        .creation_flags(0x0800_0000)
        .status()
        .context("cannot run reg.exe")?;
    let _ = std::fs::remove_file(&reg_file);
    if !status.success() {
        bail!("registry import failed");
    }

    // Best-effort Explorer refresh so the new "Open with" entry appears now.
    let script = r#"Add-Type -Namespace W -Name S -MemberDefinition '[DllImport("shell32.dll")] public static extern void SHChangeNotify(uint e, uint f, IntPtr a, IntPtr b);'; [W.S]::SHChangeNotify(0x8000000,0,[IntPtr]::Zero,[IntPtr]::Zero)"#;
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(0x0800_0000)
        .status();

    println!("Installed to {}", dest.display());
    Ok(())
}

#[cfg(target_os = "linux")]
fn linux(exe: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .context("HOME is not set")?;
    let bin_dir = home.join(".local/bin");
    std::fs::create_dir_all(&bin_dir).context("cannot create ~/.local/bin")?;
    let dest = bin_dir.join("mdview");
    if exe != dest {
        std::fs::copy(exe, &dest).context("cannot copy mdview")?;
    }
    std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))
        .context("cannot make mdview executable")?;

    let apps_dir = home.join(".local/share/applications");
    std::fs::create_dir_all(&apps_dir).context("cannot create applications directory")?;
    std::fs::write(
        apps_dir.join("mdview.desktop"),
        include_str!("../packaging/linux/mdview.desktop"),
    )
    .context("cannot write mdview.desktop")?;

    let _ = Command::new("update-desktop-database").arg(&apps_dir).status();
    match Command::new("xdg-mime")
        .args(["default", "mdview.desktop", "text/markdown"])
        .status()
    {
        Ok(status) if status.success() => {}
        _ => eprintln!("warning: could not set MD Viewer as the default Markdown handler"),
    }
    println!("Installed to {}", dest.display());
    Ok(())
}
