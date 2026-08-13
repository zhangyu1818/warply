use std::ffi::{CStr, c_char};
use std::sync::Mutex;

use async_channel::{Receiver, Sender};
use warpui::{Entity, ModelContext, SingletonEntity};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdaterStatus {
    Unavailable,
    Idle,
    Checking,
    UpdateAvailable { version: String },
    Error { message: String },
}

#[derive(Clone, Debug)]
pub enum UpdaterEvent {
    StatusChanged,
}

#[derive(Clone, Debug)]
enum NativeUpdaterEvent {
    Unavailable,
    Idle,
    UpdateAvailable { version: String },
    Checking,
    Error { message: String },
}

pub struct WarplyUpdater {
    status: UpdaterStatus,
    event_receiver: Receiver<NativeUpdaterEvent>,
}

static EVENT_SENDER: Mutex<Option<Sender<NativeUpdaterEvent>>> = Mutex::new(None);

impl WarplyUpdater {
    pub fn new(ctx: &mut ModelContext<Self>) -> Self {
        let (event_sender, event_receiver) = async_channel::unbounded();
        *EVENT_SENDER.lock().expect("updater sender mutex poisoned") = Some(event_sender);

        unsafe {
            warply_sparkle_set_event_callback(Some(handle_native_updater_event));
        }

        let mut model = Self {
            status: UpdaterStatus::Unavailable,
            event_receiver,
        };
        model.schedule_event_receiver(ctx);

        if unsafe { warply_sparkle_start() } {
            model.status = UpdaterStatus::Idle;
            unsafe {
                warply_sparkle_check_for_update_information();
            }
        }

        model
    }

    #[cfg(test)]
    pub fn new_for_test(_ctx: &mut ModelContext<Self>) -> Self {
        let (_event_sender, event_receiver) = async_channel::unbounded();
        Self {
            status: UpdaterStatus::Unavailable,
            event_receiver,
        }
    }

    pub fn status(&self) -> &UpdaterStatus {
        &self.status
    }

    pub fn has_update_available(&self) -> bool {
        matches!(self.status, UpdaterStatus::UpdateAvailable { .. })
    }

    pub fn check_for_updates(&mut self, ctx: &mut ModelContext<Self>) {
        if unsafe { warply_sparkle_check_for_updates() } {
            self.status = UpdaterStatus::Checking;
        } else {
            self.status = UpdaterStatus::Unavailable;
        }
        ctx.emit(UpdaterEvent::StatusChanged);
    }

    fn schedule_event_receiver(&mut self, ctx: &mut ModelContext<Self>) {
        let receiver = self.event_receiver.clone();
        ctx.spawn(async move { receiver.recv().await }, |model, event, ctx| {
            if let Ok(event) = event {
                model.handle_native_event(event, ctx);
                model.schedule_event_receiver(ctx);
            }
        });
    }

    fn handle_native_event(&mut self, event: NativeUpdaterEvent, ctx: &mut ModelContext<Self>) {
        self.status = match event {
            NativeUpdaterEvent::Unavailable => UpdaterStatus::Unavailable,
            NativeUpdaterEvent::Idle => {
                if self.has_update_available() {
                    self.status.clone()
                } else {
                    UpdaterStatus::Idle
                }
            }
            NativeUpdaterEvent::Checking => UpdaterStatus::Checking,
            NativeUpdaterEvent::UpdateAvailable { version } => {
                UpdaterStatus::UpdateAvailable { version }
            }
            NativeUpdaterEvent::Error { message } => UpdaterStatus::Error { message },
        };
        ctx.emit(UpdaterEvent::StatusChanged);
    }
}

impl Entity for WarplyUpdater {
    type Event = UpdaterEvent;
}

impl SingletonEntity for WarplyUpdater {}

extern "C" fn handle_native_updater_event(
    event: i32,
    version: *const c_char,
    message: *const c_char,
) {
    let version = c_string(version);
    let message = c_string(message);

    let event = match event {
        0 => NativeUpdaterEvent::Unavailable,
        1 => NativeUpdaterEvent::Idle,
        2 => NativeUpdaterEvent::UpdateAvailable {
            version: version
                .as_deref()
                .map(format_update_version_for_display)
                .unwrap_or_else(|| "unknown".to_string()),
        },
        3 => NativeUpdaterEvent::Checking,
        4 => NativeUpdaterEvent::Error {
            message: message.unwrap_or_else(|| "Sparkle updater error".to_string()),
        },
        _ => NativeUpdaterEvent::Error {
            message: "Unknown Sparkle updater event".to_string(),
        },
    };

    if let Some(sender) = EVENT_SENDER
        .lock()
        .expect("updater sender mutex poisoned")
        .as_ref()
    {
        let _ = sender.try_send(event);
    }
}

fn c_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }

    Some(
        unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned(),
    )
}

fn format_update_version_for_display(version: &str) -> String {
    let (date, build) = match version.split_once('.') {
        Some((date, build)) if !build.is_empty() && build.chars().all(|c| c.is_ascii_digit()) => {
            (date, Some(build))
        }
        Some(_) => return version.to_string(),
        None => (version, None),
    };

    if date.len() != 8 || !date.chars().all(|c| c.is_ascii_digit()) {
        return version.to_string();
    }

    let mut formatted = format!("{}.{}.{}", &date[0..4], &date[4..6], &date[6..8]);
    if let Some(build) = build {
        formatted.push('.');
        formatted.push_str(build);
    }
    formatted
}

unsafe extern "C" {
    fn warply_sparkle_set_event_callback(
        callback: Option<extern "C" fn(i32, *const c_char, *const c_char)>,
    );
    fn warply_sparkle_start() -> bool;
    fn warply_sparkle_check_for_update_information() -> bool;
    fn warply_sparkle_check_for_updates() -> bool;
}

#[cfg(test)]
mod tests {
    use super::format_update_version_for_display;

    #[test]
    fn formats_warply_bundle_versions_for_display() {
        assert_eq!(format_update_version_for_display("20260519"), "2026.05.19");
        assert_eq!(
            format_update_version_for_display("20260519.1"),
            "2026.05.19.1"
        );
        assert_eq!(
            format_update_version_for_display("20260519.12"),
            "2026.05.19.12"
        );
    }

    #[test]
    fn leaves_non_warply_versions_unchanged() {
        assert_eq!(format_update_version_for_display("1.2.3"), "1.2.3");
        assert_eq!(format_update_version_for_display("beta"), "beta");
        assert_eq!(format_update_version_for_display("2026051"), "2026051");
    }
}
