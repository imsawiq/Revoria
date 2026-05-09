#[cfg(target_os = "linux")]
use std::ffi::OsString;
#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows"
))]
use std::path::Path;
use std::path::PathBuf;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::Stdio;
use tokio::process::Command;

#[cfg(any(target_os = "linux", target_os = "macos"))]
const STDIN_PAYLOAD_CONFIGURED_ENV: &str =
    "MODRINTH_SANDBOX_STDIN_PAYLOAD_CONFIGURED";

pub struct SandboxConfig {
    pub allow_read: Vec<PathBuf>,
    pub allow_write: Vec<PathBuf>,
    pub sandbox_dir: PathBuf,
    pub stdin_payload: Option<String>,
}

pub fn apply(
    command: &mut Command,
    config: SandboxConfig,
) -> crate::Result<()> {
    apply_platform(command, config)
}

#[cfg(target_os = "linux")]
fn apply_platform(
    command: &mut Command,
    config: SandboxConfig,
) -> crate::Result<()> {
    use std::ffi::OsStr;

    if !is_command_available("bwrap") {
        return Err(crate::ErrorKind::LauncherError(
            "Sandbox requires the 'bwrap' command to be installed.".to_string(),
        )
        .into());
    }

    if !is_command_available("xdg-dbus-proxy") {
        return Err(crate::ErrorKind::LauncherError(
            "Sandbox requires the 'xdg-dbus-proxy' command to be installed."
                .to_string(),
        )
        .into());
    }

    let original = command.as_std();
    let program = original.get_program().to_os_string();
    let args = original
        .get_args()
        .map(OsStr::to_os_string)
        .collect::<Vec<_>>();
    let current_dir = original.get_current_dir().map(Path::to_path_buf);
    let envs = original
        .get_envs()
        .map(|(k, v)| (k.to_os_string(), v.map(OsStr::to_os_string)))
        .collect::<Vec<_>>();

    let mut bwrap_args = Vec::<OsString>::new();
    push(&mut bwrap_args, "--die-with-parent");
    push(&mut bwrap_args, "--unshare-all");
    push(&mut bwrap_args, "--share-net");
    push_pair(&mut bwrap_args, "--proc", "/proc");
    push_pair(&mut bwrap_args, "--dev", "/dev");
    push_pair(&mut bwrap_args, "--tmpfs", "/tmp");

    for path in SYSTEM_READ_ONLY {
        bind_if_exists(&mut bwrap_args, "--ro-bind-try", Path::new(path));
    }
    for path in DEVICE_BINDS {
        bind_if_exists(&mut bwrap_args, "--dev-bind-try", Path::new(path));
    }
    bind_nvidia_devices(&mut bwrap_args);
    bind_runtime_desktop(&mut bwrap_args);

    for path in config.allow_read {
        bind_if_exists(&mut bwrap_args, "--ro-bind-try", &path);
    }
    for path in config.allow_write {
        bind_if_exists(&mut bwrap_args, "--bind-try", &path);
    }

    let cache = config.sandbox_dir.join("cache");
    let data = config.sandbox_dir.join("data");
    let cfg = config.sandbox_dir.join("config");
    let home = config.sandbox_dir.join("home");
    std::fs::create_dir_all(&cache).map_err(crate::util::io::IOError::from)?;
    std::fs::create_dir_all(&data).map_err(crate::util::io::IOError::from)?;
    std::fs::create_dir_all(&cfg).map_err(crate::util::io::IOError::from)?;
    std::fs::create_dir_all(&home).map_err(crate::util::io::IOError::from)?;
    bind_if_exists(&mut bwrap_args, "--bind-try", &cache);
    bind_if_exists(&mut bwrap_args, "--bind-try", &data);
    bind_if_exists(&mut bwrap_args, "--bind-try", &cfg);
    bind_if_exists(&mut bwrap_args, "--bind-try", &home);
    push_pair_os(&mut bwrap_args, "--setenv", "XDG_CACHE_HOME", cache);
    push_pair_os(&mut bwrap_args, "--setenv", "XDG_DATA_HOME", data);
    push_pair_os(&mut bwrap_args, "--setenv", "XDG_CONFIG_HOME", cfg);
    push_pair_os(&mut bwrap_args, "--setenv", "HOME", home);

    if let Some(proxy) = start_dbus_proxy(&config.sandbox_dir)? {
        push_pair_os(
            &mut bwrap_args,
            "--bind",
            proxy.clone(),
            runtime_dir()
                .unwrap_or_else(|| PathBuf::from("/run/user/1000"))
                .join("bus"),
        );
        let bus = format!(
            "unix:path={}",
            runtime_dir()
                .unwrap_or_else(|| PathBuf::from("/run/user/1000"))
                .join("bus")
                .display()
        );
        push_pair(&mut bwrap_args, "--setenv", "DBUS_SESSION_BUS_ADDRESS");
        bwrap_args.push(bus.into());
        push_pair(&mut bwrap_args, "--setenv", "GTK_USE_PORTAL");
        bwrap_args.push("1".into());
    }

    bwrap_args.push(program);
    bwrap_args.extend(args);

    let mut wrapped = Command::new("bwrap");
    wrapped.args(bwrap_args);
    wrapped.env_clear();
    for (key, value) in std::env::vars_os() {
        if should_pass_env(&key) {
            wrapped.env(key, value);
        }
    }
    if let Some(current_dir) = current_dir {
        wrapped.current_dir(current_dir);
    }
    for (key, value) in envs {
        if let Some(value) = value {
            wrapped.env(key, value);
        } else {
            wrapped.env_remove(key);
        }
    }

    *command = wrapped;
    configure_stdin_payload(
        command,
        &config.sandbox_dir,
        config.stdin_payload,
    )?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_platform(
    command: &mut Command,
    config: SandboxConfig,
) -> crate::Result<()> {
    use std::ffi::CString;
    use std::os::unix::process::CommandExt;

    let mut profile = String::from(MACOS_BASE_PROFILE);
    profile.push_str(MACOS_NETWORK);

    for path in config.allow_read {
        allow_macos_path(&mut profile, "file-read*", &path);
    }
    for path in config.allow_write {
        allow_macos_path(
            &mut profile,
            "file-write* file-link file-read* file-map-executable process-exec",
            &path,
        );
    }

    let temp = config.sandbox_dir.join("tmp");
    std::fs::create_dir_all(&temp).map_err(crate::util::io::IOError::from)?;
    allow_macos_path(
        &mut profile,
        "file-write* file-read* file-map-executable process-exec",
        &temp,
    );
    profile.push_str(MACOS_PROTECT);

    command.env("TMPDIR", &temp);

    let c_profile = CString::new(profile).map_err(|_| {
        crate::ErrorKind::LauncherError(
            "Generated sandbox profile contained a null byte.".to_string(),
        )
    })?;
    unsafe {
        command.as_std_mut().pre_exec(move || {
            let mut error = std::ptr::null_mut();
            let result = sandbox_init(c_profile.as_ptr(), 0, &mut error);
            if !error.is_null() {
                sandbox_free_error(error);
            }
            if result != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "macOS sandbox_init failed",
                ));
            }
            Ok(())
        });
    }
    configure_stdin_payload(
        command,
        &config.sandbox_dir,
        config.stdin_payload,
    )?;
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn configure_stdin_payload(
    command: &mut Command,
    sandbox_dir: &Path,
    payload: Option<String>,
) -> crate::Result<()> {
    let Some(payload) = payload else {
        return Ok(());
    };

    std::fs::create_dir_all(sandbox_dir)
        .map_err(crate::util::io::IOError::from)?;
    let payload_path = sandbox_dir.join("launch-stdin.payload");
    std::fs::write(&payload_path, payload)
        .map_err(crate::util::io::IOError::from)?;
    let payload_file = std::fs::File::open(&payload_path)
        .map_err(crate::util::io::IOError::from)?;
    let _ = std::fs::remove_file(&payload_path);

    command.stdin(Stdio::from(payload_file));
    command.env(STDIN_PAYLOAD_CONFIGURED_ENV, "1");
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_platform(
    command: &mut Command,
    config: SandboxConfig,
) -> crate::Result<()> {
    use std::ffi::OsStr;

    let original = command.as_std();
    let program = original.get_program().to_string_lossy().to_string();
    let args = original
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let current_dir = original
        .get_current_dir()
        .map(|path| path.to_string_lossy().to_string());
    let mut envs = original
        .get_envs()
        .map(|(key, value)| {
            (
                key.to_string_lossy().to_string(),
                value.map(|value| value.to_string_lossy().to_string()),
            )
        })
        .collect::<Vec<_>>();

    std::fs::create_dir_all(&config.sandbox_dir)
        .map_err(crate::util::io::IOError::from)?;
    let temp_dir = config.sandbox_dir.join("temp");
    std::fs::create_dir_all(&temp_dir)
        .map_err(crate::util::io::IOError::from)?;
    let temp_dir = temp_dir.to_string_lossy().to_string();
    envs.retain(|(key, _)| {
        !matches!(key.to_ascii_uppercase().as_str(), "TEMP" | "TMP" | "TMPDIR")
    });
    envs.push(("TEMP".to_string(), Some(temp_dir.clone())));
    envs.push(("TMP".to_string(), Some(temp_dir.clone())));
    envs.push(("TMPDIR".to_string(), Some(temp_dir)));

    let spec_path = config.sandbox_dir.join("windows-sandbox-launch.json");
    let mut allow_write = config.allow_write;
    allow_write.push(config.sandbox_dir.clone());
    allow_write.sort();
    allow_write.dedup();
    let spec = WindowsSandboxSpec {
        program,
        args,
        current_dir,
        envs,
        allow_read: config
            .allow_read
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        allow_write: allow_write
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        stdin_payload: config.stdin_payload,
        name: "RevoriaInstanceSandbox".to_string(),
        description: "Sandbox for Minecraft instances run by Revoria"
            .to_string(),
    };
    let spec_bytes = serde_json::to_vec_pretty(&spec)?;
    std::fs::write(&spec_path, spec_bytes)
        .map_err(crate::util::io::IOError::from)?;

    let current_exe =
        std::env::current_exe().map_err(crate::util::io::IOError::from)?;
    let mut wrapped = Command::new(current_exe);
    wrapped.arg("--theseus-sandbox-launch");
    wrapped.arg(&spec_path);
    if let Some(current_dir) = original.get_current_dir() {
        wrapped.current_dir(current_dir);
    }
    for (key, value) in original.get_envs() {
        if let Some(value) = value {
            wrapped.env(key, value);
        } else {
            wrapped.env_remove(key);
        }
    }
    if original.get_args().next()
        == Some(OsStr::new("--theseus-sandbox-launch"))
    {
        return Err(crate::ErrorKind::LauncherError(
            "Refusing to recursively apply Windows sandbox.".to_string(),
        )
        .into());
    }

    *command = wrapped;
    Ok(())
}

#[cfg(target_os = "windows")]
#[derive(serde::Deserialize, serde::Serialize)]
struct WindowsSandboxSpec {
    program: String,
    args: Vec<String>,
    current_dir: Option<String>,
    envs: Vec<(String, Option<String>)>,
    allow_read: Vec<String>,
    allow_write: Vec<String>,
    stdin_payload: Option<String>,
    name: String,
    description: String,
}

#[cfg(target_os = "windows")]
pub fn run_windows_sandbox_helper(spec_path: &Path) -> crate::Result<i32> {
    let spec_bytes =
        std::fs::read(spec_path).map_err(crate::util::io::IOError::from)?;
    let spec: WindowsSandboxSpec = serde_json::from_slice(&spec_bytes)?;
    Ok(windows_appcontainer::run(spec)
        .map_err(crate::util::io::IOError::from)?)
}

#[cfg(target_os = "windows")]
pub fn set_windows_sandbox_traverse_acls(
    args: Vec<std::ffi::OsString>,
) -> crate::Result<()> {
    windows_appcontainer::set_traverse_acls(args)
        .map_err(crate::util::io::IOError::from)?;
    Ok(())
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct WindowsSandboxSpawn {
    pub process: WindowsSandboxProcess,
    pub stdout: Option<tokio::fs::File>,
    pub stderr: Option<tokio::fs::File>,
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct WindowsSandboxProcess {
    handle: isize,
    pid: u32,
    _stdin: Option<tokio::fs::File>,
}

#[cfg(target_os = "windows")]
impl WindowsSandboxProcess {
    pub fn id(&self) -> u32 {
        self.pid
    }

    pub fn try_wait(
        &mut self,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        windows_appcontainer::try_wait_process(self.handle)
    }

    pub async fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        windows_appcontainer::wait_process(self.handle).await
    }

    pub async fn kill(&mut self) -> std::io::Result<()> {
        windows_appcontainer::kill_process(self.handle)
    }
}

#[cfg(target_os = "windows")]
impl Drop for WindowsSandboxProcess {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(
                windows::Win32::Foundation::HANDLE(self.handle as *mut _),
            );
        }
    }
}

#[cfg(target_os = "windows")]
pub fn try_spawn_windows_sandboxed_direct(
    command: &mut Command,
) -> crate::Result<Option<WindowsSandboxSpawn>> {
    let original = command.as_std();
    let mut args = original.get_args();
    let Some(first_arg) = args.next() else {
        return Ok(None);
    };
    if first_arg != "--theseus-sandbox-launch" {
        return Ok(None);
    }
    let Some(spec_path) = args.next() else {
        return Ok(None);
    };

    let spec_bytes =
        std::fs::read(spec_path).map_err(crate::util::io::IOError::from)?;
    let spec: WindowsSandboxSpec = serde_json::from_slice(&spec_bytes)?;
    Ok(Some(
        windows_appcontainer::spawn_direct(spec)
            .map_err(crate::util::io::IOError::from)?,
    ))
}

#[cfg(target_os = "windows")]
#[allow(unsafe_op_in_unsafe_fn)]
mod windows_appcontainer {
    use super::{
        WindowsSandboxProcess, WindowsSandboxSpawn, WindowsSandboxSpec,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::ffi::OsString;
    use std::io::{Error, ErrorKind, Write};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::os::windows::io::{AsRawHandle, OwnedHandle};
    use std::path::{Path, PathBuf};
    use std::process::ExitStatus;
    use windows::Win32::Foundation::{
        CloseHandle, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS,
        ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, GetLastError, HANDLE,
        LocalFree, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT,
    };
    use windows::Win32::Security::Authorization::{
        ConvertSidToStringSidW, ConvertStringSidToSidW, EXPLICIT_ACCESS_W,
        GRANT_ACCESS, GetNamedSecurityInfoW, NO_MULTIPLE_TRUSTEE,
        SE_FILE_OBJECT, SetEntriesInAclW, SetNamedSecurityInfoW,
        TRUSTEE_IS_GROUP, TRUSTEE_IS_SID,
    };
    use windows::Win32::Security::Isolation::{
        CreateAppContainerProfile, DeriveAppContainerSidFromAppContainerName,
    };
    use windows::Win32::Security::{
        ACE_HEADER, ACL, CONTAINER_INHERIT_ACE, DACL_SECURITY_INFORMATION,
        FreeSid, GetAce, InitializeSecurityDescriptor, NO_INHERITANCE,
        OBJECT_INHERIT_ACE, PSECURITY_DESCRIPTOR, PSID, SECURITY_CAPABILITIES,
        SID_AND_ATTRIBUTES, SetFileSecurityW, SetSecurityDescriptorDacl,
        WELL_KNOWN_SID_TYPE, WinCapabilityInternetClientServerSid,
        WinCapabilityInternetClientSid,
        WinCapabilityPrivateNetworkClientServerSid,
    };
    use windows::Win32::Storage::FileSystem::{
        FILE_ALL_ACCESS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_TRAVERSE,
    };
    use windows::Win32::System::SystemServices::{
        SE_GROUP_ENABLED, SECURITY_DESCRIPTOR_REVISION,
    };
    use windows::Win32::System::Threading::{
        CREATE_UNICODE_ENVIRONMENT, CreateProcessW,
        DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
        GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList,
        LPPROC_THREAD_ATTRIBUTE_LIST,
        PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, PROCESS_CREATION_FLAGS,
        PROCESS_INFORMATION, STARTF_FORCEONFEEDBACK, STARTF_USESTDHANDLES,
        STARTUPINFOEXW, TerminateProcess, UpdateProcThreadAttribute,
        WaitForSingleObject,
    };
    use windows::core::{HRESULT, PCWSTR, PWSTR};

    pub fn run(spec: WindowsSandboxSpec) -> std::io::Result<i32> {
        let app_container_sid =
            create_app_container(&spec.name, &spec.description)?;
        unsafe {
            apply_file_acls(app_container_sid, &spec)?;

            let exit = spawn_in_appcontainer(app_container_sid, &spec);
            FreeSid(app_container_sid);
            exit
        }
    }

    pub fn spawn_direct(
        spec: WindowsSandboxSpec,
    ) -> std::io::Result<WindowsSandboxSpawn> {
        let app_container_sid =
            create_app_container(&spec.name, &spec.description)?;
        unsafe {
            apply_file_acls(app_container_sid, &spec)?;

            let spawned =
                spawn_in_appcontainer_direct(app_container_sid, &spec);
            FreeSid(app_container_sid);
            spawned
        }
    }

    pub fn try_wait_process(
        handle: isize,
    ) -> std::io::Result<Option<ExitStatus>> {
        unsafe {
            let handle = HANDLE(handle as *mut _);
            let wait = WaitForSingleObject(handle, 0);
            if wait == WAIT_TIMEOUT {
                return Ok(None);
            }
            if wait == WAIT_FAILED {
                return Err(Error::last_os_error());
            }
            if wait != WAIT_OBJECT_0 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "unexpected sandbox child wait result",
                ));
            }
            Ok(Some(exit_status(handle)?))
        }
    }

    pub async fn wait_process(handle: isize) -> std::io::Result<ExitStatus> {
        tokio::task::spawn_blocking(move || unsafe {
            let handle = HANDLE(handle as *mut _);
            let wait = WaitForSingleObject(handle, INFINITE);
            if wait == WAIT_FAILED {
                return Err(Error::last_os_error());
            }
            if wait != WAIT_OBJECT_0 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "unexpected sandbox child wait result",
                ));
            }
            exit_status(handle)
        })
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?
    }

    pub fn kill_process(handle: isize) -> std::io::Result<()> {
        unsafe {
            TerminateProcess(HANDLE(handle as *mut _), 1).map_err(Into::into)
        }
    }

    unsafe fn exit_status(handle: HANDLE) -> std::io::Result<ExitStatus> {
        use std::os::windows::process::ExitStatusExt;

        let mut exit_code = 1;
        GetExitCodeProcess(handle, &mut exit_code)?;
        Ok(ExitStatus::from_raw(exit_code))
    }

    unsafe fn spawn_in_appcontainer(
        app_container_sid: PSID,
        spec: &WindowsSandboxSpec,
    ) -> std::io::Result<i32> {
        let mut attr_size = 0;
        let result =
            InitializeProcThreadAttributeList(None, 1, None, &mut attr_size);
        if result
            != Err(windows::core::Error::from_hresult(HRESULT::from_win32(
                ERROR_INSUFFICIENT_BUFFER.0,
            )))
        {
            result?;
        }
        let mut attr_storage = vec![0u8; attr_size];
        let attr_list =
            LPPROC_THREAD_ATTRIBUTE_LIST(attr_storage.as_mut_ptr().cast());
        InitializeProcThreadAttributeList(
            Some(attr_list),
            1,
            None,
            &mut attr_size,
        )?;

        let mut capabilities = [
            owned_capability(WinCapabilityInternetClientSid)?,
            owned_capability(WinCapabilityInternetClientServerSid)?,
            owned_capability(WinCapabilityPrivateNetworkClientServerSid)?,
        ];
        let mut sid_attrs = capabilities
            .iter_mut()
            .map(|cap| SID_AND_ATTRIBUTES {
                Sid: PSID(cap.as_mut_ptr().cast()),
                Attributes: SE_GROUP_ENABLED as u32,
            })
            .collect::<Vec<_>>();
        let security_capabilities = SECURITY_CAPABILITIES {
            AppContainerSid: app_container_sid,
            Capabilities: sid_attrs.as_mut_ptr(),
            CapabilityCount: sid_attrs.len() as u32,
            Reserved: 0,
        };
        UpdateProcThreadAttribute(
            attr_list,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            Some(
                (&security_capabilities as *const SECURITY_CAPABILITIES).cast(),
            ),
            size_of::<SECURITY_CAPABILITIES>(),
            None,
            None,
        )?;

        let application = wide_null(&spec.program);
        let command_line_args = std::iter::once(spec.program.clone())
            .chain(spec.args.clone())
            .collect::<Vec<_>>();
        let mut command_line =
            wide_null_os(join_windows_args(&command_line_args));
        let current_dir = spec.current_dir.as_ref().map(|x| wide_null(x));
        let mut env_block = build_env_block(&spec.envs);

        let mut startup = STARTUPINFOEXW::default();
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.StartupInfo.dwFlags =
            STARTF_USESTDHANDLES | STARTF_FORCEONFEEDBACK;
        startup.StartupInfo.hStdInput =
            windows::Win32::System::Console::GetStdHandle(
                windows::Win32::System::Console::STD_INPUT_HANDLE,
            )?;
        startup.StartupInfo.hStdOutput =
            windows::Win32::System::Console::GetStdHandle(
                windows::Win32::System::Console::STD_OUTPUT_HANDLE,
            )?;
        startup.StartupInfo.hStdError =
            windows::Win32::System::Console::GetStdHandle(
                windows::Win32::System::Console::STD_ERROR_HANDLE,
            )?;
        startup.lpAttributeList = attr_list;

        let mut process_info = PROCESS_INFORMATION::default();
        CreateProcessW(
            PCWSTR(application.as_ptr()),
            Some(PWSTR(command_line.as_mut_ptr())),
            None,
            None,
            true,
            PROCESS_CREATION_FLAGS(
                CREATE_UNICODE_ENVIRONMENT.0 | EXTENDED_STARTUPINFO_PRESENT.0,
            ),
            Some(env_block.as_mut_ptr().cast()),
            current_dir
                .as_ref()
                .map(|x| PCWSTR(x.as_ptr()))
                .unwrap_or_default(),
            &startup.StartupInfo,
            &mut process_info,
        )?;
        DeleteProcThreadAttributeList(attr_list);
        let _ = CloseHandle(process_info.hThread);

        let wait = WaitForSingleObject(process_info.hProcess, INFINITE);
        if wait == WAIT_FAILED {
            let _ = CloseHandle(process_info.hProcess);
            return Err(Error::last_os_error());
        }
        if wait != WAIT_OBJECT_0 {
            let _ = CloseHandle(process_info.hProcess);
            return Err(Error::new(
                ErrorKind::Other,
                "unexpected sandbox child wait result",
            ));
        }

        let mut exit_code = 1;
        GetExitCodeProcess(process_info.hProcess, &mut exit_code)?;
        let _ = CloseHandle(process_info.hProcess);
        Ok(exit_code as i32)
    }

    unsafe fn spawn_in_appcontainer_direct(
        app_container_sid: PSID,
        spec: &WindowsSandboxSpec,
    ) -> std::io::Result<WindowsSandboxSpawn> {
        let mut attr_size = 0;
        let result =
            InitializeProcThreadAttributeList(None, 1, None, &mut attr_size);
        if result
            != Err(windows::core::Error::from_hresult(HRESULT::from_win32(
                ERROR_INSUFFICIENT_BUFFER.0,
            )))
        {
            result?;
        }
        let mut attr_storage = vec![0u8; attr_size];
        let attr_list =
            LPPROC_THREAD_ATTRIBUTE_LIST(attr_storage.as_mut_ptr().cast());
        InitializeProcThreadAttributeList(
            Some(attr_list),
            1,
            None,
            &mut attr_size,
        )?;

        let mut capabilities = [
            owned_capability(WinCapabilityInternetClientSid)?,
            owned_capability(WinCapabilityInternetClientServerSid)?,
            owned_capability(WinCapabilityPrivateNetworkClientServerSid)?,
        ];
        let mut sid_attrs = capabilities
            .iter_mut()
            .map(|cap| SID_AND_ATTRIBUTES {
                Sid: PSID(cap.as_mut_ptr().cast()),
                Attributes: SE_GROUP_ENABLED as u32,
            })
            .collect::<Vec<_>>();
        let security_capabilities = SECURITY_CAPABILITIES {
            AppContainerSid: app_container_sid,
            Capabilities: sid_attrs.as_mut_ptr(),
            CapabilityCount: sid_attrs.len() as u32,
            Reserved: 0,
        };
        UpdateProcThreadAttribute(
            attr_list,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            Some(
                (&security_capabilities as *const SECURITY_CAPABILITIES).cast(),
            ),
            size_of::<SECURITY_CAPABILITIES>(),
            None,
            None,
        )?;

        let (stdin_read, stdin_write) = std::io::pipe()?;
        let (stdout_read, stdout_write) = std::io::pipe()?;
        let (stderr_read, stderr_write) = std::io::pipe()?;

        let stdin_read: OwnedHandle = stdin_read.into();
        let stdout_write: OwnedHandle = stdout_write.into();
        let stderr_write: OwnedHandle = stderr_write.into();
        windows::Win32::Foundation::SetHandleInformation(
            HANDLE(stdin_read.as_raw_handle()),
            windows::Win32::Foundation::HANDLE_FLAG_INHERIT.0,
            windows::Win32::Foundation::HANDLE_FLAG_INHERIT,
        )?;
        windows::Win32::Foundation::SetHandleInformation(
            HANDLE(stdout_write.as_raw_handle()),
            windows::Win32::Foundation::HANDLE_FLAG_INHERIT.0,
            windows::Win32::Foundation::HANDLE_FLAG_INHERIT,
        )?;
        windows::Win32::Foundation::SetHandleInformation(
            HANDLE(stderr_write.as_raw_handle()),
            windows::Win32::Foundation::HANDLE_FLAG_INHERIT.0,
            windows::Win32::Foundation::HANDLE_FLAG_INHERIT,
        )?;

        let application = wide_null(&spec.program);
        let command_line_args = std::iter::once(spec.program.clone())
            .chain(spec.args.clone())
            .collect::<Vec<_>>();
        let mut command_line =
            wide_null_os(join_windows_args(&command_line_args));
        let current_dir = spec.current_dir.as_ref().map(|x| wide_null(x));
        let mut env_block = build_env_block(&spec.envs);

        let mut startup = STARTUPINFOEXW::default();
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.StartupInfo.dwFlags =
            STARTF_USESTDHANDLES | STARTF_FORCEONFEEDBACK;
        startup.StartupInfo.hStdInput = HANDLE(stdin_read.as_raw_handle());
        startup.StartupInfo.hStdOutput = HANDLE(stdout_write.as_raw_handle());
        startup.StartupInfo.hStdError = HANDLE(stderr_write.as_raw_handle());
        startup.lpAttributeList = attr_list;

        let mut process_info = PROCESS_INFORMATION::default();
        let create_result = CreateProcessW(
            PCWSTR(application.as_ptr()),
            Some(PWSTR(command_line.as_mut_ptr())),
            None,
            None,
            true,
            PROCESS_CREATION_FLAGS(
                CREATE_UNICODE_ENVIRONMENT.0 | EXTENDED_STARTUPINFO_PRESENT.0,
            ),
            Some(env_block.as_mut_ptr().cast()),
            current_dir
                .as_ref()
                .map(|x| PCWSTR(x.as_ptr()))
                .unwrap_or_default(),
            &startup.StartupInfo,
            &mut process_info,
        );
        DeleteProcThreadAttributeList(attr_list);
        create_result?;
        let _ = CloseHandle(process_info.hThread);
        drop(stdin_read);
        drop(stdout_write);
        drop(stderr_write);

        let stdin = if let Some(stdin_payload) = &spec.stdin_payload {
            let mut stdin_write = stdin_write;
            stdin_write.write_all(stdin_payload.as_bytes())?;
            stdin_write.flush()?;
            drop(stdin_write);
            None
        } else {
            let stdin_write: OwnedHandle = stdin_write.into();
            Some(tokio::fs::File::from_std(std::fs::File::from(stdin_write)))
        };
        let stdout_read: OwnedHandle = stdout_read.into();
        let stderr_read: OwnedHandle = stderr_read.into();
        let stdout =
            tokio::fs::File::from_std(std::fs::File::from(stdout_read));
        let stderr =
            tokio::fs::File::from_std(std::fs::File::from(stderr_read));

        Ok(WindowsSandboxSpawn {
            process: WindowsSandboxProcess {
                handle: process_info.hProcess.0 as isize,
                pid: process_info.dwProcessId,
                _stdin: stdin,
            },
            stdout: Some(stdout),
            stderr: Some(stderr),
        })
    }

    fn create_app_container(
        name: &str,
        description: &str,
    ) -> std::io::Result<PSID> {
        let name = wide_null(name);
        let description = wide_null(description);
        let result = unsafe {
            CreateAppContainerProfile(
                PCWSTR(name.as_ptr()),
                PCWSTR(name.as_ptr()),
                PCWSTR(description.as_ptr()),
                None,
            )
        };
        match result {
            Ok(sid) => Ok(sid),
            Err(err)
                if err.code()
                    == HRESULT::from_win32(ERROR_ALREADY_EXISTS.0) =>
            {
                unsafe {
                    DeriveAppContainerSidFromAppContainerName(PCWSTR(
                        name.as_ptr(),
                    ))
                }
                .map_err(Into::into)
            }
            Err(err) => Err(err.into()),
        }
    }

    unsafe fn owned_capability(
        kind: WELL_KNOWN_SID_TYPE,
    ) -> std::io::Result<Vec<u8>> {
        let mut size = 0;
        let result = windows::Win32::Security::CreateWellKnownSid(
            kind, None, None, &mut size,
        );
        if result
            != Err(windows::core::Error::from_hresult(HRESULT::from_win32(
                ERROR_INSUFFICIENT_BUFFER.0,
            )))
        {
            result?;
        }
        let mut sid = vec![0u8; size as usize];
        windows::Win32::Security::CreateWellKnownSid(
            kind,
            None,
            Some(PSID(sid.as_mut_ptr().cast())),
            &mut size,
        )?;
        Ok(sid)
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum PermissionType {
        TraverseNoInherit,
        Read,
        Write,
    }

    unsafe fn apply_file_acls(
        sid: PSID,
        spec: &WindowsSandboxSpec,
    ) -> std::io::Result<()> {
        let mut readable = BTreeSet::<PathBuf>::new();
        let mut parents_to_add = BTreeSet::<PathBuf>::new();

        for path in &spec.allow_read {
            let path = PathBuf::from(path);
            if !path.exists() {
                continue;
            }
            readable.insert(path.clone());
            if let Some(parent) = path.parent() {
                parents_to_add.insert(parent.to_path_buf());
            }
            if let Err(err) = add_to_acl(sid, &path, PermissionType::Read) {
                tracing::error!(
                    "Unable to allow reading from path {path:?}: {err}"
                );
            }
        }

        for path in &spec.allow_write {
            let path = PathBuf::from(path);
            if !path.exists() {
                continue;
            }
            readable.insert(path.clone());
            if let Some(parent) = path.parent() {
                parents_to_add.insert(parent.to_path_buf());
            }
            if let Err(err) = add_to_acl(sid, &path, PermissionType::Write) {
                tracing::error!(
                    "Unable to allow writing to path {path:?}: {err}"
                );
            }
        }

        let program_path = PathBuf::from(&spec.program);
        if program_path.exists() {
            readable.insert(program_path.clone());
            if let Some(parent) = program_path.parent() {
                parents_to_add.insert(parent.to_path_buf());
            }
            let _ = add_to_acl(sid, &program_path, PermissionType::Read);
        }

        let mut parents = BTreeSet::<PathBuf>::new();
        loop {
            let taken = std::mem::take(&mut parents_to_add);
            if taken.is_empty() {
                break;
            }
            for to_add in taken {
                if readable.contains(&to_add) || parents.contains(&to_add) {
                    continue;
                }
                if let Some(parent) = to_add.parent() {
                    parents_to_add.insert(parent.to_path_buf());
                }
                parents.insert(to_add);
            }
        }

        let mut elevated_parents = Vec::new();
        for parent in parents {
            if let Err(err) =
                add_to_acl(sid, &parent, PermissionType::TraverseNoInherit)
            {
                let raw_access_denied = err.raw_os_error().unwrap_or(0)
                    == HRESULT::from_win32(ERROR_ACCESS_DENIED.0).0;
                if err.kind() == ErrorKind::PermissionDenied
                    || raw_access_denied
                {
                    tracing::warn!(
                        "Lacking permission to allow sandbox traversal of {parent:?}; requesting elevation"
                    );
                    elevated_parents.push(parent);
                } else {
                    tracing::error!(
                        "Unable to allow sandbox traversal of path {parent:?}: {err}"
                    );
                }
            }
        }

        if !elevated_parents.is_empty() {
            run_elevated_traverse_acl_helper(sid, &elevated_parents)?;
        }

        Ok(())
    }

    unsafe fn add_to_acl(
        sid: PSID,
        path: &Path,
        permission: PermissionType,
    ) -> std::io::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let mut access = EXPLICIT_ACCESS_W::default();
        access.grfAccessMode = GRANT_ACCESS;
        access.grfAccessPermissions = match permission {
            PermissionType::TraverseNoInherit => {
                (FILE_GENERIC_READ | FILE_TRAVERSE).0
            }
            PermissionType::Read => {
                (FILE_GENERIC_READ | FILE_TRAVERSE | FILE_GENERIC_EXECUTE).0
            }
            PermissionType::Write => FILE_ALL_ACCESS.0,
        };
        access.grfInheritance =
            if permission == PermissionType::TraverseNoInherit {
                NO_INHERITANCE
            } else {
                OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE
            };
        access.Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
        access.Trustee.pMultipleTrustee = std::ptr::null_mut();
        access.Trustee.ptstrName = PWSTR(sid.0.cast());
        access.Trustee.TrusteeForm = TRUSTEE_IS_SID;
        access.Trustee.TrusteeType = TRUSTEE_IS_GROUP;

        let path = wide_null_os(path.as_os_str().to_os_string());
        let mut old_acl = std::ptr::null_mut();
        let err = GetNamedSecurityInfoW(
            PCWSTR(path.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut old_acl),
            None,
            std::ptr::null_mut(),
        );
        if err != ERROR_SUCCESS {
            return Err(windows::core::Error::from_hresult(
                HRESULT::from_win32(err.0),
            )
            .into());
        }

        let mut new_acl = std::ptr::null_mut();
        let err =
            SetEntriesInAclW(Some(&[access]), Some(old_acl), &mut new_acl);
        if new_acl.is_null() {
            return Err(Error::new(ErrorKind::Other, "new acl was null"));
        }
        if err != ERROR_SUCCESS {
            LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl.cast())));
            return Err(windows::core::Error::from_hresult(
                HRESULT::from_win32(err.0),
            )
            .into());
        }

        if acl_eq(old_acl, new_acl)? {
            LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl.cast())));
            return Ok(());
        }

        let err = if permission == PermissionType::TraverseNoInherit {
            let mut security_buf = vec![0; 64];
            InitializeSecurityDescriptor(
                PSECURITY_DESCRIPTOR(security_buf.as_mut_ptr().cast()),
                SECURITY_DESCRIPTOR_REVISION,
            )?;
            SetSecurityDescriptorDacl(
                PSECURITY_DESCRIPTOR(security_buf.as_mut_ptr().cast()),
                true,
                Some(new_acl),
                false,
            )?;
            let success = SetFileSecurityW(
                PCWSTR(path.as_ptr()),
                DACL_SECURITY_INFORMATION,
                PSECURITY_DESCRIPTOR(security_buf.as_mut_ptr().cast()),
            );
            if success.as_bool() {
                ERROR_SUCCESS
            } else {
                GetLastError()
            }
        } else {
            SetNamedSecurityInfoW(
                PWSTR(path.as_ptr().cast_mut()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(new_acl),
                None,
            )
        };
        LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl.cast())));
        if err != ERROR_SUCCESS {
            return Err(windows::core::Error::from_hresult(
                HRESULT::from_win32(err.0),
            )
            .into());
        }
        let _ = GetLastError();
        Ok(())
    }

    fn acl_eq(first: *const ACL, second: *const ACL) -> std::io::Result<bool> {
        if first.is_null() && second.is_null() {
            return Ok(true);
        }
        let first = unsafe { first.as_ref() };
        let Some(first) = first else {
            return Ok(false);
        };
        let second = unsafe { second.as_ref() };
        let Some(second) = second else {
            return Ok(false);
        };
        if first.AceCount != second.AceCount {
            return Ok(false);
        }

        for i in 0..first.AceCount {
            let mut first_ace_ptr = std::ptr::null_mut::<std::ffi::c_void>();
            let mut second_ace_ptr = std::ptr::null_mut::<std::ffi::c_void>();
            unsafe { GetAce(first, i as u32, &mut first_ace_ptr)? };
            unsafe { GetAce(second, i as u32, &mut second_ace_ptr)? };

            if first_ace_ptr.is_null() && second_ace_ptr.is_null() {
                continue;
            }
            let first_ace_header =
                unsafe { first_ace_ptr.cast::<ACE_HEADER>().as_ref() };
            let Some(first_ace_header) = first_ace_header else {
                return Ok(false);
            };
            let second_ace_header =
                unsafe { second_ace_ptr.cast::<ACE_HEADER>().as_ref() };
            let Some(second_ace_header) = second_ace_header else {
                return Ok(false);
            };

            if first_ace_header != second_ace_header {
                return Ok(false);
            }

            let first_data = unsafe {
                std::slice::from_raw_parts(
                    first_ace_ptr.cast::<u8>(),
                    first_ace_header.AceSize as usize,
                )
            };
            let second_data = unsafe {
                std::slice::from_raw_parts(
                    second_ace_ptr.cast::<u8>(),
                    second_ace_header.AceSize as usize,
                )
            };

            if first_data != second_data {
                return Ok(false);
            }
        }

        Ok(true)
    }

    unsafe fn sid_to_os_string(sid: PSID) -> std::io::Result<OsString> {
        let mut sid_string = PWSTR::default();
        ConvertSidToStringSidW(sid, &mut sid_string)?;
        if sid_string.is_null() {
            return Err(Error::new(
                ErrorKind::Other,
                "ConvertSidToStringSidW returned null",
            ));
        }
        let value = OsString::from_wide(sid_string.as_wide());
        LocalFree(Some(windows::Win32::Foundation::HLOCAL(
            sid_string.0.cast(),
        )));
        Ok(value)
    }

    unsafe fn run_elevated_traverse_acl_helper(
        sid: PSID,
        parents: &[PathBuf],
    ) -> std::io::Result<()> {
        let sid = sid_to_os_string(sid)?;
        let exe = std::env::current_exe()?;
        let mut args = Vec::with_capacity(parents.len() + 2);
        args.push(OsString::from("--theseus-set-traverse-acls"));
        args.push(sid);
        args.extend(parents.iter().map(|path| path.as_os_str().to_os_string()));

        let mut script = String::from("$p = Start-Process -FilePath ");
        script.push_str(&quote_powershell_single(
            &exe.as_os_str().to_string_lossy(),
        ));
        script.push_str(" -ArgumentList @(");
        for (index, arg) in args.iter().enumerate() {
            if index != 0 {
                script.push(',');
            }
            script.push_str(&quote_powershell_single(&arg.to_string_lossy()));
        }
        script.push_str(") -Verb RunAs -Wait -PassThru; exit $p.ExitCode");

        let status = std::process::Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(script)
            .status()?;
        if !status.success() {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                format!("elevated traverse ACL helper failed: {status}"),
            ));
        }
        Ok(())
    }

    fn quote_powershell_single(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }

    pub fn set_traverse_acls(args: Vec<OsString>) -> std::io::Result<()> {
        if args.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "missing sid"));
        }

        let string_sid = args[0].encode_wide().chain([0]).collect::<Vec<_>>();
        let mut sid = PSID::default();
        unsafe {
            ConvertStringSidToSidW(PCWSTR(string_sid.as_ptr()), &mut sid)?
        };
        if sid.is_invalid() {
            return Err(Error::new(
                ErrorKind::Other,
                "ConvertStringSidToSidW returned invalid sid",
            ));
        }

        let mut first_error = None;
        for arg in &args[1..] {
            if let Err(err) = unsafe {
                add_to_acl(
                    sid,
                    Path::new(arg),
                    PermissionType::TraverseNoInherit,
                )
            } {
                if first_error.is_none() {
                    first_error = Some(err);
                }
            }
        }
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(sid.0))) };

        if let Some(err) = first_error {
            return Err(err);
        }
        Ok(())
    }

    fn build_env_block(envs: &[(String, Option<String>)]) -> Vec<u16> {
        let mut map = std::env::vars_os().collect::<BTreeMap<_, _>>();
        for (key, value) in envs {
            if let Some(value) = value {
                map.insert(OsString::from(key), OsString::from(value));
            } else {
                map.remove(&OsString::from(key));
            }
        }

        let mut block = Vec::new();
        for (key, value) in map {
            block.extend(key.encode_wide());
            block.push('=' as u16);
            block.extend(value.encode_wide());
            block.push(0);
        }
        block.push(0);
        block
    }

    fn join_windows_args(args: &[String]) -> OsString {
        let mut joined = OsString::new();
        for (index, arg) in args.iter().enumerate() {
            if index != 0 {
                joined.push(" ");
            }
            joined.push(quote_windows_arg(arg));
        }
        joined
    }

    fn quote_windows_arg(arg: &str) -> String {
        if arg.is_empty() {
            return "\"\"".to_string();
        }
        let needs_quotes =
            arg.bytes().any(|b| matches!(b, b' ' | b'\t' | b'"'));
        if !needs_quotes {
            return arg.to_string();
        }
        let mut quoted = String::from("\"");
        let mut backslashes = 0;
        for ch in arg.chars() {
            if ch == '\\' {
                backslashes += 1;
            } else if ch == '"' {
                quoted.push_str(&"\\".repeat(backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            } else {
                quoted.push_str(&"\\".repeat(backslashes));
                backslashes = 0;
                quoted.push(ch);
            }
        }
        quoted.push_str(&"\\".repeat(backslashes * 2));
        quoted.push('"');
        quoted
    }

    fn wide_null(value: &str) -> Vec<u16> {
        OsString::from(value).encode_wide().chain([0]).collect()
    }

    fn wide_null_os(value: OsString) -> Vec<u16> {
        value.encode_wide().chain([0]).collect()
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows"
)))]
fn apply_platform(
    _command: &mut Command,
    _config: SandboxConfig,
) -> crate::Result<()> {
    Err(crate::ErrorKind::LauncherError(
        "Sandbox is not supported on this operating system.".to_string(),
    )
    .into())
}

#[cfg(target_os = "linux")]
const DEVICE_BINDS: &[&str] = &[
    "/dev/dri",
    "/dev/udmabuf",
    "/dev/mali",
    "/dev/mali0",
    "/dev/umplock",
    "/dev/kgsl-3d0",
    "/dev/ion",
    "/dev/disk/by-uuid",
    "/dev/dm",
    "/dev/loop",
    "/dev/mapper",
    "/dev/ram",
    "/dev/ntsync",
    "/dev/snd",
];

#[cfg(target_os = "linux")]
const SYSTEM_READ_ONLY: &[&str] = &[
    "/bin",
    "/sbin",
    "/usr",
    "/lib",
    "/lib32",
    "/lib64",
    "/etc/alternatives",
    "/etc/resolv.conf",
    "/run/systemd/resolve",
    "/usr/share/ca-certificates",
    "/etc/ca-certificates",
    "/etc/ssl",
    "/etc/pki",
    "/etc/pkcs11",
    "/etc/hosts",
    "/etc/ld.so.cache",
    "/etc/ld.so.conf.d",
    "/etc/localtime",
    "/etc/os-release",
    "/etc/machine-id",
    "/etc/timezone",
    "/etc/fonts",
    "/sys/dev/char",
    "/sys/bus/pci/devices",
    "/sys/devices/system/cpu",
    "/sys/devices/virtual/dmi/id",
    "/sys/class/net",
    "/sys/firmware/devicetree/base/model",
    "/sys/class/power_supply",
    "/sys/class/hwmon",
    "/sys/class/thermal",
    "/sys/class/drm",
];

#[cfg(target_os = "linux")]
fn push(args: &mut Vec<OsString>, value: &str) {
    args.push(value.into());
}

#[cfg(target_os = "linux")]
fn push_pair(args: &mut Vec<OsString>, a: &str, b: &str) {
    args.push(a.into());
    args.push(b.into());
}

#[cfg(target_os = "linux")]
fn push_pair_os(
    args: &mut Vec<OsString>,
    a: &str,
    b: impl Into<OsString>,
    c: impl Into<OsString>,
) {
    args.push(a.into());
    args.push(b.into());
    args.push(c.into());
}

#[cfg(target_os = "linux")]
fn bind_if_exists(args: &mut Vec<OsString>, kind: &str, path: &Path) {
    let Ok(path) = path.canonicalize() else {
        return;
    };
    args.push(kind.into());
    args.push(path.as_os_str().to_os_string());
    args.push(path.into_os_string());
}

#[cfg(target_os = "linux")]
fn bind_nvidia_devices(args: &mut Vec<OsString>) {
    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("nvidia"))
            {
                bind_if_exists(args, "--dev-bind-try", &path);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn bind_runtime_desktop(args: &mut Vec<OsString>) {
    let Some(runtime) = runtime_dir() else {
        return;
    };
    push_pair_os(
        args,
        "--dir",
        runtime.as_os_str().to_os_string(),
        OsString::new(),
    );
    let _ = args.pop();

    let wayland = std::env::var_os("WAYLAND_DISPLAY")
        .unwrap_or_else(|| "wayland-0".into());
    bind_if_exists(args, "--ro-bind-try", &runtime.join(wayland));
    bind_if_exists(args, "--ro-bind-try", &runtime.join("pipewire-0"));
    bind_if_exists(args, "--ro-bind-try", &runtime.join("pulse"));
    bind_if_exists(args, "--bind-try", Path::new("/run/pulse"));

    if let Some(display) = std::env::var_os("DISPLAY") {
        let display = display.to_string_lossy();
        let index = display
            .strip_prefix(':')
            .and_then(|x| x.chars().next())
            .filter(|x| x.is_ascii_digit())
            .unwrap_or('0');
        bind_if_exists(
            args,
            "--ro-bind-try",
            Path::new(&format!("/tmp/.X11-unix/X{index}")),
        );
    }
    if let Some(xauthority) = std::env::var_os("XAUTHORITY") {
        bind_if_exists(args, "--ro-bind-try", Path::new(&xauthority));
    }
}

#[cfg(target_os = "linux")]
fn start_dbus_proxy(sandbox_dir: &Path) -> crate::Result<Option<PathBuf>> {
    let Some(session_bus_address) =
        std::env::var_os("DBUS_SESSION_BUS_ADDRESS")
    else {
        return Ok(None);
    };

    let proxy_dir = sandbox_dir.join("dbus-proxy");
    std::fs::create_dir_all(&proxy_dir)
        .map_err(crate::util::io::IOError::from)?;
    let proxy_sock = proxy_dir.join("session.sock");

    std::process::Command::new("xdg-dbus-proxy")
        .arg(session_bus_address)
        .arg(&proxy_sock)
        .arg("--filter")
        .arg("--talk=com.feralinteractive.GameMode")
        .arg("--call=com.feralinteractive.GameMode=/com/feralinteractive/GameMode")
        .arg("--talk=org.kde.StatusNotifierWatcher")
        .arg("--call=org.kde.StatusNotifierWatcher=/StatusNotifierWatcher")
        .arg("--talk=org.freedesktop.Notifications")
        .arg("--call=org.freedesktop.Notifications=/org/freedesktop/Notifications")
        .arg("--talk=org.freedesktop.portal.*")
        .arg("--talk=org.mpris.MediaPlayer2.*")
        .spawn()
        .map_err(crate::util::io::IOError::from)?;

    Ok(Some(proxy_sock))
}

#[cfg(target_os = "linux")]
fn runtime_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from)
}

#[cfg(target_os = "linux")]
fn is_command_available(command: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|path| path.join(command).exists())
    })
}

#[cfg(target_os = "linux")]
fn should_pass_env(key: &std::ffi::OsStr) -> bool {
    key.as_encoded_bytes().starts_with(b"XDG_")
        || [
            "GDMSESSION",
            "DESKTOP_SESSION",
            "PATH",
            "LANG",
            "LC_ALL",
            "TERM",
            "USER",
            "USERNAME",
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "PULSE_SERVER",
        ]
        .iter()
        .any(|allowed| key == std::ffi::OsStr::new(allowed))
}

#[cfg(target_os = "macos")]
fn allow_macos_path(profile: &mut String, permissions: &str, path: &Path) {
    let Ok(path) = path.canonicalize() else {
        return;
    };
    let kind = if path.is_dir() { "subpath" } else { "literal" };
    profile.push_str("(allow ");
    profile.push_str(permissions);
    profile.push_str(" (");
    profile.push_str(kind);
    profile.push_str(" \"");
    profile.push_str(
        &path
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\""),
    );
    profile.push_str("\"))\n");
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn sandbox_init(
        profile: *const libc::c_char,
        flags: u64,
        errorbuf: *mut *mut libc::c_char,
    ) -> libc::c_int;
    fn sandbox_free_error(errorbuf: *mut libc::c_char);
}

#[cfg(target_os = "macos")]
const MACOS_BASE_PROFILE: &str = r#"
(version 1)
(deny default)
(import "system.sb")
(deny nvram*)
(deny process-info*)
(deny file-link)
(allow hid-control process-fork lsopen)
(allow signal (target same-sandbox))
(allow process-info-pidinfo)
(allow process-info-pidfdinfo process-info-pidfileportinfo process-info-setcontrol process-info-dirtycontrol process-info-rusage process-info-ledger (target self))
(allow sysctl-write (sysctl-name "kern.tcsm_enable"))
(system-graphics)
(allow file-read-metadata)
(allow file-read*
  (literal "/private/etc/hosts")
  (literal "/private/etc/passwd")
  (literal "/private/etc/resolv.conf")
  (literal "/private/etc/ssl/cert.pem")
  (literal "/private/var/run/resolv.conf")
  (subpath "/Library/Audio")
  (subpath "/bin")
  (subpath "/sbin")
  (subpath "/usr/bin")
  (subpath "/usr/sbin"))
(allow file-read* file-write* (subpath "/dev/fd"))
(allow user-preference-read)
(allow mach-lookup)
(allow device-microphone)
"#;

#[cfg(target_os = "macos")]
const MACOS_NETWORK: &str = r#"
(system-network)
(allow network-outbound (literal "/private/var/run/mDNSResponder"))
(allow network-outbound (remote ip))
(allow network-inbound (local ip))
"#;

#[cfg(target_os = "macos")]
const MACOS_PROTECT: &str = r#"
(deny network-outbound (literal "/private/var/run/cupsd"))
(deny network-outbound (remote ip "localhost:631"))
(deny file-write-xattr (xattr "com.apple.quarantine") (with no-log))
(deny file-read-xattr file-write-xattr (xattr-prefix "com.apple.security.private."))
"#;
