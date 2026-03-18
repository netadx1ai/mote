mod models;
mod storage;
mod ui;

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Mote")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1200, 800))
                )
        )
        .launch(ui::app::App);
}
