use std::path::{Path, PathBuf};

pub fn create_profile_shortcut(
    mut path: PathBuf,
    name: &str,
    bin: &Path,
    profile_path: &str,
    icon_path: Option<&Path>,
) -> std::io::Result<PathBuf> {
    let encoded_profile_path = urlencoding::encode(profile_path);
    let launch_arg = format!("modrinth://profile/{encoded_profile_path}");

    create_shortcut(&mut path, name, bin, &[launch_arg.as_str()], icon_path)?;
    Ok(path)
}

#[cfg(target_os = "linux")]
fn create_shortcut(
    path: &mut PathBuf,
    name: &str,
    bin: &Path,
    args: &[&str],
    icon_path: Option<&Path>,
) -> std::io::Result<()> {
    if !has_extension(path, "desktop") {
        path.set_extension("desktop");
    }

    let exec = std::iter::once(shell_quote_unix(&bin.to_string_lossy()))
        .chain(args.iter().map(|arg| shell_quote_unix(arg)))
        .collect::<Vec<_>>()
        .join(" ");

    let icon = icon_path
        .map(|path| format!("Icon={}\n", path.display()))
        .unwrap_or_default();
    std::fs::write(
        path,
        format!(
            "[Desktop Entry]\nType=Application\nVersion=1.0\nName={name}\nExec={exec}\n{icon}Categories=Game;Minecraft;Launcher;\n"
        ),
    )?;

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn create_shortcut(
    path: &mut PathBuf,
    name: &str,
    bin: &Path,
    args: &[&str],
    _icon_path: Option<&Path>,
) -> std::io::Result<()> {
    if !has_extension(path, "app") {
        path.set_extension("app");
    }

    let contents = path.join("Contents");
    let macos = contents.join("MacOS");
    std::fs::create_dir_all(&macos)?;

    std::fs::write(
        contents.join("Info.plist"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>run.sh</string>
    <key>CFBundleIdentifier</key>
    <string>com.revoria.launcher.Shortcut</string>
    <key>CFBundleName</key>
    <string>{name}</string>
    <key>CFBundleDisplayName</key>
    <string>{name}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleVersion</key>
    <string>1</string>
</dict>
</plist>"#
        ),
    )?;

    let exec = std::iter::once(shell_quote_unix(&bin.to_string_lossy()))
        .chain(args.iter().map(|arg| shell_quote_unix(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    let script_path = macos.join("run.sh");
    std::fs::write(&script_path, format!("#!/bin/sh\nexec {exec}\n"))?;

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn create_shortcut(
    path: &mut PathBuf,
    name: &str,
    bin: &Path,
    args: &[&str],
    icon_path: Option<&Path>,
) -> std::io::Result<()> {
    if !has_extension(path, "lnk") {
        path.set_extension("lnk");
    }

    let args = args
        .iter()
        .map(|arg| arg.replace('"', "\\\""))
        .collect::<Vec<_>>()
        .join(" ");
    let icon = icon_path.unwrap_or(bin);
    let icon_location = format!("{},0", icon.display());
    let working_dir = bin.parent().unwrap_or_else(|| Path::new("."));
    let script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell\n\
         $Shortcut = $WshShell.CreateShortcut('{}')\n\
         $Shortcut.TargetPath = '{}'\n\
         $Shortcut.Arguments = '{}'\n\
         $Shortcut.WorkingDirectory = '{}'\n\
         $Shortcut.Description = '{}'\n\
         $Shortcut.IconLocation = '{}'\n\
         $Shortcut.Save()\n",
        powershell_quote(&path.to_string_lossy()),
        powershell_quote(&bin.to_string_lossy()),
        powershell_quote(&args),
        powershell_quote(&working_dir.to_string_lossy()),
        powershell_quote(name),
        powershell_quote(&icon_location),
    );

    let status = std::process::Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other("failed to create Windows shortcut"));
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn shell_quote_unix(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(target_os = "windows")]
fn powershell_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}
