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
// Use the nearest monitor's usable rectangle, excluding the taskbar.
#[cfg(windows)]
pub(super) fn fit_work_area(window: &slint::Window) {
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }
    #[repr(C)]
    struct MonitorInfo {
        size: u32,
        monitor: Rect,
        work: Rect,
        flags: u32,
    }
    #[link(name = "user32")]
    extern "system" {
        fn MonitorFromPoint(point: Point, flags: u32) -> *mut std::ffi::c_void;
        fn GetMonitorInfoW(monitor: *mut std::ffi::c_void, info: *mut MonitorInfo) -> i32;
    }
    let position = window.position();
    let mut info = MonitorInfo {
        size: std::mem::size_of::<MonitorInfo>() as u32,
        monitor: Rect::default(),
        work: Rect::default(),
        flags: 0,
    };
    // SAFETY: Windows receives a correctly sized writable MONITORINFO structure.
    let ok = unsafe {
        let monitor = MonitorFromPoint(
            Point {
                x: position.x,
                y: position.y,
            },
            2,
        );
        GetMonitorInfoW(monitor, &mut info)
    };
    if ok == 0 {
        return;
    }
    let scale = window.scale_factor().max(1.0);
    let margin = (16.0 * scale) as i32;
    // Reserve space for native title bar and resize borders, also at higher DPI.
    let frame = (48.0 * scale) as i32;
    let width = (info.work.right - info.work.left - 2 * margin).max(1) as u32;
    let height = (info.work.bottom - info.work.top - frame - 2 * margin).max(1) as u32;
    let size = window.size();
    let width = size.width.min(width);
    let height = size.height.min(height);
    window.set_size(slint::PhysicalSize::new(width, height));
    window.set_position(slint::PhysicalPosition::new(
        info.work.left + (info.work.right - info.work.left - width as i32) / 2,
        info.work.top + margin + frame,
    ));
}
