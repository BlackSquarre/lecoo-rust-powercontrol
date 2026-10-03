use windows::{
    core::w,
    Win32::{Foundation::COLORREF, Graphics::Gdi::*, System::Registry::*},
};

pub fn system_dark() -> bool {
    let mut light = 1u32;
    let mut size = 4u32;
    unsafe {
        let _ = RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut light as *mut u32).cast()),
            Some(&mut size),
        );
    }
    light == 0
}
pub fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

pub struct Brush(pub HBRUSH);
impl Brush {
    pub fn new(color: COLORREF) -> Self {
        Self(unsafe { CreateSolidBrush(color) })
    }
}
impl Drop for Brush {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.0 .0));
        }
    }
}

pub struct Palette {
    pub dark: bool,
    pub card: COLORREF,
    pub text: COLORREF,
    pub muted: COLORREF,
    pub border: COLORREF,
    pub accent: COLORREF,
    pub green: COLORREF,
    pub red: COLORREF,
    pub button: COLORREF,
    pub pressed: COLORREF,
    pub track: COLORREF,
    pub background: Brush,
    pub background_color: COLORREF,
    pub surface: Brush,
}
impl Palette {
    pub fn new(dark: bool) -> Self {
        let (bg, card, text, muted, border, button, pressed, track) = if dark {
            (
                rgb(22, 24, 28),
                rgb(31, 34, 39),
                rgb(237, 239, 243),
                rgb(160, 166, 176),
                rgb(58, 63, 72),
                rgb(41, 45, 52),
                rgb(54, 60, 69),
                rgb(56, 61, 70),
            )
        } else {
            (
                rgb(245, 246, 248),
                rgb(255, 255, 255),
                rgb(29, 34, 44),
                rgb(104, 113, 128),
                rgb(222, 226, 233),
                rgb(248, 249, 251),
                rgb(231, 236, 244),
                rgb(228, 231, 236),
            )
        };
        Self {
            dark,
            background_color: bg,
            card,
            text,
            muted,
            border,
            button,
            pressed,
            track,
            accent: if dark {
                rgb(98, 167, 255)
            } else {
                rgb(30, 108, 219)
            },
            green: if dark {
                rgb(80, 205, 138)
            } else {
                rgb(24, 139, 81)
            },
            red: if dark {
                rgb(255, 115, 122)
            } else {
                rgb(207, 57, 66)
            },
            background: Brush::new(bg),
            surface: Brush::new(card),
        }
    }
}
