//! Physical-pixel placement, including native decorations and the taskbar work area.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[derive(Debug, PartialEq)]
pub(super) struct Placement {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
pub(super) fn fit(work: Rect, desired: (u32, u32), frame: (u32, u32), scale: f32) -> Placement {
    let scale = if scale.is_finite() {
        scale.clamp(0.5, 8.0)
    } else {
        1.0
    };
    let work_w = i64::from(work.right) - i64::from(work.left);
    let work_h = i64::from(work.bottom) - i64::from(work.top);
    let margin = (16.0 * scale).round() as i64;
    let width = i64::from(desired.0).min((work_w - i64::from(frame.0) - 2 * margin).max(1)) as u32;
    let height = i64::from(desired.1).min((work_h - i64::from(frame.1) - 2 * margin).max(1)) as u32;
    Placement {
        x: (i64::from(work.left) + (work_w - i64::from(width) - i64::from(frame.0)) / 2) as i32,
        y: (i64::from(work.top) + (work_h - i64::from(height) - i64::from(frame.1)) / 2) as i32,
        width,
        height,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn inside(work: Rect, p: &Placement, frame: (u32, u32)) {
        assert!(p.x >= work.left && p.y >= work.top, "{p:?}");
        assert!(
            i64::from(p.x) + i64::from(p.width) + i64::from(frame.0) <= i64::from(work.right),
            "{p:?}"
        );
        assert!(
            i64::from(p.y) + i64::from(p.height) + i64::from(frame.1) <= i64::from(work.bottom),
            "{p:?}"
        );
    }
    #[test]
    fn clipped_125_percent_startup_stays_above_taskbar() {
        let work = Rect {
            left: 0,
            top: 0,
            right: 1600,
            bottom: 850,
        };
        let p = fit(work, (1475, 875), (16, 48), 1.25);
        assert_eq!(p.height, 762);
        inside(work, &p, (16, 48));
    }
    #[test]
    fn common_scales_and_negative_monitor_coordinates() {
        for scale in [1.0, 1.25, 1.5, 1.75, 2.0] {
            for work in [
                Rect {
                    left: 0,
                    top: 0,
                    right: 1920,
                    bottom: 1032,
                },
                Rect {
                    left: -1600,
                    top: 40,
                    right: 0,
                    bottom: 900,
                },
            ] {
                let frame = ((16.0 * scale) as u32, (40.0 * scale) as u32);
                let p = fit(
                    work,
                    ((1180.0 * scale) as u32, (700.0 * scale) as u32),
                    frame,
                    scale,
                );
                inside(work, &p, frame);
            }
        }
    }
    #[test]
    fn does_not_enlarge_an_already_small_window() {
        let p = fit(
            Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1040,
            },
            (1000, 600),
            (16, 48),
            1.25,
        );
        assert_eq!((p.width, p.height), (1000, 600));
    }
}
