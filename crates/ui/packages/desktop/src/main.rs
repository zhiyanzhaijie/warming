use dioxus::prelude::*;

use ui::views::{HomeView, PracticeView};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    HomeView {},
    #[route("/practice/:piece_id/:arrangement_id")]
    PracticeView { piece_id: String, arrangement_id: String },
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
