use dioxus::prelude::*;
use dioxus_heroicons::outline::Shape;

use crate::{
    components::ui_kit::button::{self, Button},
    LANGUAGE,
};

#[allow(non_snake_case)]
pub fn Welcome(cx: Scope) -> Element {
    let l = use_atom_ref(&cx, LANGUAGE).read();

    cx.render(rsx! {
        div {
            id: "welcome",
            img {
                src: "extra/assets/img/uplink_muted.png"
            },
            p {
                class: "muted",
                "No active chats, wanna make one?"
            },
            Button {
                icon: Shape::Plus,
                text: l.start_one.to_string(),
                state: button::State::Secondary,
                on_pressed: move |_| {} // show_friends.set(true),
            },
        }
    })
}
