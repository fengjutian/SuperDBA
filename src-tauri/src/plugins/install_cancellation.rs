use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use tokio::sync::Notify;

pub const INSTALL_CANCELLED_ERROR: &str = "PLUGIN_INSTALL_CANCELLED";

struct CancellationInner {
    cancelled: AtomicBool,
    notify: Notify,
}

#[derive(Clone)]
pub struct InstallCancellation {
    inner: Arc<CancellationInner>,
}

impl InstallCancellation {
    fn new() -> Self {
        Self {
            inner: Arc::new(CancellationInner {
                cancelled: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Acquire)
    }

    pub fn check(&self) -> Result<(), String> {
        if self.is_cancelled() {
            Err(INSTALL_CANCELLED_ERROR.to_string())
        } else {
            Ok(())
        }
    }

    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }

        let notified = self.inner.notify.notified();
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }

    fn cancel(&self) {
        if !self.inner.cancelled.swap(true, Ordering::AcqRel) {
            self.inner.notify.notify_waiters();
        }
    }
}

static ACTIVE_INSTALLS: Lazy<Mutex<HashMap<String, InstallCancellation>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct InstallGuard {
    plugin_id: String,
    cancellation: InstallCancellation,
}

impl InstallGuard {
    pub fn cancellation(&self) -> &InstallCancellation {
        &self.cancellation
    }
}

impl Drop for InstallGuard {
    fn drop(&mut self) {
        let mut installs = ACTIVE_INSTALLS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        installs.remove(&self.plugin_id);
    }
}

pub fn begin(plugin_id: &str) -> Result<InstallGuard, String> {
    let mut installs = ACTIVE_INSTALLS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if installs.contains_key(plugin_id) {
        return Err(format!(
            "An installation is already running for plugin '{}'",
            plugin_id
        ));
    }

    let cancellation = InstallCancellation::new();
    installs.insert(plugin_id.to_string(), cancellation.clone());
    Ok(InstallGuard {
        plugin_id: plugin_id.to_string(),
        cancellation,
    })
}

pub fn cancel(plugin_id: &str) -> bool {
    let cancellation = ACTIVE_INSTALLS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(plugin_id)
        .cloned();

    if let Some(cancellation) = cancellation {
        cancellation.cancel();
        true
    } else {
        false
    }
}
