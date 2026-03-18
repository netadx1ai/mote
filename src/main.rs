mod models;
mod storage;
mod ui;

fn main() {
    dioxus::launch(ui::app::App);
}
