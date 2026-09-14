mod models;
mod storage;
mod ui;

use dioxus::desktop::tao::event::{Event, WindowEvent};

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Mote")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1200, 800))
                )
                .with_custom_event_handler(|event, _| {
                    match event {
                        Event::WindowEvent { event: WindowEvent::CloseRequested, .. }
                        | Event::WindowEvent { event: WindowEvent::Destroyed, .. }
                        | Event::LoopDestroyed => {
                            ui::app::flush_editor_pending_global();
                        }
                        _ => {}
                    }
                })
        )
        .launch(ui::app::App);
}
