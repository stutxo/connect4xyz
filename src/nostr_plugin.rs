use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use futures::StreamExt;
use nostr_sdk::{
    prelude::serde_macros::serde_details::SerdeHash, serde_json, Client, ClientMessage, Event,
    EventBuilder, Filter, RelayPoolNotification, Tag, Timestamp,
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    messages::NetworkMessage,
    resources::{NetworkStuff, NostrStuff, SendNetMsg},
};

pub struct NostrPlugin;

impl Plugin for NostrPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NostrStuff::new())
            .insert_resource(NetworkStuff::new())
            .insert_resource(SendNetMsg::new())
            .add_systems(Startup, setup)
            .add_systems(Update, handle_net_msg);
    }
}

fn setup(
    nostr_stuff: Res<NostrStuff>,
    mut network_stuff: ResMut<NetworkStuff>,
    mut send_net_msg: ResMut<SendNetMsg>,
) {
    let (send_tx, send_rx) = futures::channel::mpsc::channel::<String>(1000);
    let (input_tx, mut input_rx) = futures::channel::mpsc::channel::<String>(1000);

    network_stuff.read = Some(send_rx);
    send_net_msg.send = Some(input_tx);

    info!("nostr pubkey {:?}", nostr_stuff.local_keys.public_key());
    let nostr_keys = Arc::new(Mutex::new(nostr_stuff.local_keys));

    spawn_local(async move {
        let client = Client::new(&nostr_keys.lock().unwrap());

        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://relay.damus.io").await.unwrap();
        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://relay.snort.social").await.unwrap();
        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://nostr.lu.ke").await.unwrap();
        client.connect().await;

        let tag = "connect4.xyz".to_string();

        let lfg_msg = NetworkMessage::Lfg;
        let serialized_message = serde_json::to_string(&lfg_msg).unwrap();

        let broadcast_peer = ClientMessage::new_event(
            EventBuilder::new_text_note(serialized_message, &[Tag::Hashtag(tag.clone())])
                .to_event(&nostr_keys.lock().unwrap())
                .unwrap(),
        );

        //send start up game message

        client.send_msg(broadcast_peer).await.unwrap();

        let client_clone = client.clone();
        let tag_clone = tag.clone();
        let nostr_keys_clone: Arc<Mutex<nostr_sdk::Keys>> = nostr_keys.clone();

        spawn_local(async move {
            while let Some(message) = input_rx.next().await {
                info!("sending {:?}", message);

                let input = ClientMessage::new_event(
                    EventBuilder::new_text_note(
                        message,
                        &[
                            Tag::Hashtag(tag_clone.clone()),
                            Tag::Hashtag("MOVE TEST".to_string()),
                        ],
                    )
                    .to_event(&nostr_keys_clone.lock().unwrap())
                    .unwrap(),
                );
                client_clone.clone().send_msg(input).await.unwrap();
            }
        });

        let subscription = Filter::new().since(Timestamp::now()).hashtag(tag.clone());

        client.subscribe(vec![subscription]).await;

        client
            .handle_notifications(|notification| async {
                if let RelayPoolNotification::Event(_url, event) = notification {
                    match send_tx.clone().try_send(event.content.clone()) {
                        Ok(()) => {}
                        Err(e) => error!("Error sending message: {} CHANNEL FULL???", e),
                    };
                }
                Ok(false)
            })
            .await
            .unwrap();
    });
}

fn handle_net_msg(mut network_stuff: ResMut<NetworkStuff>, mut send_net_msg: ResMut<SendNetMsg>) {
    if let Some(ref mut receive_rx) = network_stuff.read {
        while let Ok(Some(message)) = receive_rx.try_next() {
            match serde_json::from_str::<NetworkMessage>(&message) {
                Ok(network_message) => match network_message {
                    NetworkMessage::Lfg => {
                        info!("received lfg");
                        let start_game_msg = NetworkMessage::StartGame;
                        let serialized_message = serde_json::to_string(&start_game_msg).unwrap();

                        match send_net_msg
                            .send
                            .as_mut()
                            .unwrap()
                            .try_send(serialized_message)
                        {
                            Ok(()) => {}
                            Err(e) => error!("Error sending message: {} CHANNEL FULL???", e),
                        };
                    }
                    NetworkMessage::StartGame => {
                        if send_net_msg.start {
                            return;
                        }
                        info!("received start game");
                        let start_game_msg = NetworkMessage::StartGame;
                        let serialized_message = serde_json::to_string(&start_game_msg).unwrap();

                        match send_net_msg
                            .send
                            .as_mut()
                            .unwrap()
                            .try_send(serialized_message)
                        {
                            Ok(()) => {}
                            Err(e) => error!("Error sending message: {} CHANNEL FULL???", e),
                        };
                    }
                    NetworkMessage::Input(content) => {
                        info!("received input {:?}", content);
                    }
                },
                Err(e) => {
                    // Handle deserialization error
                    info!("Failed to deserialize message: {:?}", e);
                }
            }
        }
    }
}
