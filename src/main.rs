mod chip_core;
mod app;

use app::App;

fn main() {
    App::new().run();
}