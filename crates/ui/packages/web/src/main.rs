use dioxus::prelude::*;

use ui::views::HomeView;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    HomeView {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const WEB_CSS: Asset = asset!("/assets/web.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: WEB_CSS }

        Router::<Route> {}
    }
}
