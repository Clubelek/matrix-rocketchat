use std::collections::HashMap;

use matrix_rocketchat::api::RestApi;
use matrix_rocketchat::models::Events;
use reqwest::{Method, StatusCode};
use ruma_events::EventType;
use ruma_events::collections::all::Event;
use ruma_events::room::member::{MemberEvent, MemberEventContent, MembershipState};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::to_string;
use super::HS_TOKEN;

pub fn create_admin_room(as_url: String, admin_room_id: RoomId, test_user_id: UserId, bot_user_id: UserId) {
    invite(as_url.clone(), admin_room_id.clone(), test_user_id.clone(), bot_user_id.clone());
    join(as_url, admin_room_id, bot_user_id);
}

pub fn invite(as_url: String, room_id: RoomId, sender_id: UserId, user_id: UserId) {
    let url = format!("{}/transactions/{}", as_url, "specid");

    let invite_event = MemberEvent {
        content: MemberEventContent {
            avatar_url: None,
            displayname: None,
            membership: MembershipState::Invite,
            third_party_invite: None,
        },
        event_id: EventId::new("localhost").unwrap(),
        event_type: EventType::RoomMember,
        invite_room_state: None,
        prev_content: None,
        room_id: room_id.clone(),
        state_key: format!("{}", user_id),
        unsigned: None,
        user_id: sender_id,
    };

    let events = Events { events: vec![Box::new(Event::RoomMember(invite_event))] };

    let invite_payload = to_string(&events).unwrap();

    simulate_message_from_matrix(Method::Put, &url, &invite_payload);
}

pub fn join(as_url: String, room_id: RoomId, user_id: UserId) {
    let url = format!("{}/transactions/{}", as_url, "specid");

    let join_event = MemberEvent {
        content: MemberEventContent {
            avatar_url: None,
            displayname: None,
            membership: MembershipState::Join,
            third_party_invite: None,
        },
        event_id: EventId::new("localhost").unwrap(),
        event_type: EventType::RoomMember,
        invite_room_state: None,
        prev_content: None,
        room_id: room_id,
        state_key: format!("{}", user_id),
        unsigned: None,
        user_id: user_id,
    };

    let events = Events { events: vec![Box::new(Event::RoomMember(join_event))] };

    let join_payload = to_string(&events).unwrap();

    simulate_message_from_matrix(Method::Put, &url, &join_payload);
}

pub fn leave_room(as_url: String, room_id: RoomId, user_id: UserId) {
    let url = format!("{}/transactions/{}", as_url, "specid");

    let leave_event = MemberEvent {
        content: MemberEventContent {
            avatar_url: None,
            displayname: None,
            membership: MembershipState::Leave,
            third_party_invite: None,
        },
        event_id: EventId::new("localhost").unwrap(),
        event_type: EventType::RoomMember,
        invite_room_state: None,
        prev_content: None,
        room_id: room_id,
        state_key: format!("{}", user_id),
        unsigned: None,
        user_id: user_id,
    };

    let events = Events { events: vec![Box::new(Event::RoomMember(leave_event))] };
    let leave_payload = to_string(&events).unwrap();

    simulate_message_from_matrix(Method::Put, &url, &leave_payload);
}

pub fn simulate_message_from_matrix(method: Method, url: &str, payload: &str) -> (String, StatusCode) {
    let mut params = HashMap::new();
    params.insert("access_token", HS_TOKEN);
    RestApi::call(method, url, payload, &mut params, None).unwrap()
}
