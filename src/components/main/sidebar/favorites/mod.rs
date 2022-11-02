use crate::{
    components::ui_kit::icon_button::IconButton,
    state::{Actions, ConversationInfo},
    utils, Account, Messaging, LANGUAGE, STATE,
};

use dioxus::prelude::*;
use dioxus_heroicons::outline::Shape;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Props, PartialEq)]
pub struct Props {
    account: Account,
    messaging: Messaging,
}

#[allow(non_snake_case)]
pub fn Favorites(cx: Scope<Props>) -> Element {
    let state = use_atom_ref(&cx, STATE);
    let state2 = state.clone();
    let state3 = state2.clone();
    let l = use_atom_ref(&cx, LANGUAGE).read();
    let popout = use_state(&cx, || false);

    let favString = l.favorites.to_string();
    let newchatdString = l.new_chat.to_string();

    let mut favorites = state.read().favorites.clone();
    let all_chats = state.read().all_chats.clone();
    let all_chats2 = all_chats.clone();

    cx.render(rsx!(
        label {
            "{favString}"
        },
        div {
            class: "favorites",
            (popout).then(|| rsx!(
                div {
                    class: "popout-mask",
                    onclick: move |_| {
                        popout.set(false);
                    },
                    Conversations {
                        all_chats: all_chats2,
                        on_pressed: move |uuid| {
                            favorites.insert(uuid);
                            state2.write().dispatch(Actions::UpdateFavorites(favorites.clone()));
                        },
                    },
                }
            )),
            div {
                class: "labeled",
                IconButton {
                    icon: Shape::Plus,
                    on_pressed: move |_| popout.set(true),
                },
                span {
                    "{newchatdString}"
                }
            },
            state.read().favorites.clone().iter().filter_map(|chat_id| all_chats.get(chat_id)).cloned().map(|conv_info| {
                let state3 = state3.clone();
                cx.render(rsx!(
                    FavoriteChat {
                        mp: cx.props.account.clone(),
                        conversation_info: conv_info.clone(),
                        on_pressed: move |_| {
                            // this goes to an onclick handler
                            // the onclick event should propagate up to the div with class=popout-mask and close the window
                            if state3.read().current_chat != Some(conv_info.conversation.id()) {
                                state3.write().dispatch(Actions::ChatWith(conv_info.clone()));
                            }
                        },
                    }
                ))
            })
        },
    ))
}

#[inline_props]
#[allow(non_snake_case)]
pub fn FavoriteChat<'a>(
    cx: Scope,
    mp: Account,
    conversation_info: ConversationInfo,
    on_pressed: EventHandler<'a, Uuid>,
) -> Element<'a> {
    let conversation_id = conversation_info.conversation.id();
    let (_, conversation_name) = utils::get_username_from_conversation(conversation_info, mp);
    let color = match conversation_info.num_unread_messages > 0 {
        true => "blue",
        _ => "",
    };
    cx.render(rsx! {
        div {
            class: "favorite-container",
            onclick: move |_| on_pressed.call(conversation_id),
            div {
                class: "pfp"
            },
            div {
                class: "pfs {color}"
            }
            span {
                "{conversation_name}"
            }
        }
    })
}

#[inline_props]
#[allow(non_snake_case)]
pub fn Conversations<'a>(
    cx: Scope,
    all_chats: HashMap<Uuid, ConversationInfo>,
    on_pressed: EventHandler<'a, Uuid>,
) -> Element<'a> {
    cx.render(rsx!(
       div {
        class: "add-favorites",
        all_chats.iter().map(|(uuid, _conv)| {
            cx.render(rsx!(
                div {
                    onclick: move |_| on_pressed.call(*uuid),
                    "{uuid}"
                }
            ))
        })
       }
    ))
}
