use parking_lot::Mutex;
use std::collections::HashMap;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct CancelRegistry {
    flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
    cli_child: Mutex<Option<Child>>,
}

impl CancelRegistry {
    pub fn new() -> Self {
        Self {
            flags: Mutex::new(HashMap::new()),
            cli_child: Mutex::new(None),
        }
    }

    pub fn begin(&self, id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.flags.lock().insert(id.to_string(), Arc::clone(&flag));
        flag
    }

    pub fn is_cancelled(&self, id: &str) -> bool {
        self.flags
            .lock()
            .get(id)
            .is_some_and(|f| f.load(Ordering::SeqCst))
    }

    pub fn cancel(&self, id: &str) {
        if let Some(f) = self.flags.lock().get(id) {
            f.store(true, Ordering::SeqCst);
        }
        if let Some(mut child) = self.cli_child.lock().take() {
            let _ = child.kill();
        }
    }

    pub fn finish(&self, id: &str) {
        self.flags.lock().remove(id);
        *self.cli_child.lock() = None;
    }

    pub fn set_cli_child(&self, child: Child) {
        *self.cli_child.lock() = Some(child);
    }

    pub fn take_cli_child(&self) -> Option<Child> {
        self.cli_child.lock().take()
    }
}

impl Default for CancelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
