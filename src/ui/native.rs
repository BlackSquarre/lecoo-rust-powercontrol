//! System HWND controls, with small GDI paint handlers for consistent light/dark colors.
//! No graphics context, bitmap back buffer, bundled fonts or continuous animation.
use super::native_theme::{system_dark, Brush, Palette};
use anyhow::{bail, Result};
use lecoo_control_center::localization::text as tr;
use std::cell::{Cell, RefCell};
use windows::{
    core::{w, HSTRING},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::{
            Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
            Gdi::*,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::*,
            HiDpi::*,
            Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled, SetFocus},
            WindowsAndMessaging::*,
        },
    },
};

pub const COMMAND: u32 = WM_APP + 1;
pub const CLOSE: u32 = WM_APP + 2;
pub const HIDE: u32 = WM_APP + 3;
pub const LAYOUT: u32 = WM_APP + 4;
pub const SLIDER: u32 = WM_APP + 5;
pub const EXIT: u32 = WM_APP + 6;
pub const DPI: u32 = WM_APP + 7;
pub const THEME: u32 = WM_APP + 8;
pub const QUIET: u16 = 110;
pub const BALANCE: u16 = 111;
pub const PERFORMANCE: u16 = 112;
pub const AUTO: u16 = 120;
pub const MAXIMUM: u16 = 121;
pub const TARGET: u16 = 122;
pub const MANUAL: u16 = 123;
pub const RECONNECT: u16 = 124;
pub const STARTUP: u16 = 130;
pub const RESET_CLOSE: u16 = 132;
pub const SETTINGS: u16 = 140;
pub const LANGUAGE: u16 = 141;
pub const CLOSE_BEHAVIOR: u16 = 142;
pub const ABOUT: u16 = 143;
pub const BILIBILI: u16 = 144;
pub const PROJECT: u16 = 145;
pub const NOTICES: u16 = 146;
pub const THERMAL_ZONE: u16 = 200;
pub const RPM: u16 = 201;
pub const MEMORY: u16 = 202;
pub const MEMORY_BAR: u16 = 203;
pub const DISK: u16 = 204;
pub const DISK_BAR: u16 = 205;
pub const ERROR: u16 = 206;
pub const STARTUP_STATUS: u16 = 212;
pub const CURRENT_MODE: u16 = 213;
pub const PERCENT: u16 = 214;
pub const CONNECTION: u16 = 216;
pub const DIALOG_TRAY: u16 = 901;
pub const DIALOG_EXIT: u16 = 902;
pub const REMEMBER: u16 = 903;
pub const DIALOG_CANCEL: u16 = 904;
const TBM_GETPOS: u32 = WM_USER;

thread_local! {static CONTROLLER:Cell<HWND>=const{Cell::new(HWND(std::ptr::null_mut()))};}
fn post(message: u32, wp: WPARAM, lp: LPARAM) {
    CONTROLLER.with(|slot| unsafe {
        let _ = PostMessageW(slot.get(), message, wp, lp);
    });
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Label,
    Heading,
    Number,
    Radio,
    Check,
    Button,
    Slider,
    Progress,
    Muted,
    Small,
    Combo,
    Center,
    CenterHeading,
    Link,
    Document,
}
struct Control {
    id: u16,
    hwnd: HWND,
    kind: Kind,
    text: String,
    checked: bool,
    progress: u8,
}
struct Font(HFONT);
impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.0 .0));
        }
    }
}
struct WindowData {
    palette: Palette,
    controls: Vec<Control>,
    cards: Vec<RECT>,
    fonts: Vec<Font>,
    dpi: i32,
    dialog: bool,
    settings: bool,
    about: bool,
    notices: bool,
    mode: Option<u8>,
}
pub struct NativeWindow {
    pub hwnd: HWND,
    data: Box<RefCell<WindowData>>,
    icon: Option<HICON>,
}

unsafe fn data<'a>(hwnd: HWND) -> Option<&'a RefCell<WindowData>> {
    (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<WindowData>).as_ref()
}
unsafe fn fill(dc: HDC, rect: &RECT, color: COLORREF) {
    let brush = Brush::new(color);
    let _ = FillRect(dc, rect, brush.0);
}
unsafe fn rounded(dc: HDC, rect: RECT, background: COLORREF, border: COLORREF, radius: i32) {
    let brush = Brush::new(background);
    let pen = CreatePen(PS_SOLID, 1, border);
    let old_brush = SelectObject(dc, HGDIOBJ(brush.0 .0));
    let old_pen = SelectObject(dc, HGDIOBJ(pen.0));
    let _ = RoundRect(
        dc,
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
        radius,
        radius,
    );
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(HGDIOBJ(pen.0));
}
unsafe fn draw_text(dc: HDC, text: &str, mut rect: RECT, color: COLORREF, flags: DRAW_TEXT_FORMAT) {
    let _ = SetTextColor(dc, color);
    let _ = SetBkMode(dc, TRANSPARENT);
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    let _ = DrawTextW(dc, &mut wide, &mut rect, flags);
}
fn mode_color(p: &Palette, id: u16) -> COLORREF {
    match id {
        QUIET => p.green,
        PERFORMANCE => p.red,
        _ => p.accent,
    }
}
unsafe fn draw_button(state: &WindowData, item: &DRAWITEMSTRUCT) {
    let Some(control) = state.controls.iter().find(|c| c.id == item.CtlID as u16) else {
        return;
    };
    let p = &state.palette;
    let dc = item.hDC;
    let rect = item.rcItem;
    let disabled = item.itemState.0 & ODS_DISABLED.0 != 0;
    let focused = item.itemState.0 & ODS_FOCUS.0 != 0;
    let pressed = item.itemState.0 & ODS_SELECTED.0 != 0;
    let scale = |v: i32| v * state.dpi / 96;
    let background = if state.settings && control.id == ABOUT {
        p.background_color
    } else {
        p.card
    };
    fill(dc, &rect, background);
    let _ = SelectObject(
        dc,
        HGDIOBJ(state.fonts[if control.id == SETTINGS { 4 } else { 0 }].0 .0),
    );
    let mut label = rect;
    let color = if disabled {
        p.muted
    } else if [QUIET, BALANCE, PERFORMANCE].contains(&control.id) {
        mode_color(p, control.id)
    } else if control.kind == Kind::Link {
        p.accent
    } else {
        p.text
    };
    match control.kind {
        Kind::Radio | Kind::Check => {
            let edge = scale(17);
            let top = (rect.bottom - edge) / 2;
            let indicator = RECT {
                left: scale(2),
                top,
                right: scale(2) + edge,
                bottom: top + edge,
            };
            let mark = if disabled {
                p.muted
            } else if [QUIET, BALANCE, PERFORMANCE].contains(&control.id) {
                color
            } else {
                p.accent
            };
            rounded(
                dc,
                indicator,
                p.card,
                if control.checked { mark } else { p.muted },
                if control.kind == Kind::Radio {
                    edge
                } else {
                    scale(4)
                },
            );
            if control.checked {
                if control.kind == Kind::Radio {
                    let dot = RECT {
                        left: indicator.left + scale(4),
                        top: indicator.top + scale(4),
                        right: indicator.right - scale(4),
                        bottom: indicator.bottom - scale(4),
                    };
                    rounded(dc, dot, mark, mark, edge);
                } else {
                    draw_text(
                        dc,
                        "✓",
                        indicator,
                        mark,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                }
            }
            label.left += scale(29);
            draw_text(
                dc,
                &control.text,
                label,
                color,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
        }
        _ => {
            if control.kind != Kind::Link {
                rounded(
                    dc,
                    rect,
                    if pressed { p.pressed } else { p.button },
                    if focused { p.accent } else { p.border },
                    scale(7),
                );
            }
            let alignment = if control.id == RESET_CLOSE {
                label.left += scale(4);
                DT_LEFT | DT_VCENTER | DT_SINGLELINE
            } else {
                DT_CENTER | DT_VCENTER | DT_SINGLELINE
            };
            draw_text(
                dc,
                if control.id == SETTINGS {
                    "⚙"
                } else {
                    &control.text
                },
                label,
                color,
                alignment,
            );
        }
    }
    if focused && control.kind != Kind::Button {
        let _ = DrawFocusRect(dc, &rect);
    }
}

unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(lp.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    match msg {
        WM_CLOSE => {
            post(CLOSE, WPARAM(hwnd.0 as usize), LPARAM(0));
            return LRESULT(0);
        }
        WM_COMMAND => {
            let notification = (wp.0 >> 16) & 0xffff;
            // Ignore combo focus/dropdown notifications; repopulating an open
            // dropdown on CBN_DROPDOWN would prevent the user selecting a language.
            if notification == 0 || notification == CBN_SELCHANGE as usize {
                post(COMMAND, wp, lp);
            }
            return LRESULT(0);
        }
        WM_HSCROLL => {
            post(SLIDER, wp, lp);
            return LRESULT(0);
        }
        WM_SIZE => {
            let is_dialog = data(hwnd)
                .and_then(|s| s.try_borrow().ok().map(|s| s.dialog))
                .unwrap_or(false);
            post(
                if wp.0 == SIZE_MINIMIZED as usize && !is_dialog {
                    HIDE
                } else {
                    LAYOUT
                },
                WPARAM(hwnd.0 as usize),
                LPARAM(0),
            );
            return LRESULT(0);
        }
        WM_DPICHANGED => {
            let rect = &*(lp.0 as *const RECT);
            let _ = SetWindowPos(
                hwnd,
                None,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            post(DPI, WPARAM(hwnd.0 as usize), LPARAM(0));
            return LRESULT(0);
        }
        WM_SETTINGCHANGE | WM_THEMECHANGED => {
            post(THEME, WPARAM(0), LPARAM(0));
        }
        WM_ENDSESSION if wp.0 != 0 => {
            post(EXIT, wp, lp);
            return LRESULT(0);
        }
        WM_GETMINMAXINFO => {
            if data(hwnd)
                .and_then(|s| s.try_borrow().ok().map(|s| s.dialog))
                .unwrap_or(false)
            {
                return DefWindowProcW(hwnd, msg, wp, lp);
            }
            let dpi = GetDpiForWindow(hwnd).max(96);
            let info = &mut *(lp.0 as *mut MINMAXINFO);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 400 * dpi as i32 / 96,
                bottom: 520 * dpi as i32 / 96,
            };
            let _ = AdjustWindowRectExForDpi(
                &mut rect,
                WS_OVERLAPPEDWINDOW,
                false,
                WS_EX_CONTROLPARENT,
                dpi,
            );
            info.ptMinTrackSize.x = rect.right - rect.left;
            info.ptMinTrackSize.y = rect.bottom - rect.top;
            return LRESULT(0);
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        }
        _ => {}
    }
    if let Some(cell) = data(hwnd) {
        if let Ok(state) = cell.try_borrow() {
            let p = &state.palette;
            match msg {
                WM_ERASEBKGND => {
                    return LRESULT(1);
                }
                WM_PAINT => {
                    let mut paint = PAINTSTRUCT::default();
                    let dc = BeginPaint(hwnd, &mut paint);
                    let mut rect = RECT::default();
                    let _ = GetClientRect(hwnd, &mut rect);
                    let _ = FillRect(dc, &rect, p.background.0);
                    for card in &state.cards {
                        rounded(dc, *card, p.card, p.border, 10 * state.dpi / 96);
                    }
                    if state.about {
                        let icon = HICON(
                            SendMessageW(hwnd, WM_GETICON, WPARAM(ICON_BIG as usize), LPARAM(0)).0
                                as *mut _,
                        );
                        let size = 64 * state.dpi / 96;
                        let _ = DrawIconEx(
                            dc,
                            (rect.right - size) / 2,
                            20 * state.dpi / 96,
                            icon,
                            size,
                            size,
                            0,
                            None,
                            DI_NORMAL,
                        );
                    }
                    let _ = EndPaint(hwnd, &paint);
                    return LRESULT(0);
                }
                WM_CTLCOLORSTATIC | WM_CTLCOLORBTN | WM_CTLCOLORLISTBOX | WM_CTLCOLOREDIT => {
                    let dc = HDC(wp.0 as *mut _);
                    let id = GetDlgCtrlID(HWND(lp.0 as *mut _)) as u16;
                    let kind = state.controls.iter().find(|c| c.id == id).map(|c| c.kind);
                    let color = if id == CURRENT_MODE {
                        state
                            .mode
                            .map(|m| {
                                mode_color(
                                    p,
                                    match m {
                                        2 => QUIET,
                                        1 => PERFORMANCE,
                                        _ => BALANCE,
                                    },
                                )
                            })
                            .unwrap_or(p.muted)
                    } else if id == ERROR {
                        p.red
                    } else if id == CONNECTION {
                        p.green
                    } else if kind == Some(Kind::Muted) || kind == Some(Kind::Center) {
                        p.muted
                    } else {
                        p.text
                    };
                    let _ = SetTextColor(dc, color);
                    let footer = (!state.dialog && [331, 332, ERROR].contains(&id))
                        || (state.settings && [354, ERROR].contains(&id));
                    let _ = SetBkColor(dc, if footer { p.background_color } else { p.card });
                    let _ = SetBkMode(dc, TRANSPARENT);
                    return LRESULT(if footer {
                        p.background.0 .0 as isize
                    } else {
                        p.surface.0 .0 as isize
                    });
                }
                WM_DRAWITEM => {
                    draw_button(&state, &*(lp.0 as *const DRAWITEMSTRUCT));
                    return LRESULT(1);
                }
                WM_NOTIFY => {
                    let header = &*(lp.0 as *const NMHDR);
                    if header.idFrom == TARGET as usize && header.code == NM_CUSTOMDRAW {
                        let draw = &*(lp.0 as *const NMCUSTOMDRAW);
                        if draw.dwDrawStage == CDDS_PREPAINT {
                            return LRESULT(CDRF_NOTIFYPOSTPAINT as isize);
                        }
                        if draw.dwDrawStage == CDDS_POSTPAINT {
                            // Draw after the control's cached background; query its actual native
                            // channel/thumb geometry so mouse hit testing and keyboard behavior stay native.
                            let mut bounds = RECT::default();
                            let mut channel = RECT::default();
                            let mut thumb = RECT::default();
                            let _ = GetClientRect(header.hwndFrom, &mut bounds);
                            SendMessageW(
                                header.hwndFrom,
                                TBM_GETCHANNELRECT,
                                WPARAM(0),
                                LPARAM((&mut channel as *mut RECT) as isize),
                            );
                            SendMessageW(
                                header.hwndFrom,
                                TBM_GETTHUMBRECT,
                                WPARAM(0),
                                LPARAM((&mut thumb as *mut RECT) as isize),
                            );
                            fill(draw.hdc, &bounds, p.card);
                            let middle = (thumb.top + thumb.bottom) / 2;
                            channel.top = middle - 3 * state.dpi / 96;
                            channel.bottom = middle + 3 * state.dpi / 96;
                            rounded(draw.hdc, channel, p.track, p.track, 6 * state.dpi / 96);
                            channel.right = (thumb.left + thumb.right) / 2;
                            if channel.right > channel.left {
                                rounded(draw.hdc, channel, p.accent, p.accent, 6 * state.dpi / 96);
                            }
                            rounded(draw.hdc, thumb, p.button, p.accent, 5 * state.dpi / 96);
                            return LRESULT(CDRF_DODEFAULT as isize);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}
unsafe extern "system" fn controller_procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    if msg == WM_ENDSESSION && wp.0 != 0 {
        post(EXIT, wp, lp);
    }
    if msg == WM_SETTINGCHANGE || msg == WM_THEMECHANGED {
        post(THEME, wp, LPARAM(0));
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}
pub struct ControllerWindow(pub HWND);
impl ControllerWindow {
    pub fn new() -> Result<Self> {
        unsafe {
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            for (name, proc) in [
                (
                    w!("LecooNativeController"),
                    controller_procedure as unsafe extern "system" fn(_, _, _, _) -> _,
                ),
                (
                    w!("LecooNativeWindow"),
                    procedure as unsafe extern "system" fn(_, _, _, _) -> _,
                ),
            ] {
                let class = WNDCLASSW {
                    lpfnWndProc: Some(proc),
                    hInstance: instance,
                    lpszClassName: name,
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    ..Default::default()
                };
                if RegisterClassW(&class) == 0 {
                    return Err(windows::core::Error::from_win32().into());
                }
            }
            if !InitCommonControlsEx(&INITCOMMONCONTROLSEX {
                dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_BAR_CLASSES | ICC_PROGRESS_CLASS,
            })
            .as_bool()
            {
                bail!("无法初始化 Windows 控件");
            }
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("LecooNativeController"),
                w!(""),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                None,
                None,
                instance,
                None,
            )?;
            let result = Self(hwnd);
            CONTROLLER.with(|slot| slot.set(hwnd));
            if SetTimer(hwnd, 1, 250, None) == 0 {
                bail!("无法创建后台定时器");
            }
            Ok(result)
        }
    }
}
impl Drop for ControllerWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = KillTimer(self.0, 1);
            let _ = DestroyWindow(self.0);
            CONTROLLER.with(|slot| slot.set(HWND::default()));
        }
    }
}

impl NativeWindow {
    fn create(owner: Option<HWND>, dialog: bool) -> Result<Self> {
        unsafe {
            let state = Box::new(RefCell::new(WindowData {
                palette: Palette::new(system_dark()),
                controls: Vec::new(),
                cards: Vec::new(),
                fonts: Vec::new(),
                dpi: 96,
                dialog,
                settings: false,
                about: false,
                notices: false,
                mode: None,
            }));
            let dpi = GetDpiForSystem().max(96);
            let (width, height) = if dialog { (440, 195) } else { (400, 520) };
            let style = if dialog {
                WS_POPUP | WS_CAPTION | WS_SYSMENU
            } else {
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN
            };
            let ex = if dialog {
                WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME
            } else {
                WS_EX_CONTROLPARENT
            };
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: width * dpi as i32 / 96,
                bottom: height * dpi as i32 / 96,
            };
            AdjustWindowRectExForDpi(&mut rect, style, false, ex, dpi)?;
            let hwnd = CreateWindowExW(
                ex,
                w!("LecooNativeWindow"),
                &HSTRING::from(if dialog {
                    tr("关闭窗口", "Close window").to_owned()
                } else {
                    "Lecoo Rust PowerControl".to_owned()
                }),
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                owner.unwrap_or_default(),
                None,
                HINSTANCE(GetModuleHandleW(None)?.0),
                Some((&*state as *const RefCell<WindowData>).cast()),
            )?;
            let mut ui = Self {
                hwnd,
                data: state,
                icon: None,
            };
            ui.update_font()?;
            let mut pixels = crate::platform::windows::desktop::icon_rgba();
            let mut mask = [0u8; 128];
            for (i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
                pixel.swap(0, 2);
                if pixel[3] == 0 {
                    mask[i / 8] |= 1 << (7 - i % 8);
                }
            }
            let owned = CreateIcon(
                HINSTANCE(GetModuleHandleW(None)?.0),
                32,
                32,
                1,
                32,
                mask.as_ptr(),
                pixels.as_ptr(),
            )
            .ok();
            let icon = if let Some(icon) = owned {
                ui.icon = Some(icon);
                icon
            } else {
                LoadIconW(None, IDI_APPLICATION)?
            };
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_SMALL as usize),
                LPARAM(icon.0 as isize),
            );
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_BIG as usize),
                LPARAM(icon.0 as isize),
            );
            Ok(ui)
        }
    }
    pub fn new(percent: u8) -> Result<Self> {
        let ui = Self::create(None, false)?;
        for (id, text, kind) in [
            (300, tr("ACPI 热区温度", "ACPI thermal zone"), Kind::Muted),
            (THERMAL_ZONE, "—", Kind::Number),
            (302, tr("风扇转速", "Fan speed"), Kind::Muted),
            (RPM, "—", Kind::Number),
            (303, tr("内存", "Memory"), Kind::Small),
            (MEMORY, "—", Kind::Small),
            (304, tr("磁盘", "Disk"), Kind::Small),
            (DISK, "—", Kind::Small),
            (310, tr("电源模式", "Power mode"), Kind::Heading),
            (QUIET, tr("安静", "Quiet"), Kind::Radio),
            (BALANCE, tr("均衡", "Balanced"), Kind::Radio),
            (PERFORMANCE, tr("性能", "Performance"), Kind::Radio),
            (311, tr("风扇控制", "Fan control"), Kind::Heading),
            (AUTO, tr("自动", "Automatic"), Kind::Radio),
            (MANUAL, tr("手动", "Manual"), Kind::Radio),
            (MAXIMUM, tr("最大", "Maximum"), Kind::Radio),
            (PERCENT, "50%", Kind::Number),
            (331, "Lecoo MINI PRO-AHP", Kind::Small),
            (332, "AMD Ryzen 7 8745H", Kind::Small),
            (SETTINGS, tr("设置", "Settings"), Kind::Button),
            (ERROR, "", Kind::Small),
        ] {
            ui.add(id, text, kind)?;
        }
        ui.add(TARGET, "", Kind::Slider)?;
        ui.add(MEMORY_BAR, "", Kind::Progress)?;
        ui.add(DISK_BAR, "", Kind::Progress)?;
        unsafe {
            let slider = ui.handle(TARGET);
            SendMessageW(slider, TBM_SETRANGEMIN, WPARAM(0), LPARAM(35));
            SendMessageW(slider, TBM_SETRANGEMAX, WPARAM(1), LPARAM(100));
            SendMessageW(slider, TBM_SETPOS, WPARAM(1), LPARAM(percent as isize));
        }
        ui.apply_theme();
        ui.layout();
        ui.show();
        Ok(ui)
    }
    pub fn settings_dialog(owner: HWND) -> Result<Self> {
        let ui = Self::create(Some(owner), true)?;
        ui.data.borrow_mut().settings = true;
        unsafe {
            SetWindowTextW(ui.hwnd, &HSTRING::from(tr("设置", "Settings")))?;
            let dpi = GetDpiForWindow(ui.hwnd).max(96);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 380 * dpi as i32 / 96,
                bottom: 440 * dpi as i32 / 96,
            };
            AdjustWindowRectExForDpi(
                &mut rect,
                WS_POPUP | WS_CAPTION | WS_SYSMENU,
                false,
                WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME,
                dpi,
            )?;
            SetWindowPos(
                ui.hwnd,
                None,
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOMOVE | SWP_NOZORDER,
            )?;
        }
        for (id, text, kind) in [
            (352, tr("启动", "Startup"), Kind::Heading),
            (
                STARTUP,
                tr(
                    "登录后启动（进入托盘）",
                    "Start at sign-in (minimized to tray)",
                ),
                Kind::Check,
            ),
            (STARTUP_STATUS, "", Kind::Muted),
            (353, tr("窗口与语言", "Window & language"), Kind::Heading),
            (350, tr("关闭窗口时", "When closing"), Kind::Label),
            (CLOSE_BEHAVIOR, "", Kind::Combo),
            (
                RESET_CLOSE,
                tr("重置关闭选择", "Reset close preference"),
                Kind::Link,
            ),
            (351, tr("语言", "Language"), Kind::Label),
            (LANGUAGE, "", Kind::Combo),
            (ERROR, "", Kind::Small),
            (354, tr("应用信息", "Application"), Kind::Muted),
            (ABOUT, tr("关于…", "About…"), Kind::Button),
        ] {
            ui.add(id, text, kind)?;
        }
        ui.combo_items(
            CLOSE_BEHAVIOR,
            &[
                tr("每次询问", "Ask every time"),
                tr("最小化到托盘", "Minimize to tray"),
                tr("退出", "Exit"),
            ],
        );
        ui.combo_items(
            LANGUAGE,
            &[tr("跟随系统", "System default"), "简体中文", "English"],
        );
        ui.apply_theme();
        ui.layout();
        ui.center_on(owner);
        ui.show();
        Ok(ui)
    }
    pub fn about_dialog(owner: HWND, notices: bool) -> Result<Self> {
        let mut ui = Self::create(Some(owner), true)?;
        ui.data.borrow_mut().about = !notices;
        ui.data.borrow_mut().notices = notices;
        unsafe {
            let dpi = GetDpiForWindow(ui.hwnd).max(96);
            let (width, height) = if notices { (680, 500) } else { (380, 345) };
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: width * dpi as i32 / 96,
                bottom: height * dpi as i32 / 96,
            };
            AdjustWindowRectExForDpi(
                &mut rect,
                WS_POPUP | WS_CAPTION | WS_SYSMENU,
                false,
                WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME,
                dpi,
            )?;
            SetWindowPos(
                ui.hwnd,
                None,
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOMOVE | SWP_NOZORDER,
            )?;
        }
        if notices {
            ui.add(
                360,
                &include_str!("../../docs/THIRD-PARTY-NOTICES.txt").replace("\n", "\r\n"),
                Kind::Document,
            )?;
            unsafe {
                SendMessageW(
                    ui.handle(360),
                    EM_SETLIMITTEXT,
                    WPARAM(1024 * 1024),
                    LPARAM(0),
                );
            }
        } else {
            ui.install_about_icon();
            for (id, text, kind) in [
                (340, "Lecoo Rust PowerControl", Kind::CenterHeading),
                (
                    341,
                    tr("来酷迷你主机控制中心", "Lecoo mini PC control center"),
                    Kind::Center,
                ),
                (342, "", Kind::Center),
                (BILIBILI, tr("哔哩哔哩", "Bilibili"), Kind::Link),
                (PROJECT, tr("项目主页", "Project website"), Kind::Link),
                (
                    NOTICES,
                    tr("第三方声明与版权", "Third-party notices"),
                    Kind::Link,
                ),
                (343, "", Kind::Center),
                (
                    344,
                    tr("保留所有权利。", "All rights reserved."),
                    Kind::Center,
                ),
            ] {
                ui.add(id, text, kind)?;
            }
            ui.text(
                342,
                format!("{} {}", tr("版本", "Version"), env!("CARGO_PKG_VERSION")),
            );
            ui.refresh_year();
        }
        ui.retranslate();
        ui.apply_theme();
        ui.layout();
        ui.center_on(owner);
        ui.show();
        Ok(ui)
    }
    pub fn refresh_year(&self) {
        if self.data.borrow().about {
            let year = unsafe { windows::Win32::System::SystemInformation::GetLocalTime().wYear };
            self.text(343, format!("© {year} 缪凌儒 BlackSquare"));
        }
    }
    fn install_about_icon(&mut self) {
        let size = 128usize;
        let mut pixels = crate::platform::windows::desktop::icon_rgba_size(size);
        let mut mask = vec![0u8; size * size / 8];
        for (i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
            pixel.swap(0, 2);
            if pixel[3] == 0 {
                mask[i / 8] |= 1 << (7 - i % 8);
            }
        }
        unsafe {
            if let Ok(icon) = CreateIcon(
                HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0),
                size as i32,
                size as i32,
                1,
                32,
                mask.as_ptr(),
                pixels.as_ptr(),
            ) {
                SendMessageW(
                    self.hwnd,
                    WM_SETICON,
                    WPARAM(ICON_BIG as usize),
                    LPARAM(icon.0 as isize),
                );
                SendMessageW(
                    self.hwnd,
                    WM_SETICON,
                    WPARAM(ICON_SMALL as usize),
                    LPARAM(icon.0 as isize),
                );
                if let Some(old) = self.icon.replace(icon) {
                    let _ = DestroyIcon(old);
                }
            }
        }
    }
    pub fn is_settings(&self) -> bool {
        self.data.borrow().settings
    }
    pub fn combo_items(&self, id: u16, items: &[&str]) {
        unsafe {
            let hwnd = self.handle(id);
            SendMessageW(hwnd, CB_RESETCONTENT, WPARAM(0), LPARAM(0));
            for item in items {
                let text = HSTRING::from(*item);
                SendMessageW(
                    hwnd,
                    CB_ADDSTRING,
                    WPARAM(0),
                    LPARAM(text.as_ptr() as isize),
                );
            }
        }
    }
    pub fn combo_index(&self, id: u16) -> usize {
        unsafe {
            SendMessageW(self.handle(id), CB_GETCURSEL, WPARAM(0), LPARAM(0))
                .0
                .max(0) as usize
        }
    }
    pub fn select(&self, id: u16, index: usize) {
        unsafe {
            SendMessageW(self.handle(id), CB_SETCURSEL, WPARAM(index), LPARAM(0));
        }
    }
    fn center_on(&self, owner: HWND) {
        unsafe {
            let mut parent = RECT::default();
            let mut child = RECT::default();
            let _ = GetWindowRect(owner, &mut parent);
            let _ = GetWindowRect(self.hwnd, &mut child);
            let _ = SetWindowPos(
                self.hwnd,
                None,
                parent.left + (parent.right - parent.left - child.right + child.left) / 2,
                parent.top + (parent.bottom - parent.top - child.bottom + child.top) / 2,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER,
            );
            let _ = EnableWindow(owner, false);
        }
    }
    pub fn close_dialog(owner: HWND) -> Result<Self> {
        let ui = Self::create(Some(owner), true)?;
        for (id, text, kind) in [
            (
                950,
                tr("关闭窗口后要执行什么操作？", "What would you like to do?"),
                Kind::Heading,
            ),
            (951, "", Kind::Label),
            (
                DIALOG_TRAY,
                tr("最小化到托盘", "Minimize to tray"),
                Kind::Button,
            ),
            (DIALOG_EXIT, tr("退出", "Exit"), Kind::Button),
            (
                REMEMBER,
                tr("记住我的选择", "Remember my choice"),
                Kind::Check,
            ),
        ] {
            ui.add(id, text, kind)?;
        }
        ui.apply_theme();
        ui.layout();
        unsafe {
            let mut parent = RECT::default();
            let mut dialog = RECT::default();
            let _ = GetWindowRect(owner, &mut parent);
            let _ = GetWindowRect(ui.hwnd, &mut dialog);
            let _ = SetWindowPos(
                ui.hwnd,
                None,
                parent.left + (parent.right - parent.left - dialog.right + dialog.left) / 2,
                parent.top + (parent.bottom - parent.top - dialog.bottom + dialog.top) / 2,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER,
            );
            let _ = EnableWindow(owner, false);
        }
        ui.show();
        unsafe {
            let _ = SetFocus(ui.handle(DIALOG_TRAY));
        }
        Ok(ui)
    }
    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(self.hwnd);
        }
    }
    fn add(&self, id: u16, text: &str, kind: Kind) -> Result<()> {
        unsafe {
            let (class, style) = match kind {
                Kind::Radio | Kind::Check | Kind::Button | Kind::Link => {
                    ("BUTTON", WINDOW_STYLE(WS_TABSTOP.0 | BS_OWNERDRAW as u32))
                }
                Kind::Combo => (
                    "COMBOBOX",
                    WINDOW_STYLE(WS_TABSTOP.0 | CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0),
                ),
                Kind::Document => (
                    "EDIT",
                    WINDOW_STYLE(
                        ES_MULTILINE as u32
                            | ES_READONLY as u32
                            | ES_AUTOVSCROLL as u32
                            | WS_VSCROLL.0
                            | WS_TABSTOP.0,
                    ),
                ),
                Kind::Center | Kind::CenterHeading => ("STATIC", WINDOW_STYLE(1)),
                Kind::Slider => (
                    "msctls_trackbar32",
                    WINDOW_STYLE(WS_TABSTOP.0 | TBS_NOTICKS),
                ),
                Kind::Progress => ("msctls_progress32", WINDOW_STYLE(PBS_SMOOTH)),
                _ => ("STATIC", WINDOW_STYLE(0)),
            };
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                &HSTRING::from(class),
                &HSTRING::from(if kind == Kind::Document { "" } else { text }),
                WS_CHILD
                    | style
                    | if id == ERROR {
                        WINDOW_STYLE(0)
                    } else {
                        WS_VISIBLE
                    },
                0,
                0,
                1,
                1,
                self.hwnd,
                HMENU(id as usize as *mut _),
                HINSTANCE(GetModuleHandleW(None)?.0),
                None,
            )?;
            if kind == Kind::Document {
                // CreateWindowEx's initial caption has a much smaller length
                // limit than a multiline EDIT document. Set it after creation.
                SendMessageW(hwnd, EM_SETLIMITTEXT, WPARAM(1024 * 1024), LPARAM(0));
                let contents = HSTRING::from(text);
                SendMessageW(
                    hwnd,
                    WM_SETTEXT,
                    WPARAM(0),
                    LPARAM(contents.as_ptr() as isize),
                );
            }
            let font = {
                let state = self.data.borrow();
                state.fonts[match kind {
                    Kind::Heading | Kind::CenterHeading => 1,
                    Kind::Number => 2,
                    Kind::Small => 3,
                    _ => 0,
                }]
                .0
            };
            SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(0));
            if kind == Kind::Progress || kind == Kind::Slider {
                SetWindowTheme(hwnd, w!(""), w!(""))?;
            }
            self.data.borrow_mut().controls.push(Control {
                id,
                hwnd,
                kind,
                text: text.to_owned(),
                checked: false,
                progress: 255,
            });
            Ok(())
        }
    }
    pub fn update_font(&self) -> Result<()> {
        unsafe {
            let dpi = GetDpiForWindow(self.hwnd).max(96) as i32;
            let mut fonts = Vec::new();
            for (height, weight) in [(14, 400), (17, 600), (28, 600), (12, 400), (22, 400)] {
                let font = CreateFontW(
                    -height * dpi / 96,
                    0,
                    0,
                    0,
                    weight,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    5,
                    0,
                    if height == 22 {
                        w!("Segoe UI Symbol")
                    } else {
                        w!("Segoe UI")
                    },
                );
                if font.is_invalid() {
                    bail!("无法创建系统字体");
                }
                fonts.push(Font(font));
            }
            let controls: Vec<_> = self
                .data
                .borrow()
                .controls
                .iter()
                .map(|c| (c.hwnd, c.kind))
                .collect();
            for (hwnd, kind) in controls {
                SendMessageW(
                    hwnd,
                    WM_SETFONT,
                    WPARAM(
                        fonts[match kind {
                            Kind::Heading | Kind::CenterHeading => 1,
                            Kind::Number => 2,
                            Kind::Small => 3,
                            _ => 0,
                        }]
                        .0
                         .0 as usize,
                    ),
                    LPARAM(1),
                );
            }
            let mut state = self.data.borrow_mut();
            state.dpi = dpi;
            state.fonts = fonts;
            Ok(())
        }
    }
    pub fn refresh_theme(&self) {
        let dark = system_dark();
        if self.data.borrow().palette.dark != dark {
            self.data.borrow_mut().palette = Palette::new(dark);
            self.apply_theme();
        }
    }
    fn apply_theme(&self) {
        unsafe {
            let (dark, track, accent, controls) = {
                let state = self.data.borrow();
                (
                    i32::from(state.palette.dark),
                    state.palette.track,
                    state.palette.accent,
                    state
                        .controls
                        .iter()
                        .filter(|c| c.kind == Kind::Progress)
                        .map(|c| c.hwnd)
                        .collect::<Vec<_>>(),
                )
            };
            // Optional on older Windows; the client-area palette still switches normally.
            let _ = DwmSetWindowAttribute(
                self.hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                (&dark as *const i32).cast(),
                4,
            );
            for hwnd in controls {
                SendMessageW(hwnd, PBM_SETBKCOLOR, WPARAM(0), LPARAM(track.0 as isize));
                SendMessageW(hwnd, PBM_SETBARCOLOR, WPARAM(0), LPARAM(accent.0 as isize));
            }
            let _ = RedrawWindow(
                self.hwnd,
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }
    pub fn layout(&self) {
        unsafe {
            let mut client = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut client);
            let (dpi, dialog) = {
                let state = self.data.borrow();
                (state.dpi, state.dialog)
            };
            let scale = |v: i32| v * dpi / 96;
            let width = client.right * 96 / dpi;

            let mut rects: Vec<(u16, [i32; 4])> = Vec::new();
            let mut cards = Vec::new();
            let mut put = |id: u16, x: i32, y: i32, w: i32, h: i32| rects.push((id, [x, y, w, h]));
            if dialog && (self.data.borrow().about || self.data.borrow().notices) {
                cards.push(RECT {
                    left: 0,
                    top: 0,
                    right: client.right,
                    bottom: client.bottom,
                });
                if self.data.borrow().notices {
                    put(360, 12, 12, width - 24, client.bottom * 96 / dpi - 24);
                } else {
                    put(340, 16, 102, width - 32, 28);
                    put(341, 16, 137, width - 32, 22);
                    put(342, 16, 166, width - 32, 22);
                    put(BILIBILI, 70, 213, 110, 30);
                    put(PROJECT, 196, 213, 114, 30);
                    put(NOTICES, 60, 265, width - 120, 24);
                    put(343, 16, 301, width - 32, 20);
                    put(344, 16, 324, width - 32, 20);
                }
            } else if dialog {
                if self.is_settings() {
                    let inset = 32;
                    let field_x = 152;
                    let field_width = width - field_x - inset;
                    for (y, height) in [(24, 112), (148, 188)] {
                        cards.push(RECT {
                            left: scale(16),
                            top: scale(y),
                            right: scale(width - 16),
                            bottom: scale(y + height),
                        });
                    }
                    put(352, inset, 36, width - 2 * inset, 24);
                    put(STARTUP, inset, 68, width - 2 * inset, 28);
                    // Match the status line to the checkbox's text, not its indicator.
                    put(STARTUP_STATUS, inset + 29, 100, width - 2 * inset - 29, 20);
                    put(353, inset, 160, width - 2 * inset, 24);
                    put(350, inset, 198, field_x - inset - 16, 28);
                    // Combo height includes the dropdown; its closed field is one row high.
                    put(CLOSE_BEHAVIOR, field_x, 200, field_width, 180);
                    put(RESET_CLOSE, field_x, 234, field_width, 24);
                    put(351, inset, 284, field_x - inset - 16, 28);
                    put(LANGUAGE, field_x, 286, field_width, 170);
                    put(354, inset, 354, width - 2 * inset - 128, 28);
                    put(ABOUT, width - inset - 112, 352, 112, 32);
                    put(ERROR, inset, 400, width - 2 * inset, 28);
                } else {
                    cards.push(RECT {
                        left: 0,
                        top: 0,
                        right: client.right,
                        bottom: client.bottom,
                    });
                    put(950, 24, 22, width - 48, 28);
                    put(951, 24, 58, width - 48, 20);
                    put(DIALOG_TRAY, 24, 88, (width - 60) / 2, 36);
                    put(DIALOG_EXIT, 36 + (width - 60) / 2, 88, (width - 60) / 2, 36);
                    put(REMEMBER, 24, 143, width - 48, 28);
                }
            } else {
                let margin = 12;
                let gap = 10;
                let total = width - 2 * margin;
                let tile = (total - gap) / 2;
                for (x, y, w, h) in [
                    (margin, 12, tile, 112),
                    (margin + tile + gap, 12, tile, 112),
                    (margin, 134, tile, 112),
                    (margin + tile + gap, 134, tile, 112),
                    (margin, 256, total, 82),
                    (margin, 348, total, 112),
                ] {
                    cards.push(RECT {
                        left: scale(x),
                        top: scale(y),
                        right: scale(x + w),
                        bottom: scale(y + h),
                    });
                }
                let lx = margin + 12;
                let rx = margin + tile + gap + 12;
                let inner = tile - 24;
                put(300, lx, 24, inner, 20);
                put(THERMAL_ZONE, lx, 69, inner, 38);
                put(302, rx, 24, inner, 20);
                put(RPM, rx, 69, inner, 38);
                put(303, lx, 146, inner, 20);
                put(MEMORY, lx, 187, inner, 20);
                put(MEMORY_BAR, lx, 233, inner, 6);
                put(304, rx, 146, inner, 20);
                put(DISK, rx, 187, inner, 20);
                put(DISK_BAR, rx, 233, inner, 6);
                put(310, lx, 268, total - 24, 24);
                let option = (total - 24) / 3;
                for (i, id) in [QUIET, BALANCE, PERFORMANCE].into_iter().enumerate() {
                    put(id, lx + i as i32 * option, 301, option, 28);
                }
                put(311, lx, 360, total - 24, 24);
                for (i, id) in [AUTO, MANUAL, MAXIMUM].into_iter().enumerate() {
                    put(id, lx + i as i32 * option, 393, option, 28);
                }
                put(TARGET, lx, 424, total - 96, 30);
                put(PERCENT, width - margin - 73, 421, 61, 38);
                put(331, margin, 474, total - 48, 18);
                put(332, margin, 494, total - 48, 18);
                put(SETTINGS, width - margin - 32, 477, 32, 32);
                put(ERROR, margin, 516, total, 28);
            }
            let controls: Vec<_> = self
                .data
                .borrow()
                .controls
                .iter()
                .map(|c| (c.id, c.hwnd))
                .collect();
            self.data.borrow_mut().cards = cards;
            for (id, r) in rects {
                if let Some((_, hwnd)) = controls.iter().find(|c| c.0 == id) {
                    let _ = MoveWindow(
                        *hwnd,
                        scale(r[0]),
                        scale(r[1]),
                        scale(r[2].max(1)),
                        scale(r[3]),
                        true,
                    );
                }
            }
            let _ = RedrawWindow(
                self.hwnd,
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }
    pub fn retranslate(&self) {
        let settings = self.is_settings();
        let dialog = self.data.borrow().dialog;
        let caption = if self.data.borrow().notices {
            tr("第三方声明与版权", "Third-party notices")
        } else if self.data.borrow().about {
            tr(
                "关于 Lecoo Rust PowerControl",
                "About Lecoo Rust PowerControl",
            )
        } else if settings {
            tr("设置", "Settings")
        } else if dialog {
            tr("关闭窗口", "Close window")
        } else {
            "Lecoo Rust PowerControl"
        };
        unsafe {
            let _ = SetWindowTextW(self.hwnd, &HSTRING::from(caption));
        }
        for (id, text) in [
            (ABOUT, tr("关于…", "About…")),
            (
                341,
                tr("来酷迷你主机控制中心", "Lecoo mini PC control center"),
            ),
            (BILIBILI, tr("哔哩哔哩", "Bilibili")),
            (PROJECT, tr("项目主页", "Project website")),
            (NOTICES, tr("第三方声明与版权", "Third-party notices")),
            (344, tr("保留所有权利。", "All rights reserved.")),
            (300, tr("ACPI 热区温度", "ACPI thermal zone")),
            (302, tr("风扇转速", "Fan speed")),
            (303, tr("内存", "Memory")),
            (304, tr("磁盘", "Disk")),
            (310, tr("电源模式", "Power mode")),
            (QUIET, tr("安静", "Quiet")),
            (BALANCE, tr("均衡", "Balanced")),
            (PERFORMANCE, tr("性能", "Performance")),
            (311, tr("风扇控制", "Fan control")),
            (AUTO, tr("自动", "Automatic")),
            (MANUAL, tr("手动", "Manual")),
            (MAXIMUM, tr("最大", "Maximum")),
            (SETTINGS, tr("设置", "Settings")),
            (
                STARTUP,
                tr(
                    "登录后启动（进入托盘）",
                    "Start at sign-in (minimized to tray)",
                ),
            ),
            (350, tr("关闭窗口时", "When closing")),
            (352, tr("启动", "Startup")),
            (353, tr("窗口与语言", "Window & language")),
            (354, tr("应用信息", "Application")),
            (351, tr("语言", "Language")),
            (RESET_CLOSE, tr("重置关闭选择", "Reset close preference")),
            (
                950,
                tr("关闭窗口后要执行什么操作？", "What would you like to do?"),
            ),
            (DIALOG_TRAY, tr("最小化到托盘", "Minimize to tray")),
            (DIALOG_EXIT, tr("退出", "Exit")),
            (REMEMBER, tr("记住我的选择", "Remember my choice")),
        ] {
            self.text(id, text.into());
        }
        if self.data.borrow().about {
            self.text(
                342,
                format!("{} {}", tr("版本", "Version"), env!("CARGO_PKG_VERSION")),
            );
        }
        if settings {
            self.combo_items(
                CLOSE_BEHAVIOR,
                &[
                    tr("每次询问", "Ask every time"),
                    tr("最小化到托盘", "Minimize to tray"),
                    tr("退出", "Exit"),
                ],
            );
            self.combo_items(
                LANGUAGE,
                &[tr("跟随系统", "System default"), "简体中文", "English"],
            );
        }
    }
    pub fn fan_state(&self, target: u8, ready: bool) {
        unsafe {
            let _ = SetPropW(
                self.hwnd,
                w!("Lecoo.FanTarget"),
                windows::Win32::Foundation::HANDLE((target as usize + 1) as *mut _),
            );
            let _ = SetPropW(
                self.hwnd,
                w!("Lecoo.FanReady"),
                windows::Win32::Foundation::HANDLE((usize::from(ready) + 1) as *mut _),
            );
        }
    }
    pub fn handle(&self, id: u16) -> HWND {
        self.data
            .borrow()
            .controls
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.hwnd)
            .unwrap_or_default()
    }
    pub fn count(&self) -> usize {
        self.data.borrow().controls.len()
    }
    pub fn dark(&self) -> bool {
        self.data.borrow().palette.dark
    }
    pub fn text(&self, id: u16, text: String) {
        if id == ERROR {
            let hwnd = self.handle(id);
            if hwnd.is_invalid() {
                return;
            }
            unsafe {
                if IsWindowVisible(hwnd).as_bool() != !text.is_empty() {
                    if !self.data.borrow().dialog {
                        let mut bounds = RECT::default();
                        let _ = GetWindowRect(self.hwnd, &mut bounds);
                        let delta =
                            28 * self.data.borrow().dpi / 96 * if text.is_empty() { -1 } else { 1 };
                        let _ = SetWindowPos(
                            self.hwnd,
                            None,
                            0,
                            0,
                            bounds.right - bounds.left,
                            bounds.bottom - bounds.top + delta,
                            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                    let _ = ShowWindow(hwnd, if text.is_empty() { SW_HIDE } else { SW_SHOW });
                    let _ = InvalidateRect(self.hwnd, None, true);
                }
            }
        }
        let hwnd = {
            let mut state = self.data.borrow_mut();
            let Some(c) = state.controls.iter_mut().find(|c| c.id == id) else {
                return;
            };
            if c.text == text {
                return;
            }
            c.text = text.clone();
            c.hwnd
        };
        unsafe {
            let _ = SetWindowTextW(hwnd, &HSTRING::from(text));
            let _ = InvalidateRect(hwnd, None, true);
        }
    }
    pub fn check(&self, id: u16, value: bool) {
        let hwnd = {
            let mut state = self.data.borrow_mut();
            let Some(c) = state.controls.iter_mut().find(|c| c.id == id) else {
                return;
            };
            if c.checked == value {
                return;
            }
            c.checked = value;
            c.hwnd
        };
        unsafe {
            let _ = InvalidateRect(hwnd, None, true);
        }
    }
    pub fn checked(&self, id: u16) -> bool {
        self.data
            .borrow()
            .controls
            .iter()
            .find(|c| c.id == id)
            .is_some_and(|c| c.checked)
    }
    pub fn enable(&self, id: u16, value: bool) {
        let hwnd = self.handle(id);
        if hwnd.is_invalid() {
            return;
        }
        unsafe {
            if IsWindowEnabled(hwnd).as_bool() != value {
                let _ = EnableWindow(hwnd, value);
            }
        }
    }
    pub fn mode(&self, mode: Option<u8>) {
        if self.data.borrow().mode != mode {
            self.data.borrow_mut().mode = mode;
            unsafe {
                let hwnd = self.handle(CURRENT_MODE);
                if !hwnd.is_invalid() {
                    let _ = InvalidateRect(hwnd, None, true);
                }
            }
        }
    }
    pub fn progress(&self, id: u16, value: f32) {
        let value = value.clamp(0.0, 100.0) as u8;
        let hwnd = {
            let mut state = self.data.borrow_mut();
            let Some(c) = state.controls.iter_mut().find(|c| c.id == id) else {
                return;
            };
            if c.progress == value {
                return;
            }
            c.progress = value;
            c.hwnd
        };
        unsafe {
            SendMessageW(hwnd, PBM_SETPOS, WPARAM(value as usize), LPARAM(0));
        }
    }
    pub fn percent(&self) -> u8 {
        unsafe {
            SendMessageW(self.handle(TARGET), TBM_GETPOS, WPARAM(0), LPARAM(0))
                .0
                .clamp(35, 100) as u8
        }
    }
}
impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe {
            if self.data.borrow().dialog {
                let owner = GetWindow(self.hwnd, GW_OWNER).unwrap_or_default();
                let _ = EnableWindow(owner, true);
            }
            let _ = DestroyWindow(self.hwnd);
            if let Some(icon) = self.icon.take() {
                let _ = DestroyIcon(icon);
            }
        }
    }
}
