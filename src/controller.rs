pub struct Controller {
    model: model::Model,
    swarm: libp2p::Swarm<Behavior>,
}

#[derive(libp2p::swarm::NetworkBehaviour)]
struct Behavior {
    gossipsub: libp2p::gossipsub::Behaviour,
    mdns: libp2p::mdns::tokio::Behaviour,
}

impl Controller {
    pub fn new() -> Self {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )
            .expect("failed to configure tcp for swarm")
            .with_quic()
            .with_behaviour(|key| {
                // To content-address message, we can take the hash of message and use it as an ID.
                let message_id_fn = |message: &libp2p::gossipsub::Message| {
                    let mut s = std::collections::hash_map::DefaultHasher::new();
                    message.data.hash(&mut s);
                    libp2p::gossipsub::MessageId::from(s.finish().to_string())
                };

                // Set a custom gossipsub configuration
                let gossipsub_config = libp2p::gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(std::time::Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                    .validation_mode(libp2p::gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                    .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                    .build()
                    .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

                // build a gossipsub network behaviour
                let gossipsub = libp2p::gossipsub::Behaviour::new(
                    libp2p::gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let mdns = libp2p::mdns::tokio::Behaviour::new(
                    libp2p::mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;
                Ok(Behavior { gossipsub, mdns })
            })
            .expect("failed to configure behavior for swarm")
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(std::time::Duration::from_secs(60))
            })
            .build();

        // Create a Gossipsub topic
        let topic = libp2p::gossipsub::IdentTopic::new("n2p-test");
        // subscribes to our topic
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic)
            .expect("failed to subscribe to gossipsub topic");

        swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
            .expect("failed to listen with quic");
        swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .expect("failed to listen with tcp");

        Self {
            model: model::Model::new(),
            swarm,
        }
    }

    pub async fn send_note(&mut self, note: note::Signed<note::Note>) {
        let topic = libp2p::gossipsub::IdentTopic::new("n2p-test");
        let encoded_note = note.encode_to_vec().expect("failed to encode to vec");
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), encoded_note)
            .expect("publish note failed");
        self.model
            .topics
            .entry(note.inner.topic.clone())
            .or_default()
            .add_note(note);
    }

    pub async fn poll(&mut self) {
        let event = self.swarm.select_next_some().await;

        match event {
            libp2p::swarm::SwarmEvent::Behaviour(BehaviorEvent::Mdns(
                libp2p::mdns::Event::Discovered(peers),
            )) => {
                for (peer_id, _multiaddr) in peers {
                    println!("mDNS discovered a new peer: {peer_id}");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                }
            }

            libp2p::swarm::SwarmEvent::Behaviour(BehaviorEvent::Mdns(
                libp2p::mdns::Event::Expired(peers),
            )) => {
                for (peer_id, _multiaddr) in peers {
                    println!("mDNS discovered peer has expired: {peer_id}");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }

            libp2p::swarm::SwarmEvent::Behaviour(BehaviorEvent::Gossipsub(
                libp2p::gossipsub::Event::Message {
                    propagation_source,
                    message_id,
                    message,
                },
            )) => {
                println!("Got message {message_id} from {propagation_source}");
                let note = note::Signed::<note::Note>::decode(message.data.as_slice())
                    .expect("decode failed");
                self.model
                    .topics
                    .entry(note.inner.topic.clone())
                    .or_default()
                    .add_note(note);
            }

            _ => {}
        }
    }

    pub fn model(&self) -> &model::Model {
        &self.model
    }
}

use crate::note::Decode as _;
use crate::note::Encode as _;

use libp2p::futures::StreamExt as _;

use std::hash::Hash as _;
use std::hash::Hasher as _;

use crate::model;
use crate::note;
