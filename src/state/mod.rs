use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    cmp::{Ord, Ordering},
    collections::{HashMap, HashSet},
};
// use utils::{notifications::PushNotification, sounds::Sounds};
use uuid::Uuid;
use warp::{crypto::DID, raygun::Conversation};

use crate::{Messaging, DEFAULT_PATH};

pub enum Actions {
    AddConversation(Conversation),
    // remove the chat from active_chats but don't delete the conversation
    HideChat(Uuid),
    // show a possibly hidden chat
    ShowChat(Uuid),
    // retrieve existing chat for DID or create a new chat
    ChatWith { rg: Messaging, friend: DID },
    UpdateConversation(ConversationInfo),
    UpdateFavorites(HashSet<Uuid>),
    HideSidebar(bool),
    //DeselectChat,
    SetShowPrerelaseNotice(bool),
    // SendNotification(String, String, Sounds),
}

/// tracks the active conversations. Chagnes are persisted
#[derive(Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct PersistedState {
    /// the currently selected conversation
    pub selected_chat: Option<Uuid>,
    /// all active conversations
    pub active_chats: HashMap<Uuid, ConversationInfo>,
    // RayGun receives messages from conversations whether or not the conversation is on the chats sidebar.
    // although these conversations aren't displayed, Uplink needs to track them so that if the user opens the chat,
    // the correct information is displayed.
    pub all_chats: HashMap<Uuid, Conversation>,
    /// a list of favorited conversations.
    /// Uuid is for Conversation and can be used to look things up in all_chats
    pub favorites: HashSet<Uuid>,
    // show sidebar boolean, used with in mobile view
    pub hide_sidebar: bool,
    pub total_unreads: u32,
    pub show_prerelease_notice: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Eq, PartialEq)]
pub struct LastMsgSent {
    pub value: String,
    pub time: DateTime<Utc>,
}

/// composes `Conversation` with relevant metadata
#[derive(Serialize, Deserialize, Default, Clone, Eq, PartialEq)]
pub struct ConversationInfo {
    pub conversation: Conversation,
    /// the uuid of the last message read.
    /// used to determine the number of unread messages
    pub num_unread_messages: u32,
    /// the first two lines of the last message sent
    pub last_msg_sent: Option<LastMsgSent>,
    /// the time the conversation was created. used to sort the chats
    pub creation_time: DateTime<Utc>,
}

impl Ord for ConversationInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        // partial_cmp never returns None, but if it did, comparing by name is the next best thing.
        self.partial_cmp(other)
            .unwrap_or_else(|| self.conversation.name().cmp(&other.conversation.name()))
    }
}

impl PartialOrd for ConversationInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left = match &self.last_msg_sent {
            Some(left) => left.time,
            None => self.creation_time,
        };

        let right = match &other.last_msg_sent {
            Some(right) => right.time,
            None => other.creation_time,
        };

        Some(left.cmp(&right))
    }
}

pub fn total_notifications(s: &PersistedState) -> u32 {
    let mut count = 0;
    for convo in s.active_chats.iter() {
        let convo_count = convo.1.clone().num_unread_messages;
        count += convo_count;
    }
    count
}

impl PersistedState {
    pub fn load_or_initial() -> Self {
        match std::fs::read(DEFAULT_PATH.read().join(".uplink.state.json")) {
            Ok(b) => serde_json::from_slice::<PersistedState>(&b).unwrap_or_default(),
            Err(_) => PersistedState {
                show_prerelease_notice: true,
                ..Default::default()
            },
        }
    }

    pub fn save(&self) {
        match serde_json::to_vec(self) {
            Ok(bytes) => {
                match std::fs::write(DEFAULT_PATH.read().join(".uplink.state.json"), &bytes) {
                    Ok(_) => {}
                    Err(e) => eprintln!("error saving: {}", e),
                }
            }
            Err(e) => eprintln!("error serializing on save: {}", e),
        }
    }

    pub fn dispatch(&mut self, action: Actions) {
        match action {
            Actions::AddConversation(conversation) => {
                self.all_chats
                    .insert(conversation.id(), conversation.clone());
                self.active_chats.insert(
                    conversation.id(),
                    ConversationInfo {
                        conversation,
                        creation_time: DateTime::from(Local::now()),
                        ..Default::default()
                    },
                );
            }
            Actions::HideChat(conversation_id) => {
                self.active_chats.remove(&conversation_id);
                if self.selected_chat == Some(conversation_id) {
                    self.selected_chat = None;
                }
                // todo: should the hidden chat be removed from favorites too?
                // let favorites = self
                //     .favorites
                //     .iter()
                //     .filter(|id| conversation_id == **id)
                //     .cloned()
                //     .collect();
                // self.favorites = favorites;
            }
            Actions::ShowChat(uuid) => {
                // look up uuid in all_chats
                match self.all_chats.get(&uuid) {
                    // add to active_chats
                    Some(conv) => {
                        self.active_chats.insert(
                            uuid,
                            ConversationInfo {
                                conversation: conv.clone(),
                                ..Default::default()
                            },
                        );
                        // set selected_chat
                        self.selected_chat = Some(uuid);
                    }
                    None => {
                        log::error!("ShowChat called for nonexistent chat. uuid: {}", uuid);
                    }
                }
            }
            //Actions::DeselectChat => {
            //    self.selected_chat = None;
            //}
            Actions::ChatWith { mut rg, friend } => {
                // hopefully there isn't a race condition between this part of code and the part that receives events for new conversations. want this to be done before that
                // event is received.
                let conversation =
                    match warp::async_block_in_place_uncheck(rg.create_conversation(&friend)) {
                        Ok(v) => v,
                        Err(warp::error::Error::ConversationExist { conversation }) => conversation,
                        Err(e) => {
                            log::error!("failed to chat with friend {}: {}", friend, e);
                            return;
                        }
                    };

                self.active_chats.insert(
                    conversation.id(),
                    ConversationInfo {
                        conversation: conversation.clone(),
                        ..Default::default()
                    },
                );
                // set selected_chat
                self.selected_chat = Some(conversation.id());

                // if this conversation was newly created, add it here
                self.all_chats
                    .entry(conversation.id())
                    .or_insert(conversation);
            }
            Actions::UpdateConversation(info) => {
                self.active_chats.insert(info.conversation.id(), info);
            }
            Actions::UpdateFavorites(favorites) => {
                self.favorites = favorites;
            }
            Actions::HideSidebar(slide_bar_bool) => {
                self.hide_sidebar = slide_bar_bool;
            }
            Actions::SetShowPrerelaseNotice(value) => {
                self.show_prerelease_notice = value;
            } // Actions::SendNotification(title, content, sound) => {
              //     let _ = PushNotification(title, content, sound);
              //     PersistedState {
              //         current_chat: self.current_chat,
              //         all_chats: self.all_chats.clone(),
              //         favorites: self.favorites.clone(),
              //         hide_sidebar: self.hide_sidebar,
              //         total_unreads: total_notifications(&self),
              //     }
              // }
        };
        self.total_unreads = total_notifications(self);
        self.save();
    }
}

// doesn't run when the window is closed.
// does run on the value inside of dispatch though.
// basically don't use this
//impl Drop for PersistedState {
//    fn drop(&mut self) {
//        println!("saving PersistedState");
//        self.save();
//    }
//}

impl LastMsgSent {
    pub fn new(msg: &[String]) -> Self {
        Self {
            // the sizing of the conversation box is fixed, so approximate the needed string length using
            // the placeholder text
            value: msg
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<String>>()
                .join("\n")
                .chars()
                .take(24)
                .collect(),
            time: DateTime::from(Local::now()),
        }
    }
}
