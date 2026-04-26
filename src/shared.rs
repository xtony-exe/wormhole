//! Shared data structures, protocol definitions, and transport utilities.
//!
//! Part of the `wormhole` project — by THINKING TEAM · XTONY.

use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use tokio_util::codec::{AnyDelimiterCodec, Framed, FramedParts};
use tracing::trace;
use uuid::Uuid;

/// TCP port used for control connections between client and server.
pub const CONTROL_PORT: u16 = 7835;

/// Maximum byte length for a JSON frame in the stream.
pub const MAX_FRAME_LENGTH: usize = 256;

/// Timeout for initial network connections and protocol handshakes.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(3);

/// A message sent from the client over the control connection.
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Response to an authentication challenge from the server.
    Authenticate(String),

    /// Initial handshake — specifies a port to forward (0 = auto-assign).
    Hello(u16),

    /// Accepts an incoming TCP connection, using this stream as a proxy.
    Accept(Uuid),
}

/// A message sent from the server over the control connection.
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Authentication challenge, sent as the first message if auth is enabled.
    Challenge(Uuid),

    /// Response to the client's Hello, confirming the assigned public port.
    Hello(u16),

    /// No-op heartbeat — tests whether the client is still reachable.
    Heartbeat,

    /// Asks the client to accept and proxy a new incoming TCP connection.
    Connection(Uuid),

    /// Indicates a server-side error, terminating the connection.
    Error(String),
}

/// A length-delimited stream carrying null-terminated JSON frames.
pub struct Delimited<U>(Framed<U, AnyDelimiterCodec>);

impl<U: AsyncRead + AsyncWrite + Unpin> Delimited<U> {
    /// Construct a new delimited stream from a raw async I/O transport.
    pub fn new(stream: U) -> Self {
        let codec = AnyDelimiterCodec::new_with_max_length(vec![0], vec![0], MAX_FRAME_LENGTH);
        Self(Framed::new(stream, codec))
    }

    /// Read the next null-delimited JSON message from the stream.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        trace!("waiting to receive json message");
        if let Some(next_message) = self.0.next().await {
            let byte_message = next_message.context("frame error: invalid byte length")?;
            let obj = serde_json::from_slice(&byte_message).context("failed to parse message")?;
            Ok(obj)
        } else {
            Ok(None)
        }
    }

    /// Read the next message, with a timeout (used during handshakes).
    pub async fn recv_timeout<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        timeout(NETWORK_TIMEOUT, self.recv())
            .await
            .context("timed out waiting for initial message")?
    }

    /// Send a null-terminated JSON message on the stream.
    pub async fn send<T: Serialize>(&mut self, msg: T) -> Result<()> {
        trace!("sending json message");
        self.0.send(serde_json::to_string(&msg)?).await?;
        Ok(())
    }

    /// Consume this object, returning the underlying buffers and transport.
    pub fn into_parts(self) -> FramedParts<U, AnyDelimiterCodec> {
        self.0.into_parts()
    }
}
