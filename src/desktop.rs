//! Integrazione desktop: URL, directory dati e geometria della finestra.
use std::path::PathBuf;

/// Apre un indirizzo nel browser predefinito. Solo i siti del progetto: la GUI non può chiedere
/// di aprire altro.
pub(super) fn open_url(url: &str) {
    const ALLOWED: [&str; 3] = [
        "https://music.youtube.com/",
        "https://github.com/Kyo89-sudo/AmbraMusic",
        "https://github.com/SimoHypers/limusic",
    ];
    if !ALLOWED.iter().any(|a| url.starts_with(a)) {
        return;
    }
    #[cfg(windows)]
    let _ = std::process::Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
}
pub(super) fn open_browser() {
    open_url("https://music.youtube.com/");
}
/// Nascosta nell'area di notifica la finestra non disegna nulla: si restituisce a Windows la
/// memoria inutilizzata (pagine riprese dal sistema e ricaricate solo se servono).
pub(super) fn trim_memory() {
    #[cfg(windows)]
    {
        use std::ffi::c_void;
        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> *mut c_void;
            fn SetProcessWorkingSetSize(process: *mut c_void, min: usize, max: usize) -> i32;
        }
        // SAFETY: (usize::MAX, usize::MAX) è il valore documentato per "svuota il working set".
        unsafe {
            SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
        }
    }
}
pub(super) fn data_dir() -> PathBuf {
    directories::ProjectDirs::from("io", "AmbraMusic", "AmbraMusic")
        .map(|p| p.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("ambramusic-data"))
}
/// Fit the actual native outer frame, once the window handle and DPI are available.
#[cfg(windows)]
pub(super) fn fit_work_area(window: &slint::Window) {
    use crate::window_geometry::{self, Rect};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::ffi::c_void;
    #[repr(C)]
    struct MonitorInfo {
        size: u32,
        monitor: Rect,
        work: Rect,
        flags: u32,
    }
    #[link(name = "user32")]
    extern "system" {
        fn MonitorFromWindow(window: *mut c_void, flags: u32) -> *mut c_void;
        fn GetMonitorInfoW(monitor: *mut c_void, info: *mut MonitorInfo) -> i32;
        fn GetWindowRect(window: *mut c_void, rect: *mut Rect) -> i32;
        fn GetClientRect(window: *mut c_void, rect: *mut Rect) -> i32;
        fn IsZoomed(window: *mut c_void) -> i32;
    }
    let provider = window.window_handle();
    let Ok(handle) = provider.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let hwnd = handle.hwnd.get() as *mut c_void;
    let mut info = MonitorInfo {
        size: std::mem::size_of::<MonitorInfo>() as u32,
        monitor: Rect::default(),
        work: Rect::default(),
        flags: 0,
    };
    let (mut outer, mut client) = (Rect::default(), Rect::default());
    // SAFETY: handle originates from this Slint window; all output structs have Win32 layout.
    let ok = unsafe {
        if IsZoomed(hwnd) != 0 {
            return;
        }
        let monitor = MonitorFromWindow(hwnd, 2);
        GetMonitorInfoW(monitor, &mut info) != 0
            && GetWindowRect(hwnd, &mut outer) != 0
            && GetClientRect(hwnd, &mut client) != 0
    };
    if !ok {
        return;
    }
    let frame = (
        (outer.right - outer.left - (client.right - client.left)).max(0) as u32,
        (outer.bottom - outer.top - (client.bottom - client.top)).max(0) as u32,
    );
    let client_size = (client.right - client.left, client.bottom - client.top);
    if client_size.0 <= 0 || client_size.1 <= 0 {
        return;
    }
    let placement = window_geometry::fit(
        info.work,
        (client_size.0 as u32, client_size.1 as u32),
        frame,
        window.scale_factor(),
    );
    window.set_size(slint::PhysicalSize::new(placement.width, placement.height));
    window.set_position(slint::PhysicalPosition::new(placement.x, placement.y));
}
