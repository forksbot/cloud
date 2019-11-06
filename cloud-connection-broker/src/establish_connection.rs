use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::sync::{Mutex, Weak, RwLock};
use tokio::net::{TcpListener, TcpStream};
use std::fs::File;
use futures::{StreamExt, TryStreamExt};
use tokio::net::tcp::split::WriteHalf;
use tokio::prelude::*;
use std::net::Shutdown;
use tokio::codec::{FramedRead, LengthDelimitedCodec};
use bytes::ByteOrder;
use std::collections::BTreeMap;
use std::cmp::Ordering;
use crate::jwt;

pub type SubscriberID = [u8; 32];

#[derive(Serialize, Deserialize)]
struct ToBrokerPacket {
    /// Is this a packet from a subscriber or a publisher?
    connection_type: ConnectionType,
    /// The key ID points to the key to be used for decrypting the subscriber ID and also
    /// to the public key parameters to verify the signature.
    key_id: u32,
    /// Nonce
    nonce: [u8; 12],
    /// The encrypted subscriber ID
    subscriber_id: SubscriberID,
    /// Ed25519 signature
    signature: [u8; 64],
}

#[derive(Serialize, Deserialize)]
enum ConnectionType {
    ConnectSubscriber,
    ConnectPublisher,
}

#[derive(Serialize, Deserialize)]
enum FromBrokerPacket {
    ConnectionACK,
    Timeout,
    /// The provided signature is invalid. This usually means that either the broker hasn't refreshed
    /// its public signature bytes in time
    ConnectionRefusedInvalidSignature,
    /// Informs a publisher connection that the requested client is not connected at the moment.
    ConnectionRefusedNoClientFound,
    /// This message is send to a subscriber connection when a tunnel has been established
    /// between a publisher and this subscriber. A subscriber should establish a new
    /// connection to the broker to be available for further publisher connections.
    SubscriberBound(u64),
}

enum SubscriberClientState {
    ConnectionAvailable(TcpStream),
    /// The receiver can be used to get informed of a new available subscriber connection for a publisher.
    /// The embedded value
    AwaitingNewConnection(crossbeam_channel::Sender<usize>, crossbeam_channel::Receiver<usize>),
    Disconnected(),
}

struct SubscriberClient {
    id: SubscriberID,
    state: SubscriberClientState,
}


impl Ord for SubscriberClient {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for SubscriberClient {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl PartialEq for SubscriberClient {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}


struct PublisherClient {
    connected_to: Weak<SubscriberClient>
}

pub struct SubscriberList {
    list: Vec<SubscriberClient>,
    unsorted_start: usize,
    operations_since_last_sort: usize,
}

impl SubscriberList {
    pub fn new() -> Self {
        SubscriberList {
            list: Default::default(),
            unsorted_start: 0,
            operations_since_last_sort: 0,
        }
    }
    pub fn maintenance(&mut self) {
        self.list.drain_filter(|f| f.state == SubscriberClientState::Disconnected);
        self.list.sort_unstable();
        self.unsorted_start = self.list.len();
        self.operations_since_last_sort = 0;
    }

    pub fn push(&mut self, id: SubscriberID, stream: TcpStream) -> Result<(), crate::Error> {
        let existing = self.lookup(id);
        match existing {
            Some(index) => {
                let entry = &mut self.list[index].state;
                let mut old = &SubscriberClientState::Disconnected;
                std::mem::swap(old, entry);

                match entry.state {
                    SubscriberClientState::ConnectionAvailable(_) => {
                        Err(crate::Error::ClientIdExists)
                    }
                    SubscriberClientState::AwaitingNewConnection(sender, _) => {
                        self.list[index].state = SubscriberClientState::ConnectionAvailable(stream);
                        sender.send(index)
                    }
                    SubscriberClientState::Disconnected() => {
                        self.list[index].state = SubscriberClientState::ConnectionAvailable(stream);
                        Ok(())
                    }
                }
            }
            None => {
                self.list.push(SubscriberClient {
                    id,
                    state: SubscriberClientState::ConnectionAvailable(stream),
                });
                self.operations_since_last_sort += 1;
                Ok(())
            }
        }
    }

    pub fn take(&mut self, id: SubscriberID, stream: TcpStream) {
        self.list.push(SubscriberClient {
            id,
            state: SubscriberClientState::ConnectionAvailable(stream),
        });
        self.operations_since_last_sort += 1;
    }

    fn lookup(&self, id: SubscriberID) -> Option<usize> {
        let r = self.list[0..self.unsorted_start].binary_search_by_key(id, |f| f.id);
        match r {
            // Found within the sorted slice
            Ok(index) => Some(index),
            // Not found in the sorted slice and there is no unsorted area
            Err(_) if self.unsorted_start >= self.list.len() => {
                None
            }
            // Search the unsorted area
            Err(_) => {
                self.list[self.unsorted_start..].iter().position(|&&f| f.id == id)
            }
        }
    }
}

pub type PublisherList = generational_arena::Arena<PublisherClient>;

/// The publishers list. It has pre-allocated slots for 10 concurrent publishers.
struct Publishers {
    publishers: generational_arena::Arena<PublisherClient>
}

impl Default for Publishers {
    fn default() -> Self {
        Publishers {
            publishers: generational_arena::Arena::with_capacity(10)
        }
    }
}

async fn init_connections(stream: TcpStream) {
    use tokio::timer::timeout;

    let verification = match jwt::JWKVerificator::new(&jwk_url).await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to retrieve public key to verify tokens: {}", e);
            return;
        }
    };

    let mut stream = async_bincode::AsyncBincodeStream::<_, ToBrokerPacket, FromBrokerPacket, _>::from(stream).for_async();

    let p = match stream.next().await {
        None => return,
        Some(Err(e)) => {
            info!("Failed to decode received connection packet: {}", e);
            return;
        }
        Some(Ok(v)) => v
    };

    if let Ok(v) = stream.peer_addr() {
        info!("Accepting from: {} - {:?}", v, p.subscriber_id);
    }

}
