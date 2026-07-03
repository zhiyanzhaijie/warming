use dioxus::prelude::*;

use ui::views::HomeView;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    HomeView {},
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
