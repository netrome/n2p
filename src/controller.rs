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
    pub fn new() -> anyhow::Result<Self> {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )
            .context("failed to configure tcp for swarm")?
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
            .context("failed to configure behavior for swarm")?
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
            .context("failed to subscribe to gossipsub topic")?;

        swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
            .context("failed to listen with quic")?;
        swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .context("failed to listen with tcp")?;

        Ok(Self {
            model: model::Model::new(),
            swarm,
        })
    }

    pub fn send_note(&mut self, note: note::Signed<note::Note>) {
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
                    //println!("mDNS discovered a new peer: {peer_id}");
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
                    //println!("mDNS discovered peer has expired: {peer_id}");
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
                //println!("Got message {message_id} from {propagation_source}");
                let note = note::Signed::<note::Note>::decode(message.data.as_slice())
                    .expect("decode failed");
                self.model
                    .topics
                    .entry(note.inner.topic.clone())
                    .or_default()
                    .add_note(note);
            }

            other => {
                //println!("Other event: {other:?}");
            }
        }
    }

    pub fn model(&self) -> &model::Model {
        &self.model
    }
}

use crate::note::Decode as _;
use crate::note::Encode as _;

use anyhow::Context;
use libp2p::futures::StreamExt as _;

use std::hash::Hash as _;
use std::hash::Hasher as _;

use crate::model;
use crate::note;

#[cfg(test)]
mod tests {
    use super::*;

    use fake::Fake as _;
    use libp2p::identity;
    use note::Sign as _;
    use rand::RngCore as _;
    use rand::SeedableRng as _;

    #[tokio::test]
    async fn controllers_should_be_able_to_communicate() {
        let mut c1 = Controller::new();

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut secret_key_bytes = [0; 32];
        rng.fill_bytes(&mut secret_key_bytes);
        let keypair = identity::Keypair::ed25519_from_bytes(secret_key_bytes)
            .expect("Failed to generate keypair");

        let note: note::Note = fake::Faker.fake_with_rng(&mut rng);
        let signed = note.sign(&keypair).expect("Failed to sign note");
        let s1 = signed.clone();

        for i in 0..10 {
            //println!("{i}");
            c1.poll().await;
        }
        println!(".... C2");

        let mut c2 = Controller::new();

        for i in 0..10 {
            println!("{i}");
            c2.poll().await;
        }

        println!(".... Both");

        let h1 = tokio::spawn(async move {
            for i in 0..149 {
                println!("c1: {i}");
                c1.poll().await;
            }
            println!("Sent note");
            c1.send_note(s1);

            c1.poll().await;

            c1.model
        });

        let h2 = tokio::spawn(async move {
            for i in 0..200 {
                println!("c2: {i}");
                c2.poll().await;
                if !c2.model.topics.is_empty() {
                    break;
                }
            }

            c2.model
        });

        let m1 = h1.await.expect("oh noooo");
        let m2 = h2.await.expect("oh noooo");

        assert_eq!(m1, m2);
        println!("Model: {:?}", m1);
    }
}
