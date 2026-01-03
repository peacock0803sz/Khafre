//! Khafre - Sphinx Documentation Editor
//!
//! A native desktop application for editing Sphinx documentation
//! with live preview and embedded terminal support.

mod app;
mod components;
mod services;
mod state;
mod types;


fn main() {
    env_logger::init();
    dioxus::launch(app::App);
}
