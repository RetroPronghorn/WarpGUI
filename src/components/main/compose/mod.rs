pub mod messages;
pub mod msg;
pub mod reply;
pub mod topbar;
pub mod write;

use dioxus::prelude::*;
use dioxus_heroicons::outline::Shape;
use warp::raygun::RayGun;

use crate::{
    components::{
        main::compose::{messages::Messages, topbar::TopBar, write::Write},
        ui_kit::icon_button::IconButton,
    },
    state::{Actions, LastMsgSent},
    Account, Messaging, LANGUAGE, STATE,
};

#[derive(PartialEq, Props)]
pub struct Props {
    account: Account,
    messaging: Messaging,
}

#[allow(non_snake_case)]
pub fn Compose(cx: Scope<Props>) -> Element {
    let state = use_atom_ref(&cx, STATE);
    let current_chat = state.read().current_chat;
    let l = use_atom_ref(&cx, LANGUAGE).read();
    let warningMessage = l.prerelease_warning.to_string();

    let blur = state.read().current_chat.is_none();
    let text = use_state(&cx, String::new);
    let show_warning = use_state(&cx, || true);

    cx.render(rsx! {
        div {
            class: "compose",
            if blur {
                rsx!(
                    div {
                        class: "blurmask"
                    }
                )
            } else {
                rsx!(
                    TopBar {
                        account: cx.props.account.clone(),
                        on_call: move |_| {},
                    }
                )
            },
            (**show_warning).then(|| rsx!(
                div {
                    class: "alpha-warning animate__animated animate__slideInDown",
                    "{warningMessage}",
                    IconButton {
                        on_pressed: move |_| {
                            show_warning.set(false);
                        },
                        icon: Shape::Check,
                    }
                },
            )),
            div {
                class: "messages-container",
                div { class: "gradient_mask" },
                Messages {
                    account: cx.props.account.clone(),
                    messaging: cx.props.messaging.clone(),
                }
                div { class: "gradient_mask is_bottom" },
            },
            div {
                class: "writer-container",
                Write {
                    on_submit: move |message: String| {
                        text.set(String::from(""));
                        let mut rg = cx.props.messaging.clone();

                        let text_as_vec = message
                            .split('\n')
                            .filter(|&s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();

                        if text_as_vec.is_empty() {
                            return;
                        }

                        // clicking the send button is meaningless if there isn't a conversation. 
                        if let Some(id) = current_chat {

                            // mutate the state
                            let cur = state.read().all_chats.get(&id).cloned();
                            if let Some( mut conversation_info) = cur {
                                // for multiline messages, take at most 2 lines
                                let v: Vec<String> = text_as_vec.iter().take(2).cloned().collect();
                                let s: String = v.join("\n");
                                // the sizing of the conversation box is fixed, so approximate the needed string length using
                                // the placeholder text
                                let msg = s.chars().take(24).collect();
                                conversation_info.last_msg_sent = Some(LastMsgSent::new(msg));
                                state.write().dispatch(Actions::UpdateConversation(conversation_info));
                            }

                            // TODO: We need to wire this message up to display differently
                            // until we confim whether it was successfully sent or failed
                            if let Err(_e) = warp::async_block_in_place_uncheck(rg.send(id, None, text_as_vec)) {
                                //TODO: Handle error
                            };
                        }
                    },
                    on_upload: move |_| {}
                }
            },
        }
    })
}
