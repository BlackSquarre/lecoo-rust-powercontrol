//! Native text and dropdown checks in a separate test process; no hardware access.
use super::*;
use lecoo_control_center::localization::{self, Language, Text};
use windows::Win32::{
    Foundation::SIZE,
    Graphics::Gdi::{GetDC, GetTextExtentPoint32W, ReleaseDC},
};

fn check_text_fits(ui: &NativeWindow, language: Language) {
    let state = ui.data.borrow();
    for control in &state.controls {
        if control.text.is_empty() || control.id == SETTINGS || control.kind == Kind::Document {
            continue;
        }
        unsafe {
            let mut rect = RECT::default();
            GetClientRect(control.hwnd, &mut rect).unwrap();
            let dc = GetDC(control.hwnd);
            let font = SendMessageW(control.hwnd, WM_GETFONT, WPARAM(0), LPARAM(0));
            let previous = SelectObject(dc, HGDIOBJ(font.0 as *mut _));
            let mut size = SIZE::default();
            let text: Vec<u16> = control.text.encode_utf16().collect();
            let measured = GetTextExtentPoint32W(dc, &text, &mut size).as_bool();
            SelectObject(dc, previous);
            ReleaseDC(control.hwnd, dc);
            assert!(measured);
            let inset = if matches!(control.kind, Kind::Radio | Kind::Check) {
                29 * state.dpi / 96
            } else {
                0
            };
            assert!(
                size.cx <= rect.right - inset,
                "{} control {} clips {:?}: {}px > {}px",
                language.value(),
                control.id,
                control.text,
                size.cx,
                rect.right - inset
            );
        }
    }
}

#[test]
#[ignore = "requires an interactive Windows desktop; no hardware or preference changes"]
fn native_language_switching_and_text_fit() {
    let _controller = ControllerWindow::new().unwrap();
    localization::set_language(Language::English);
    let main = NativeWindow::new(50).unwrap();
    unsafe {
        let _ = ShowWindow(main.hwnd, SW_HIDE);
    }
    let settings = NativeWindow::settings_dialog(main.hwnd).unwrap();
    unsafe {
        let _ = ShowWindow(settings.hwnd, SW_HIDE);
    }
    let about = NativeWindow::about_dialog(settings.hwnd, false).unwrap();
    unsafe {
        let _ = ShowWindow(about.hwnd, SW_HIDE);
    }
    let close = NativeWindow::close_dialog(main.hwnd).unwrap();
    unsafe {
        let _ = ShowWindow(close.hwnd, SW_HIDE);
    }

    for language in Language::ALL.into_iter().skip(1) {
        localization::set_language(language);
        for ui in [&main, &settings, &about, &close] {
            ui.retranslate();
            check_text_fits(ui, language);
        }
        settings.select(LANGUAGE, language.index());
        assert_eq!(settings.combo_index(LANGUAGE), language.index());
        unsafe {
            let combo = settings.handle(LANGUAGE);
            assert_eq!(SendMessageW(combo, CB_GETCOUNT, WPARAM(0), LPARAM(0)).0, 9);
            for (index, expected) in language_options().into_iter().enumerate() {
                let length = SendMessageW(combo, CB_GETLBTEXTLEN, WPARAM(index), LPARAM(0)).0;
                assert!(length >= 0);
                let mut buffer = vec![0u16; length as usize + 1];
                SendMessageW(
                    combo,
                    CB_GETLBTEXT,
                    WPARAM(index),
                    LPARAM(buffer.as_mut_ptr() as isize),
                );
                assert_eq!(
                    String::from_utf16(&buffer[..length as usize]).unwrap(),
                    expected
                );
            }
        }
        assert_eq!(
            settings
                .data
                .borrow()
                .controls
                .iter()
                .find(|control| control.id == 351)
                .unwrap()
                .text,
            tr(Text::SettingsLanguageLabel)
        );
        println!(
            "{}: dashboard, settings, About, close prompt and nine language options passed",
            language.value()
        );
    }
}
