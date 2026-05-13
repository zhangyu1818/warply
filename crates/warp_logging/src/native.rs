use std::path::{Path, PathBuf};
use std::{
    env,
    fs::{self, File},
    io::{IsTerminal, Write},
};

use anyhow::Result;
use log::LevelFilter;
use std::sync::OnceLock;

use crate::{LogConfig, LogDestination};
use warp_core::channel::ChannelState;

const MAX_FILES_IN_GUI_ROTATION: usize = 5;
const MAX_FILES_IN_CLI_ROTATION: usize = 10;
const CLI_LOG_SUBDIRECTORY: &str = "cli";
const TEMP_LOG_FILE_SUFFIX: &str = "old.temp";

/// Runtime logging state, computed from `LogConfig` during initialization.
#[derive(Debug)]
struct LogState {
    /// Whether or not logs should be written to a file.
    use_logfile: bool,

    /// The directory that logs should be written to. This is set even if `use_logfile` is false,
    /// as we sometimes generate other log files.
    log_directory: PathBuf,

    /// The maximum number of backup log files to keep during rotation.
    max_rotation: usize,
}

static LOG_STATE: OnceLock<LogState> = OnceLock::new();

/// Formats a log record to be output to the terminal.
fn format_for_terminal_output(
    buf: &mut env_logger::fmt::Formatter,
    record: &log::Record,
) -> std::io::Result<()> {
    let level = record.level();
    let mut level_style = buf.default_level_style(record.level());
    // Adjust colors to match what we're used to from simplelog.
    match &level {
        log::Level::Info => {
            level_style.set_color(env_logger::fmt::Color::Blue);
        }
        log::Level::Debug => {
            level_style.set_color(env_logger::fmt::Color::Green);
        }
        _ => {}
    }
    let level = level_style.value(format!("[{level}]"));

    let mut target_style = buf.style();
    let target = if cfg!(debug_assertions) {
        target_style.set_dimmed(true);
        target_style.value(format!("[{}] ", record.target()))
    } else {
        target_style.value(String::default())
    };

    let time = chrono::Local::now();
    writeln!(
        buf,
        "{} {level} {target}{}",
        time.format("%H:%M:%S%.3f"),
        record.args()
    )
}

/// Formats a log record to be output to a file.
fn format_for_file_output(
    buf: &mut env_logger::fmt::Formatter,
    record: &log::Record,
) -> std::io::Result<()> {
    let target = if cfg!(debug_assertions) {
        format!("[{}] ", record.target())
    } else {
        String::default()
    };

    writeln!(
        buf,
        "{} [{}] {}{}",
        buf.timestamp(),
        record.level(),
        target,
        record.args()
    )
}

/// Rotates the log files, such that:
/// - Each file stores the logs of a single execution.
/// - The .old files store the previous executions, with larger suffixes indicating older executions.
pub async fn rotate_log_files() {
    let config = LOG_STATE.get().expect("Logging not initialized");
    if !config.use_logfile {
        return;
    }

    let max_rotation = config.max_rotation;

    if let Err(err) = rotate_files(&ChannelState::logfile_name(), max_rotation).await {
        log::error!("Failed to rotate log files: {err:?}");
    }
}

pub async fn rotate_files(channel_file_name: &str, max_rotation: usize) -> Result<()> {
    let log_directory = match log_directory() {
        Ok(log_directory) => log_directory,
        Err(err) => {
            return Err(anyhow::anyhow!("Could not get log directory {err:?}"));
        }
    };

    // Delete the oldest log file.
    let largest_log_file_suffix = max_rotation.saturating_sub(1);
    let _ = fs::remove_file(
        log_directory.join(format!("{channel_file_name}.old.{largest_log_file_suffix}")),
    );

    // Rotate the log files.
    for file_no in (0..largest_log_file_suffix).rev() {
        let old_file_path = log_directory.join(format!("{channel_file_name}.old.{file_no}"));
        let new_file_path = log_directory.join(format!("{channel_file_name}.old.{}", file_no + 1));
        let _ = fs::rename(old_file_path, new_file_path);
    }

    // Rename `warp.log.old.temp` (the temporary file) to `warp.log.old.0`.
    let temp_file_path = temp_log_file_path(&log_directory);

    let _ = fs::rename(
        temp_file_path,
        log_directory.join(format!("{channel_file_name}.old.0")),
    );

    Ok(())
}

/// Initializes the global logger for the application.
/// If `config.log_destination` is `Some`, always use the specified destination regardless of
/// environment. If `config.is_cli` is true, logs are written to a separate CLI subdirectory with
/// a higher rotation limit so that CLI invocations don't evict GUI application logs.
pub fn init(config: LogConfig) -> Result<()> {
    init_internal(config.is_cli, config.log_destination)
}

/// Returns the path to the main process's log file.
fn main_process_log_file_path(log_directory: impl AsRef<Path>) -> PathBuf {
    log_directory.as_ref().join(&*ChannelState::logfile_name())
}

/// Returns the path to the current execution's main log file.
///
/// Note: logging must be initialized before calling this function, otherwise this will
/// return an error.
pub fn log_file_path() -> Result<PathBuf> {
    let dir = log_directory()?;
    Ok(main_process_log_file_path(&dir))
}

fn temp_log_file_path(log_directory: impl AsRef<Path>) -> PathBuf {
    let channel_logfile_name = ChannelState::logfile_name();
    log_directory
        .as_ref()
        .join(format!("{channel_logfile_name}.{TEMP_LOG_FILE_SUFFIX}"))
}

fn init_internal(is_cli: bool, log_destination: Option<LogDestination>) -> Result<()> {
    /// Returns an empty file named `warp.log` to log the current execution, and
    /// renames the previous execution's log to a temporary name.
    fn setup_log_files_for_current_execution(log_directory: &Path) -> Result<File> {
        fs::create_dir_all(log_directory)?;

        let main_log_path = main_process_log_file_path(log_directory);
        let _ = fs::rename(main_log_path.clone(), temp_log_file_path(log_directory));

        let main_log_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(main_log_path)?;
        Ok(main_log_file)
    }

    let mut base_logger = env_logger::builder();

    base_logger.filter_level(LevelFilter::Info);

    // Only include `WARN` or higher logs for wgpu. By default, wgpu outputs logs at the `INFO`
    // level multiple times _per_ frame. See https://github.com/gfx-rs/wgpu/issues/3206.
    // Naga is overly noisy at `DEBUG`, so increase to `INFO`.
    base_logger
        .filter(Some("naga"), LevelFilter::Info)
        .filter(Some("wgpu_core"), LevelFilter::Warn)
        // Since we always pair an insertion with a deletion to avoid duplicate,
        // tantivy will log a lot of warnings for deleting a non-existing doc.
        .filter(Some("tantivy"), LevelFilter::Error)
        .filter(
            Some("wgpu_hal"),
            // On Windows with the DX12 backend, wgpu_hal outputs a ton of WARN-level logs.
            if cfg!(windows) {
                LevelFilter::Error
            } else {
                LevelFilter::Warn
            },
        );
    base_logger.parse_default_env();

    let stdout_is_a_tty = std::io::stdout().is_terminal();
    let in_ci = env::var("CI").is_ok();
    let integration_test = env::var("WARP_INTEGRATION").is_ok();
    let use_logfile = match log_destination {
        Some(LogDestination::File) => true,
        Some(LogDestination::Stderr) => false,
        None => !stdout_is_a_tty && !in_ci && !integration_test,
    };

    let max_rotation = if is_cli {
        MAX_FILES_IN_CLI_ROTATION
    } else {
        MAX_FILES_IN_GUI_ROTATION
    };

    let mut log_directory = init_log_directory()?;
    if is_cli {
        log_directory = log_directory.join(CLI_LOG_SUBDIRECTORY);
    }
    if use_logfile {
        base_logger.target(env_logger::Target::Pipe(Box::new(
            setup_log_files_for_current_execution(&log_directory)?,
        )));
        base_logger.format(format_for_file_output);
    } else {
        base_logger.write_style(env_logger::WriteStyle::Always);
        base_logger.format(format_for_terminal_output);
    }

    base_logger.init();

    LOG_STATE
        .set(LogState {
            use_logfile,
            log_directory,
            max_rotation,
        })
        .expect("Logging already initialized");
    // We can .expect here because .init would have already panicked if we initialized logging twice.

    Ok(())
}

pub fn log_directory() -> Result<std::path::PathBuf> {
    LOG_STATE
        .get()
        .map(|config| config.log_directory.clone())
        .ok_or_else(|| anyhow::anyhow!("Logging not initialized"))
}

fn init_log_directory() -> Result<std::path::PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            Ok(dirs::home_dir()
                .ok_or_else(|| {
                    anyhow::anyhow!("could not locate home directory in order to create a log file")
                })?
                .join("Library/Logs/"))
        } else if #[cfg(any(target_os = "linux", target_os = "freebsd"))] {
            Ok(warp_core::paths::state_dir())
        } else if #[cfg(windows)] {
            Ok(warp_core::paths::state_dir().join(warp_core::paths::WARPLY_LOGS_DIR))
        } else {
            Err(anyhow::anyhow!("Have not configured file-based logging for the current platform!"))
        }
    }
}

/// Initializes the logger before running tests.
///
/// Additionally, we must not write anything to stdout in this function, as it
/// can interfere with test harnesses collecting the set of tests to run.  (This
/// is why we're not simply calling the init() function above.)
pub fn init_logging_for_unit_tests() {
    env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .parse_default_env()
        .format(format_for_terminal_output)
        .init();
}
