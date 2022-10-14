use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn ExtensionPlaceholder(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "extension-renderer",
            "Ext. Frame",
            button {
                "Get Extensions"
            }
        }
    })
}
