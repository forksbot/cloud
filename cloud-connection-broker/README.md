# ohx Cloud Message Broker

This connection broker is tailored for millions of concurrent subscriber connections with a low memory footprint
and low latency. The broker differentiates between subscriber and publisher connections.

The brokers purpose is to overcome [NAT](https://en.wikipedia.org/wiki/Network_address_translation) middleware,
like routers and establish a dedicated, tcp tunnel between a publisher and subscriber
and serve as low latency proxy for small (~4k) messages between two peers. 

**OHX Usecase**:
1. A few non-periodic, short lived connections from publishers. A publisher can be
the Alexa Smarthome skill, a Google Home fulfilment intent or a remote control App.
2. Many, many subscriber connections by the OHX Cloud Connector Addon.

#### Connection type differences
**Publisher connections** are expected to transmit / exchange a few messages with a subscriber connection and disconnect.
**Subscriber connections** are expected to be many and to stay connected for a long period.
They cannot initiate a communication to a publisher connection or another subscriber connection.

To avoid head of line blocking (TCP is not the perfect protocol for multiplexed streams),
each publisher communication request results in a separate tcp subscriber connection. 

## Security

The TCP Stream is not wrapped in TLS.
A TLS session comes with some overhead in terms of connection establishment latency (3 roundtrips vs 1),
but also in terms of memory usage per connection.

To establish a Publisher-Broker connection and Publisher-Subscriber TCP tunnel only one message with a few
bytes (encoding the subscriber target) need to be send.
Those are encrypted and the connection authenticated via [ChaCha20/Poly1305](https://tools.ietf.org/html/rfc7539)
*Authenticated Encryption with Associated Data* (AEAD) directly.

The TCP tunnels payload between peers is opaque to the broker and is end-to-end encrypted,
also via [ChaCha20/Poly1305](https://tools.ietf.org/html/rfc7539).

Find out more about [security aspects, weaknesses and vulnerabilities](doc/security.md).

## Implementation details

This broker is stateless. It must be authenticated to an OAuth server though to encrypt and authenticate new connections.
The OAuth refresh token is embedded during compile time.
The access token is stored in memory and will be refreshed periodically and on start up.

A new tcp connection is requested to transmit a `ConnectPacket`,
framed in a `ToBrokerPacket` bincode serialized packet within 1 second.
The inline signature is verified first.

If it is a publisher connection, the requested subscriber connection, identified by the `subscriber_id`, is looked up:

* If none is found the connection attempt is aborted by responding with a `FromBrokerPacket::ConnectionRefusedNoClientFound`.
* If a subscriber connection could be identified, it will be bound to the publisher, forming a dedicated tcp tunnel.
  The subscriber will be informed 
 
If it is a subscriber connection, it is registered to the internal 

### Connection limitations, invariants

The implementation can handle 5 connection establishments at a time with a back-pressure spmc queue of 100 entries.
If the queue is saturated, no new tcp connections are accepted for that point in time. A connection establishment
is time limited to 1 second.

A TCP keepalive of 2 minutes is used to keep the connection alive in middleware, like routers with NAT
and to detect failing connections within that time period. All connection writes have a timeout of 1 second.
This is reasonable because the estimated message size is 4kb, meaning the required minimum connection speed for this
broker is around 4 kb/s.

### Memory consumption, lookups

Each subscriber connection consumes roughly 128 byte memory by the `TCPStream` object (72 Bytes), the subscriber connection id (32 Bytes)
plus padding/bookkeeping. This excludes operating system overhead by open TCP connections.

This results in the following memory usage for *n* subscriber connections:

| Connections |    Memory |
|------------:|----------:|
|       1.000 |    125 kb |
|      10.000 |   1.25 Mb |
|     100.000 |  12.50 Mb |
|   1.000.000 | 125.00 Mb |

Subscriber connections are kept in a single vector. New connections are pushed to the back.
Removed connections result in an `Optional::None` entry.

To keep the lookup performance mostly `O(log n)`, the vector is sorted in-place from time to time via a maintenance routine.
A lookup consists of a binary search for the sorted part and a linear lookup for the tailing new-connections part.

A hash has not been used because average `O(1)` lookups are not important for "rare" connection establishments,
in contrast to the memory overhead and additional allocations per insert that would have to be paid.

The maintenance routine is triggered by either 1000 changes to the subscriber list (additions / removals) or
after 24 hours after a first change has happened.

Subscriber connections result in a dedicated tokio async task

### Async

The broker is fully asynchronous using *tokio* and Rusts async/await support.
Encryption and signature verification is provided by *Ring*.

The following async tasks are active:
* *1* OAuth Access Token refresh task
* *1* TCP socket listener task 
* *5* connect tasks. Such a task verifies the signature of an incoming connections first packet with a timeout of 1 second.
* *N* tasks for *N* publisher connections.
 
Because subscriber connections are not expected to send anything before they are in a dedicated communication tunnel
with a publisher, there are no further tasks spawned for subscriber connections. 

## Future improvements for the OHX use-case

If a subscriber connection has a fixed IP or domain, the publisher should bypass the broker completely and establish a P2P
connection.
The client libraries should handle this situation, based on the JWT access token and certain private claims, like
"fixed_ip" or similar.