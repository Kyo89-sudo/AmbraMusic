//! Windows session-local single instance, with a wake event for tray/minimized windows.
use std::{ffi::c_void, io};
type Handle = *mut c_void;
const ERROR_ALREADY_EXISTS: u32 = 183;
const WAIT_OBJECT_0: u32 = 0;
const ALLOW_FOREGROUND_ANY: u32 = u32::MAX;
const INSTANCE_NAME: &str = "Local\\AmbraMusic.Desktop";
#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(attributes: *const c_void, owner: i32, name: *const u16) -> Handle;
    fn CreateEventW(
        attributes: *const c_void,
        manual: i32,
        initial: i32,
        name: *const u16,
    ) -> Handle;
    fn GetLastError() -> u32;
    fn SetEvent(event: Handle) -> i32;
    fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
    fn CloseHandle(handle: Handle) -> i32;
}
#[link(name = "user32")]
extern "system" {
    fn AllowSetForegroundWindow(process: u32) -> i32;
    fn FindWindowW(class: *const u16, title: *const u16) -> Handle;
    fn SetForegroundWindow(window: Handle) -> i32;
}
struct OwnedHandle(Handle);
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: this wrapper owns a valid kernel handle exactly once.
        unsafe {
            CloseHandle(self.0);
        }
    }
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
pub struct Instance {
    _mutex: OwnedHandle,
    event: OwnedHandle,
}
impl Instance {
    /// None means another running instance was notified; the caller must exit.
    pub fn acquire() -> io::Result<Option<Self>> {
        Self::acquire_named(INSTANCE_NAME)
    }
    fn acquire_named(name: &str) -> io::Result<Option<Self>> {
        let event_name = wide(&format!("{name}.Wake"));
        let mutex_name = wide(&format!("{name}.Instance"));
        // Create the shared event first so even simultaneous starts cannot lose a wake request.
        // SAFETY: NUL-terminated names live through each call; null security uses Windows defaults.
        unsafe {
            let event = CreateEventW(std::ptr::null(), 0, 0, event_name.as_ptr());
            if event.is_null() {
                return Err(io::Error::last_os_error());
            }
            let event = OwnedHandle(event);
            let mutex = CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr());
            let status = GetLastError();
            if mutex.is_null() {
                return Err(io::Error::from_raw_os_error(status as i32));
            }
            let mutex = OwnedHandle(mutex);
            if status == ERROR_ALREADY_EXISTS {
                // ERROR_ALREADY_EXISTS
                AllowSetForegroundWindow(ALLOW_FOREGROUND_ANY);
                if SetEvent(event.0) == 0 {
                    return Err(io::Error::last_os_error());
                }
                return Ok(None);
            }
            Ok(Some(Self {
                _mutex: mutex,
                event,
            }))
        }
    }
    pub fn wake_requested(&self) -> bool {
        // SAFETY: the event remains owned by self; zero timeout never blocks the UI.
        unsafe { WaitForSingleObject(self.event.0, 0) == WAIT_OBJECT_0 }
    }
    pub fn activate_window() {
        let title = wide(concat!("AmbraMusic ", env!("CARGO_PKG_VERSION")));
        // SAFETY: valid NUL-terminated title; only foreground our own named app window.
        unsafe {
            let window = FindWindowW(std::ptr::null(), title.as_ptr());
            if !window.is_null() {
                SetForegroundWindow(window);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_child() {
        if let Ok(name) = std::env::var("AMBRA_INSTANCE_TEST_NAME") {
            assert!(Instance::acquire_named(&name).unwrap().is_none());
        }
    }
    #[test]
    fn duplicate_signals_first_and_exit_releases_instance() {
        let name = format!("Local\\AmbraMusic.Test.{}", std::process::id());
        let first = Instance::acquire_named(&name).unwrap().unwrap();
        assert!(!first.wake_requested());
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tests::duplicate_child"])
            .env("AMBRA_INSTANCE_TEST_NAME", &name)
            .status()
            .unwrap();
        assert!(status.success());
        assert!(first.wake_requested());
        assert!(!first.wake_requested());
        drop(first);
        assert!(Instance::acquire_named(&name).unwrap().is_some());
    }
}
