//! Message-loop timer: keeps tray/session work alive when winit suppresses hidden-window redraws.
use anyhow::{bail, Result};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

thread_local! { static CALLBACKS: RefCell<HashMap<usize, Box<dyn FnMut()>>> = RefCell::new(HashMap::new()); }
unsafe extern "system" fn dispatch(_: HWND, _: u32, id: usize, _: u32) {
    CALLBACKS.with(|callbacks| {
        if let Ok(mut callbacks) = callbacks.try_borrow_mut() {
            if let Some(callback) = callbacks.get_mut(&id) {
                callback();
            }
        }
    });
}
pub struct GuiTimer {
    id: usize,
    _thread: PhantomData<Rc<()>>,
}
impl GuiTimer {
    pub fn new(callback: impl FnMut() + 'static) -> Result<Self> {
        let id = unsafe { SetTimer(None, 0, 500, Some(dispatch)) };
        if id == 0 {
            bail!("无法创建托盘消息定时器");
        }
        CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().insert(id, Box::new(callback));
        });
        Ok(Self {
            id,
            _thread: PhantomData,
        })
    }
}
impl Drop for GuiTimer {
    fn drop(&mut self) {
        unsafe {
            let _ = KillTimer(None, self.id);
        }
        CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().remove(&self.id);
        });
    }
}
