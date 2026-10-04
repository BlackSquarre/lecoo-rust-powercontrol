//! Interactive native-icon regression check. No hardware calls or preference writes.
use super::*;
use crate::platform::windows::desktop::Desktop;
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetGuiResources, GR_GDIOBJECTS, GR_USEROBJECTS,
};

fn resources() -> (u32, u32) {
    unsafe {
        (
            GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS),
            GetGuiResources(GetCurrentProcess(), GR_USEROBJECTS),
        )
    }
}

fn verify_pixels(icon: HICON, size: i32, color: [u8; 3]) {
    unsafe {
        let mut info = ICONINFO::default();
        GetIconInfo(icon, &mut info).unwrap();
        let mut bitmap = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let dc = CreateCompatibleDC(None);
        let mut pixels = vec![0u8; size as usize * size as usize * 4];
        let lines = GetDIBits(
            dc,
            info.hbmColor,
            0,
            size as u32,
            Some(pixels.as_mut_ptr().cast()),
            &mut bitmap,
            DIB_RGB_COLORS,
        );
        let _ = DeleteDC(dc);
        let _ = DeleteObject(info.hbmColor);
        let _ = DeleteObject(info.hbmMask);
        assert_eq!(lines, size);
        assert_eq!(bitmap.bmiHeader.biWidth, size);
        let index = ((size / 4) * size + size / 2) as usize * 4;
        // Windows can resample a nearby ICO frame at fractional DPI, rounding a
        // solid channel by one level. Still reject a wrong mode or translucent stem.
        let expected = [color[2], color[1], color[0], 255];
        for (actual, expected) in pixels[index..index + 4].iter().zip(expected) {
            assert!(
                actual.abs_diff(expected) <= 2,
                "wrong icon color at {size}px: {:?}, expected {expected}",
                &pixels[index..index + 4]
            );
        }
    }
}

fn verify_window(ui: &NativeWindow, mode: Option<PowerMode>, color: [u8; 3]) {
    ui.icon_mode(mode).unwrap();
    let icons = ui.icons.borrow();
    let icons = icons.as_ref().unwrap();
    let small_size = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, icons.dpi) }.max(16);
    let big_size = unsafe { GetSystemMetricsForDpi(SM_CXICON, icons.dpi) }.max(32);
    verify_pixels(icons.small.0, small_size, color);
    verify_pixels(icons.big.0, big_size, color);
    for (kind, expected) in [(ICON_SMALL, icons.small.0), (ICON_BIG, icons.big.0)] {
        let actual = unsafe { SendMessageW(ui.hwnd, WM_GETICON, WPARAM(kind as usize), LPARAM(0)) };
        assert_eq!(actual.0, expected.0 as isize);
    }
    if let Some(icon) = &icons.about {
        verify_pixels(icon.0, (64 * icons.dpi / 96) as i32, color);
        assert_eq!(ui.data.borrow().about_icon, icon.0);
    }
    let handles = (icons.small.0, icons.big.0);
    // Every unchanged refresh must retain exactly the same native icon handles.
    for _ in 0..100 {
        ui.icon_mode(mode).unwrap();
        let current = ui.icons.borrow();
        let current = current.as_ref().unwrap();
        assert_eq!((current.small.0, current.big.0), handles);
    }
}

#[test]
#[ignore = "requires an interactive Windows desktop; simulates icon states without changing hardware"]
fn native_mode_icons_and_resource_lifetime() {
    let _controller = ControllerWindow::new().unwrap();
    let desktop = Desktop::new().unwrap();
    // Warm up native common controls and fonts before recording leak baselines.
    drop(NativeWindow::new(50).unwrap());
    let mut baseline = resources();
    for cycle in 0..12 {
        let main = NativeWindow::new(50).unwrap();
        let settings = NativeWindow::settings_dialog(main.hwnd).unwrap();
        let about = NativeWindow::about_dialog(settings.hwnd, false).unwrap();
        let notices = NativeWindow::about_dialog(about.hwnd, true).unwrap();
        let close = NativeWindow::close_dialog(main.hwnd).unwrap();
        // Hide the probe's windows while exercising the production update code.
        for ui in [&main, &settings, &about, &notices, &close] {
            unsafe {
                let _ = ShowWindow(ui.hwnd, SW_HIDE);
            }
        }
        for (mode, color) in [
            (Some(PowerMode::Quiet), [0, 210, 166]),
            (Some(PowerMode::Balance), [98, 167, 255]),
            (Some(PowerMode::Performance), [255, 115, 122]),
            (None, [160, 166, 176]),
        ] {
            desktop.refresh(mode);
            assert!(desktop.is_available());
            for ui in [&main, &settings, &about, &notices, &close] {
                verify_window(ui, mode, color);
            }
            let stable = resources();
            for _ in 0..1000 {
                desktop.refresh(mode);
            }
            assert_eq!(
                resources(),
                stable,
                "unchanged tray refresh allocates resources"
            );
        }
        drop(close);
        drop(notices);
        drop(about);
        drop(settings);
        drop(main);
        let after = resources();
        if cycle == 0 {
            baseline = after;
        }
        println!("Cycle {}: GDI={} USER={}", cycle + 1, after.0, after.1);
        assert!(
            after.0 <= baseline.0 + 2 && after.1 <= baseline.1 + 2,
            "native resources leaked: baseline={baseline:?}, after={after:?}"
        );
    }
    for dpi in [96, 120, 144, 192] {
        let icons = WindowIcons::load(ModeIcon::Performance, dpi, true).unwrap();
        verify_pixels(
            icons.small.0,
            unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) }.max(16),
            [255, 115, 122],
        );
        verify_pixels(
            icons.big.0,
            unsafe { GetSystemMetricsForDpi(SM_CXICON, dpi) }.max(32),
            [255, 115, 122],
        );
        verify_pixels(
            icons.about.as_ref().unwrap().0,
            (64 * dpi / 96) as i32,
            [255, 115, 122],
        );
    }
    println!("Four mode colors, all owned windows, stable refresh handles, 12 release cycles and 100/125/150/200% DPI passed; no hardware writes.");
}

#[test]
#[ignore = "requires an interactive Windows desktop; no hardware or preference changes"]
fn events_wake_idle_controller_without_waiting_for_timer() {
    use crate::platform::windows::wake::{UiWake, WAKE};
    use std::time::{Duration, Instant};
    use windows::Win32::{
        Foundation::CloseHandle,
        System::Threading::{CreateEventW, SetEvent},
    };
    let controller = ControllerWindow::new().unwrap();
    controller.set_interval(5000).unwrap();
    let event = unsafe { CreateEventW(None, false, false, None) }.unwrap();
    unsafe {
        let mut message = MSG::default();
        while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
            DispatchMessageW(&message);
        }
    }
    let wake = UiWake::new(controller.0);
    let sender = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        wake.notify();
    });
    let at = Instant::now();
    assert!(!wait_for_activity(event).unwrap());
    sender.join().unwrap();
    let posted_ms = at.elapsed().as_millis();
    assert!(posted_ms < 2000, "posted result waited for idle timer");
    unsafe {
        let mut message = MSG::default();
        assert!(PeekMessageW(&mut message, None, WAKE, WAKE, PM_REMOVE).as_bool());
        assert_eq!(message.hwnd, controller.0);
    }
    let event_value = event.0 as usize;
    let sender = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        unsafe {
            SetEvent(HANDLE(event_value as *mut _)).unwrap();
        }
    });
    let at = Instant::now();
    assert!(wait_for_activity(event).unwrap());
    sender.join().unwrap();
    let event_ms = at.elapsed().as_millis();
    unsafe {
        CloseHandle(event).unwrap();
    }
    assert!(event_ms < 2000, "second launch waited for idle timer");
    println!("Idle wake latency: posted={posted_ms}ms, event={event_ms}ms; timer=5000ms");
}
