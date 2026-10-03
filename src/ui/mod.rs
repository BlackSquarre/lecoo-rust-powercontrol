pub mod app;
mod native;
mod native_theme;
mod preferences;

pub use app::ControlCenterApp;
pub fn initialize_language() {
    lecoo_control_center::localization::set_language(preferences::load_language());
}
