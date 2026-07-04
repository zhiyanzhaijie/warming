use dioxus::prelude::*;

use ui::views::{HomeView, PracticeView, SettingsView};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    HomeView {},
    #[route("/practice/:piece_id/:arrangement_id")]
    PracticeView { piece_id: String, arrangement_id: String },
    #[route("/settings")]
    SettingsView {},
}

const DESKTOP_CSS: Asset = asset!("/assets/desktop.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: DESKTOP_CSS }

        Router::<Route> {}
    }
}
