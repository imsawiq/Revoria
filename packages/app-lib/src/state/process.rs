use crate::event::emit::{emit_process, emit_profile};
use crate::event::{ProcessPayloadType, ProfilePayloadType};
use crate::profile;
use crate::util::io::IOError;
use crate::util::rpc::RpcServer;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use dashmap::DashMap;
use quick_xml::Reader;
use quick_xml::events::Event;
use serde::Deserialize;
use serde::Serialize;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use uuid::Uuid;

const LAUNCHER_LOG_PATH: &str = "launcher_log.txt";
const RUNNING_PROCESSES_STATE: &str = "running_processes.json";
const STDIN_PAYLOAD_CONFIGURED_ENV: &str =
    "MODRINTH_SANDBOX_STDIN_PAYLOAD_CONFIGURED";

pub struct ProcessManager {
    processes: DashMap<Uuid, Process>,
    state_file: PathBuf,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self {
            processes: DashMap::new(),
            state_file: PathBuf::from(RUNNING_PROCESSES_STATE),
        }
    }
}

impl ProcessManager {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            processes: DashMap::new(),
            state_file: settings_dir.join(RUNNING_PROCESSES_STATE),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn insert_new_process(
        &self,
        profile_path: &str,
        mut mc_command: Command,
        post_exit_command: Option<String>,
        logs_folder: PathBuf,
        xml_logging: bool,
        main_class_keep_alive: TempDir,
        rpc_server: RpcServer,
        post_process_init: impl AsyncFnOnce(
            &ProcessMetadata,
            &RpcServer,
        ) -> crate::Result<()>,
    ) -> crate::Result<ProcessMetadata> {
        mc_command.stdout(std::process::Stdio::piped());
        mc_command.stderr(std::process::Stdio::piped());
        if stdin_payload_is_configured(&mc_command) {
            mc_command.env_remove(STDIN_PAYLOAD_CONFIGURED_ENV);
        } else {
            mc_command.stdin(std::process::Stdio::piped());
        }

        let SpawnedMinecraftProcess {
            child,
            pid,
            stdout,
            stderr,
        } = spawn_minecraft_process(&mut mc_command)?;

        let mut process = Process {
            metadata: ProcessMetadata {
                uuid: Uuid::new_v4(),
                start_time: Utc::now(),
                profile_path: profile_path.to_string(),
                pid,
                recovered: false,
                last_playtime_update: None,
            },
            child: Some(child),
            rpc_server: Some(rpc_server),
            _main_class_keep_alive: Some(main_class_keep_alive),
        };
        process.metadata.last_playtime_update =
            Some(process.metadata.start_time);

        let metadata = process.metadata.clone();

        if !logs_folder.exists() {
            tokio::fs::create_dir_all(&logs_folder)
                .await
                .map_err(|e| IOError::with_path(e, &logs_folder))?;
        }

        let log_path = logs_folder.join(LAUNCHER_LOG_PATH);

        {
            let mut log_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&log_path)
                .map_err(|e| IOError::with_path(e, &log_path))?;

            // Initialize with timestamp header
            let now = chrono::Local::now();
            writeln!(
                log_file,
                "# Minecraft launcher log started at {}",
                now.format("%Y-%m-%d %H:%M:%S")
            )
            .map_err(|e| IOError::with_path(e, &log_path))?;
            writeln!(log_file, "# Profile: {profile_path} \n")
                .map_err(|e| IOError::with_path(e, &log_path))?;
            writeln!(log_file).map_err(|e| IOError::with_path(e, &log_path))?;
        }

        if let Some(stdout) = stdout {
            let log_path_clone = log_path.clone();

            let profile_path = metadata.profile_path.clone();
            tokio::spawn(async move {
                Process::process_output(
                    &profile_path,
                    stdout,
                    log_path_clone,
                    xml_logging,
                )
                .await;
            });
        }

        if let Some(stderr) = stderr {
            let log_path_clone = log_path.clone();

            let profile_path = metadata.profile_path.clone();
            tokio::spawn(async move {
                Process::process_output(
                    &profile_path,
                    stderr,
                    log_path_clone,
                    xml_logging,
                )
                .await;
            });
        }

        if let Err(e) = post_process_init(
            &process.metadata,
            process
                .rpc_server
                .as_ref()
                .expect("RPC server should exist for launched processes"),
        )
        .await
        {
            tracing::error!("Failed to run post-process init: {e}");
            if let Some(child) = process.child.as_mut() {
                let _ = child.kill().await;
            }
            return Err(e);
        }

        tokio::spawn(Process::sequential_process_manager(
            profile_path.to_string(),
            post_exit_command,
            metadata.uuid,
            metadata.playtime_checkpoint(),
        ));

        self.processes.insert(process.metadata.uuid, process);
        self.persist_running_processes()?;

        emit_process(
            profile_path,
            metadata.uuid,
            ProcessPayloadType::Launched,
            "Launched Minecraft",
        )
        .await?;

        Ok(metadata)
    }

    pub fn get(&self, id: Uuid) -> Option<ProcessMetadata> {
        self.processes.get(&id).map(|x| x.metadata.clone())
    }

    pub fn get_rpc(&self, id: Uuid) -> Option<RpcServer> {
        self.processes.get(&id).and_then(|x| x.rpc_server.clone())
    }

    pub fn get_all(&self) -> Vec<ProcessMetadata> {
        self.prune_dead_recovered();
        self.processes
            .iter()
            .map(|x| x.value().metadata.clone())
            .collect()
    }

    pub fn try_wait(
        &self,
        id: Uuid,
    ) -> crate::Result<Option<Option<ExitStatus>>> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            if let Some(child) = process.child.as_mut() {
                Ok(Some(child.try_wait()?))
            } else if process.is_running() {
                Ok(Some(None))
            } else {
                Ok(Some(Some(ExitStatus::default())))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn wait_for(&self, id: Uuid) -> crate::Result<()> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            if let Some(child) = process.child.as_mut() {
                child.wait().await?;
                return Ok(());
            }
        }

        if let Some(process) = self.processes.get(&id) {
            if process.metadata.recovered {
                let pid = process.metadata.pid;
                drop(process);
                while Self::is_pid_running(pid) {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200))
                        .await;
                }
                self.remove(id);
            }
        }
        Ok(())
    }

    pub async fn kill(&self, id: Uuid) -> crate::Result<()> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            if let Some(child) = process.child.as_mut() {
                child.kill().await?;
                return Ok(());
            }
        }

        if let Some(mut process) = self.processes.get_mut(&id) {
            if process.metadata.recovered {
                process.kill_recovered()?;
                drop(process);
                self.remove(id);
            }
        }

        Ok(())
    }

    fn remove(&self, id: Uuid) {
        self.processes.remove(&id);
        let _ = self.persist_running_processes();
    }

    fn update_playtime_checkpoint(&self, id: Uuid, checkpoint: DateTime<Utc>) {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process.metadata.last_playtime_update = Some(checkpoint);
        }
        let _ = self.persist_running_processes();
    }

    pub fn recover_persisted_processes(&self) -> crate::Result<()> {
        let Ok(contents) = std::fs::read_to_string(&self.state_file) else {
            return Ok(());
        };

        let persisted: Vec<ProcessMetadata> =
            serde_json::from_str(&contents).unwrap_or_default();

        for mut metadata in persisted {
            if metadata.recovered || Self::is_pid_running(metadata.pid) {
                metadata.recovered = true;
                metadata.last_playtime_update =
                    Some(metadata.playtime_checkpoint());
                let uuid = metadata.uuid;
                let profile_path = metadata.profile_path.clone();
                let last_playtime_update = metadata.playtime_checkpoint();

                self.processes.insert(
                    uuid,
                    Process {
                        metadata,
                        child: None,
                        _main_class_keep_alive: None,
                        rpc_server: None,
                    },
                );

                tokio::spawn(Process::sequential_process_manager(
                    profile_path,
                    None,
                    uuid,
                    last_playtime_update,
                ));
            }
        }

        self.persist_running_processes()?;
        Ok(())
    }

    fn prune_dead_recovered(&self) {
        let dead = self
            .processes
            .iter()
            .filter(|entry| {
                entry.value().metadata.recovered && !entry.value().is_running()
            })
            .map(|entry| *entry.key())
            .collect::<Vec<_>>();

        if !dead.is_empty() {
            for uuid in dead {
                self.processes.remove(&uuid);
            }
            let _ = self.persist_running_processes();
        }
    }

    fn persist_running_processes(&self) -> crate::Result<()> {
        let persisted = self
            .processes
            .iter()
            .filter_map(|entry| {
                let metadata = &entry.value().metadata;
                if Self::is_pid_running(metadata.pid) {
                    Some(metadata.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if let Some(parent) = self.state_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(
            &self.state_file,
            serde_json::to_vec_pretty(&persisted)?,
        )?;
        Ok(())
    }

    fn is_pid_running(pid: u32) -> bool {
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);
        system.process(Pid::from_u32(pid)).is_some()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcessMetadata {
    pub uuid: Uuid,
    pub profile_path: String,
    pub start_time: DateTime<Utc>,
    pub pid: u32,
    #[serde(default)]
    pub recovered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_playtime_update: Option<DateTime<Utc>>,
}

impl ProcessMetadata {
    fn playtime_checkpoint(&self) -> DateTime<Utc> {
        self.last_playtime_update.unwrap_or(self.start_time)
    }
}

#[derive(Debug)]
struct Process {
    metadata: ProcessMetadata,
    child: Option<MinecraftChild>,
    _main_class_keep_alive: Option<TempDir>,
    rpc_server: Option<RpcServer>,
}

impl Process {
    fn is_running(&self) -> bool {
        if let Some(child) = self.child.as_ref() {
            child.id().is_some()
        } else {
            ProcessManager::is_pid_running(self.metadata.pid)
        }
    }

    fn kill_recovered(&mut self) -> crate::Result<()> {
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);
        if let Some(process) = system.process(Pid::from_u32(self.metadata.pid))
        {
            process.kill();
        }
        Ok(())
    }
}

struct SpawnedMinecraftProcess {
    child: MinecraftChild,
    pid: u32,
    stdout: Option<Box<dyn AsyncRead + Send + Unpin>>,
    stderr: Option<Box<dyn AsyncRead + Send + Unpin>>,
}

fn spawn_minecraft_process(
    command: &mut Command,
) -> crate::Result<SpawnedMinecraftProcess> {
    #[cfg(target_os = "windows")]
    if let Some(child) =
        crate::util::sandbox::try_spawn_windows_sandboxed_direct(command)?
    {
        let pid = child.process.id();
        let stdout = child.stdout.map(|stdout| {
            Box::new(stdout) as Box<dyn AsyncRead + Send + Unpin>
        });
        let stderr = child.stderr.map(|stderr| {
            Box::new(stderr) as Box<dyn AsyncRead + Send + Unpin>
        });
        return Ok(SpawnedMinecraftProcess {
            child: MinecraftChild::WindowsSandbox(child.process),
            pid,
            stdout,
            stderr,
        });
    }

    let mut child = command.spawn().map_err(IOError::from)?;
    let pid = child.id().ok_or_else(|| {
        crate::ErrorKind::LauncherError(
            "Launched process did not return a PID".to_string(),
        )
    })?;
    let stdout = child
        .stdout
        .take()
        .map(|stdout| Box::new(stdout) as Box<dyn AsyncRead + Send + Unpin>);
    let stderr = child
        .stderr
        .take()
        .map(|stderr| Box::new(stderr) as Box<dyn AsyncRead + Send + Unpin>);

    Ok(SpawnedMinecraftProcess {
        child: MinecraftChild::Tokio(child),
        pid,
        stdout,
        stderr,
    })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn stdin_payload_is_configured(command: &Command) -> bool {
    command.as_std().get_envs().any(|(key, value)| {
        key == OsStr::new(STDIN_PAYLOAD_CONFIGURED_ENV) && value.is_some()
    })
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn stdin_payload_is_configured(_command: &Command) -> bool {
    false
}

#[derive(Debug)]
enum MinecraftChild {
    Tokio(Child),
    #[cfg(target_os = "windows")]
    WindowsSandbox(crate::util::sandbox::WindowsSandboxProcess),
}

impl MinecraftChild {
    fn id(&self) -> Option<u32> {
        match self {
            MinecraftChild::Tokio(child) => child.id(),
            #[cfg(target_os = "windows")]
            MinecraftChild::WindowsSandbox(child) => Some(child.id()),
        }
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        match self {
            MinecraftChild::Tokio(child) => child.try_wait(),
            #[cfg(target_os = "windows")]
            MinecraftChild::WindowsSandbox(child) => child.try_wait(),
        }
    }

    async fn wait(&mut self) -> std::io::Result<ExitStatus> {
        match self {
            MinecraftChild::Tokio(child) => child.wait().await,
            #[cfg(target_os = "windows")]
            MinecraftChild::WindowsSandbox(child) => child.wait().await,
        }
    }

    async fn kill(&mut self) -> std::io::Result<()> {
        match self {
            MinecraftChild::Tokio(child) => child.kill().await,
            #[cfg(target_os = "windows")]
            MinecraftChild::WindowsSandbox(child) => child.kill().await,
        }
    }
}

#[derive(Debug, Default)]
struct Log4jEvent {
    timestamp: Option<String>,
    logger: Option<String>,
    level: Option<String>,
    thread: Option<String>,
    message: Option<String>,
}

impl Process {
    async fn process_output<R>(
        profile_path: &str,
        reader: R,
        log_path: impl AsRef<Path>,
        xml_logging: bool,
    ) where
        R: tokio::io::AsyncRead + Unpin,
    {
        let mut buf_reader = BufReader::new(reader);

        if xml_logging {
            let mut reader = Reader::from_reader(buf_reader);
            reader.config_mut().enable_all_checks(false);

            let mut buf = Vec::new();
            let mut current_event = Log4jEvent::default();
            let mut in_event = false;
            let mut in_message = false;
            let mut in_throwable = false;
            let mut current_content = String::new();

            loop {
                match reader.read_event_into_async(&mut buf).await {
                    Err(e) => {
                        tracing::error!(
                            "Error at position {}: {:?}",
                            reader.buffer_position(),
                            e
                        );
                        break;
                    }
                    // exits the loop when reaching end of file
                    Ok(Event::Eof) => break,

                    Ok(Event::Start(e)) => {
                        match e.name().as_ref() {
                            b"log4j:Event" => {
                                // Reset for new event
                                current_event = Log4jEvent::default();
                                in_event = true;

                                // Extract attributes
                                for attr in e.attributes().flatten() {
                                    let key = String::from_utf8_lossy(
                                        attr.key.into_inner(),
                                    )
                                    .to_string();
                                    let value =
                                        String::from_utf8_lossy(&attr.value)
                                            .to_string();

                                    match key.as_str() {
                                        "logger" => {
                                            current_event.logger = Some(value)
                                        }
                                        "level" => {
                                            current_event.level = Some(value)
                                        }
                                        "thread" => {
                                            current_event.thread = Some(value)
                                        }
                                        "timestamp" => {
                                            current_event.timestamp =
                                                Some(value)
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            b"log4j:Message" => {
                                in_message = true;
                                current_content = String::new();
                            }
                            b"log4j:Throwable" => {
                                in_throwable = true;
                                current_content = String::new();
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::End(e)) => {
                        match e.name().as_ref() {
                            b"log4j:Message" => {
                                in_message = false;
                                current_event.message =
                                    Some(current_content.clone());
                            }
                            b"log4j:Throwable" => {
                                in_throwable = false;
                                // Process and write the log entry
                                let thread = current_event
                                    .thread
                                    .as_deref()
                                    .unwrap_or("");
                                let level = current_event
                                    .level
                                    .as_deref()
                                    .unwrap_or("");
                                let logger = current_event
                                    .logger
                                    .as_deref()
                                    .unwrap_or("");

                                if let Some(message) = &current_event.message {
                                    let formatted_time =
                                        Process::format_timestamp(
                                            current_event.timestamp.as_deref(),
                                        );
                                    let formatted_log = format!(
                                        "{} [{}] [{}{}]: {}\n",
                                        formatted_time,
                                        thread,
                                        if !logger.is_empty() {
                                            format!("{logger}/")
                                        } else {
                                            String::new()
                                        },
                                        level,
                                        message.trim()
                                    );

                                    // Write the log message
                                    if let Err(e) = Process::append_to_log_file(
                                        &log_path,
                                        &formatted_log,
                                    ) {
                                        tracing::error!(
                                            "Failed to write to log file: {}",
                                            e
                                        );
                                    }

                                    // Write the throwable if present
                                    if !current_content.is_empty()
                                        && let Err(e) =
                                            Process::append_to_log_file(
                                                &log_path,
                                                &current_content,
                                            )
                                    {
                                        tracing::error!(
                                            "Failed to write throwable to log file: {}",
                                            e
                                        );
                                    }
                                }
                            }
                            b"log4j:Event" => {
                                in_event = false;
                                // If no throwable was present, write the log entry at the end of the event
                                if current_event.message.is_some()
                                    && !in_throwable
                                {
                                    let thread = current_event
                                        .thread
                                        .as_deref()
                                        .unwrap_or("");
                                    let level = current_event
                                        .level
                                        .as_deref()
                                        .unwrap_or("");
                                    let logger = current_event
                                        .logger
                                        .as_deref()
                                        .unwrap_or("");
                                    let message = current_event
                                        .message
                                        .as_deref()
                                        .unwrap_or("")
                                        .trim();

                                    let formatted_time =
                                        Process::format_timestamp(
                                            current_event.timestamp.as_deref(),
                                        );
                                    let formatted_log = format!(
                                        "{} [{}] [{}{}]: {}\n",
                                        formatted_time,
                                        thread,
                                        if !logger.is_empty() {
                                            format!("{logger}/")
                                        } else {
                                            String::new()
                                        },
                                        level,
                                        message
                                    );

                                    // Write the log message
                                    if let Err(e) = Process::append_to_log_file(
                                        &log_path,
                                        &formatted_log,
                                    ) {
                                        tracing::error!(
                                            "Failed to write to log file: {}",
                                            e
                                        );
                                    }

                                    if let Some(timestamp) =
                                        current_event.timestamp.as_deref()
                                        && let Err(e) = Self::maybe_handle_server_join_logging(
                                            profile_path,
                                            timestamp,
                                            message
                                        ).await {
                                            tracing::error!("Failed to handle server join logging: {e}");
                                        }
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::Text(mut e)) => {
                        if in_message || in_throwable {
                            if let Ok(text) = e.xml_content() {
                                current_content.push_str(&text);
                            }
                        } else if !in_event
                            && !e.inplace_trim_end()
                            && !e.inplace_trim_start()
                            && let Ok(text) = e.xml_content()
                            && let Err(e) = Process::append_to_log_file(
                                &log_path,
                                &format!("{text}\n"),
                            )
                        {
                            tracing::error!(
                                "Failed to write to log file: {}",
                                e
                            );
                        }
                    }
                    Ok(Event::CData(e)) => {
                        if (in_message || in_throwable)
                            && let Ok(text) = e.xml_content()
                        {
                            current_content.push_str(&text);
                        }
                    }
                    _ => (),
                }

                buf.clear();
            }
        } else {
            let mut line = String::new();

            while let Ok(bytes_read) = buf_reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break; // End of stream
                }

                if !line.is_empty() {
                    if let Err(e) = Self::append_to_log_file(&log_path, &line) {
                        tracing::warn!("Failed to write to log file: {}", e);
                    }
                    if let Err(e) = Self::maybe_handle_old_server_join_logging(
                        profile_path,
                        line.trim_ascii_end(),
                    )
                    .await
                    {
                        tracing::error!(
                            "Failed to handle old server join logging: {e}"
                        );
                    }
                }

                line.clear();
            }
        }
    }

    fn format_timestamp(timestamp: Option<&str>) -> String {
        if let Some(timestamp_str) = timestamp {
            if let Ok(timestamp_val) = timestamp_str.parse::<i64>() {
                let datetime_utc = if timestamp_val > i32::MAX as i64 {
                    let secs = timestamp_val / 1000;
                    let nsecs = ((timestamp_val % 1000) * 1_000_000) as u32;

                    chrono::DateTime::<Utc>::from_timestamp(secs, nsecs)
                        .unwrap_or_default()
                } else {
                    chrono::DateTime::<Utc>::from_timestamp_secs(timestamp_val)
                        .unwrap_or_default()
                };

                let datetime_local = datetime_utc.with_timezone(&chrono::Local);
                format!("[{}]", datetime_local.format("%H:%M:%S"))
            } else {
                "[??:??:??]".to_string()
            }
        } else {
            "[??:??:??]".to_string()
        }
    }

    fn append_to_log_file(
        path: impl AsRef<Path>,
        line: &str,
    ) -> std::io::Result<()> {
        let mut file =
            OpenOptions::new().append(true).create(true).open(path)?;

        file.write_all(line.as_bytes())?;
        Ok(())
    }

    fn append_exit_summary(
        log_path: impl AsRef<Path>,
        logs_folder: impl AsRef<Path>,
        exit_status: ExitStatus,
    ) -> std::io::Result<()> {
        let log_path = log_path.as_ref();
        let logs_folder = logs_folder.as_ref();
        let mut summary =
            format!("\n# Process exited with status: {exit_status}\n");

        if exit_status.success() {
            summary.push_str("# Result: Minecraft closed normally.\n");
        } else {
            summary.push_str(
                "# Result: Minecraft crashed or was stopped by an external error.\n",
            );
            summary.push_str(
                "# Check the lines above for the first ERROR/FATAL entry; that is usually the real cause.\n",
            );

            if let Some(crash_report) = Self::latest_crash_report(logs_folder) {
                summary.push_str(&format!(
                    "# Latest crash report: {}\n",
                    crash_report.display()
                ));

                if let Some(reason) = Self::extract_crash_reason(&crash_report)
                {
                    summary.push_str(&format!(
                        "# Likely crash reason: {reason}\n"
                    ));
                }
            } else {
                summary.push_str(
                    "# No crash-report file was generated. This often means Java failed before Minecraft could write a report, or the process was killed.\n",
                );
            }
        }

        Self::append_to_log_file(log_path, &summary)
    }

    fn latest_crash_report(logs_folder: &Path) -> Option<PathBuf> {
        let crash_reports = logs_folder.parent()?.join("crash-reports");
        let entries = std::fs::read_dir(crash_reports).ok()?;

        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if !path.is_file() {
                    return None;
                }

                let filename = path.file_name()?.to_string_lossy();
                if !filename.starts_with("crash-") {
                    return None;
                }

                let modified = entry.metadata().ok()?.modified().ok()?;
                Some((modified, path))
            })
            .max_by_key(|(modified, _)| *modified)
            .map(|(_, path)| path)
    }

    fn extract_crash_reason(path: &Path) -> Option<String> {
        let contents = std::fs::read_to_string(path).ok()?;
        let mut description = None;
        let mut caused_by = None;

        for line in contents.lines() {
            let line = line.trim();
            if description.is_none() {
                description = line
                    .strip_prefix("Description:")
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned);
            }
            if caused_by.is_none() {
                caused_by = line
                    .strip_prefix("Caused by:")
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned);
            }

            if description.is_some() && caused_by.is_some() {
                break;
            }
        }

        caused_by.or(description)
    }

    async fn maybe_handle_server_join_logging(
        profile_path: &str,
        timestamp: &str,
        message: &str,
    ) -> crate::Result<()> {
        let timestamp = timestamp
            .parse::<i64>()
            .map(|x| x / 1000)
            .map_err(|x| {
                crate::ErrorKind::OtherError(format!(
                    "Failed to parse timestamp: {x}"
                ))
            })
            .and_then(|x| {
                Utc.timestamp_opt(x, 0).single().ok_or_else(|| {
                    crate::ErrorKind::OtherError(
                        "Failed to convert timestamp to DateTime".to_string(),
                    )
                })
            })?;
        Self::parse_and_insert_server_join(profile_path, message, timestamp)
            .await
    }

    async fn maybe_handle_old_server_join_logging(
        profile_path: &str,
        line: &str,
    ) -> crate::Result<()> {
        if let Some((timestamp, message)) = line.split_once(" [CLIENT] [INFO] ")
        {
            let timestamp =
                NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")?
                    .and_local_timezone(chrono::Local)
                    .map(|x| x.to_utc())
                    .single()
                    .unwrap_or_else(Utc::now);
            Self::parse_and_insert_server_join(profile_path, message, timestamp)
                .await
        } else {
            Self::parse_and_insert_server_join(profile_path, line, Utc::now())
                .await
        }
    }

    async fn parse_and_insert_server_join(
        profile_path: &str,
        message: &str,
        timestamp: DateTime<Utc>,
    ) -> crate::Result<()> {
        let Some(host_port_string) = message.strip_prefix("Connecting to ")
        else {
            return Ok(());
        };
        let Some((host, port_string)) = host_port_string.rsplit_once(", ")
        else {
            return Ok(());
        };
        let Some(port) = port_string.parse::<u16>().ok() else {
            return Ok(());
        };

        let state = crate::State::get().await?;
        crate::state::server_join_log::JoinLogEntry {
            profile_path: profile_path.to_owned(),
            host: host.to_string(),
            port,
            join_time: timestamp,
        }
        .upsert(&state.pool)
        .await?;
        {
            let profile_path = profile_path.to_owned();
            let host = host.to_owned();
            tokio::spawn(async move {
                let _ = emit_profile(
                    &profile_path,
                    ProfilePayloadType::ServerJoined {
                        host,
                        port,
                        timestamp,
                    },
                )
                .await;
            });
        }

        Ok(())
    }

    // Spawns a new child process and inserts it into the hashmap
    // Also, as the process ends, it spawns the follow-up process if it exists
    // By convention, ExitStatus is last command's exit status, and we exit on the first non-zero exit status
    async fn sequential_process_manager(
        profile_path: String,
        post_exit_command: Option<String>,
        uuid: Uuid,
        initial_last_updated_playtime: DateTime<Utc>,
    ) -> crate::Result<()> {
        async fn update_playtime(
            last_updated_playtime: &mut DateTime<Utc>,
            profile_path: &str,
            uuid: Uuid,
            force_update: bool,
        ) {
            let now = Utc::now();
            let diff = now
                .signed_duration_since(*last_updated_playtime)
                .num_seconds();
            if diff >= 60 || force_update {
                if diff <= 0 {
                    *last_updated_playtime = now;
                    if let Ok(state) = crate::State::get().await {
                        state
                            .process_manager
                            .update_playtime_checkpoint(uuid, now);
                    }
                    return;
                }

                if let Err(e) = profile::edit(profile_path, |prof| {
                    prof.recent_time_played += diff as u64;
                    async { Ok(()) }
                })
                .await
                {
                    tracing::warn!(
                        "Failed to update playtime for profile {}: {}",
                        &profile_path,
                        e
                    );
                    return;
                }
                *last_updated_playtime = now;
                if let Ok(state) = crate::State::get().await {
                    state.process_manager.update_playtime_checkpoint(uuid, now);
                }
            }
        }

        // Wait on current Minecraft Child
        let mc_exit_status;
        let mut last_updated_playtime = initial_last_updated_playtime;

        let state = crate::State::get().await?;
        loop {
            if let Some(process) = state.process_manager.try_wait(uuid)? {
                if let Some(t) = process {
                    mc_exit_status = t;
                    break;
                }
            } else {
                mc_exit_status = ExitStatus::default();
                break;
            }

            // sleep for 10ms
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Auto-update playtime every minute
            update_playtime(
                &mut last_updated_playtime,
                &profile_path,
                uuid,
                false,
            )
            .await;
        }

        state.process_manager.remove(uuid);
        emit_process(
            &profile_path,
            uuid,
            ProcessPayloadType::Finished,
            "Exited process",
        )
        .await?;

        // Now fully complete- update playtime one last time
        update_playtime(&mut last_updated_playtime, &profile_path, uuid, true)
            .await;

        // Publish play time update
        // Allow failure, it will be stored locally and sent next time
        // Sent in another thread as first call may take a couple seconds and hold up process ending
        let profile = profile_path.clone();
        tokio::spawn(async move {
            if let Err(e) = profile::try_update_playtime(&profile).await {
                tracing::warn!(
                    "Failed to update playtime for profile {}: {}",
                    profile,
                    e
                );
            }
        });

        let logs_folder = state.directories.profile_logs_dir(&profile_path);
        let log_path = logs_folder.join(LAUNCHER_LOG_PATH);

        if log_path.exists()
            && let Err(e) = Process::append_exit_summary(
                &log_path,
                &logs_folder,
                mc_exit_status,
            )
        {
            tracing::warn!("Failed to write exit status to log file: {}", e);
        }

        let _ = state.discord_rpc.clear_to_default(true).await;

        let _ = state.friends_socket.update_status(None).await;

        // If in tauri, window should show itself again after process exists if it was hidden
        #[cfg(feature = "tauri")]
        {
            let window = crate::EventState::get_main_window().await?;
            if let Some(window) = window {
                window.unminimize()?;
                window.set_focus()?;
            }
        }

        if mc_exit_status.success() {
            // We do not wait on the post exist command to finish running! We let it spawn + run on its own.
            // This behaviour may be changed in the future
            if let Some(hook) = post_exit_command {
                let mut cmd = shlex::split(&hook)
                    .ok_or_else(|| {
                        crate::ErrorKind::LauncherError(format!(
                            "Invalid post-exit command: {hook}",
                        ))
                    })?
                    .into_iter();

                if let Some(command) = cmd.next() {
                    let mut command = Command::new(command);
                    command.args(cmd).current_dir(
                        profile::get_full_path(&profile_path).await?,
                    );
                    command.spawn().map_err(IOError::from)?;
                }
            }
        }

        Ok(())
    }
}
