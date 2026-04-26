//! HMAC-SHA256 challenge/response authentication for wormhole.
//!
//! Part of the `wormhole` project — by THINKING TEAM · XTONY.

use anyhow::{bail, ensure, Result};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncWrite};
use uuid::Uuid;

use crate::shared::{ClientMessage, Delimited, ServerMessage};

/// Handles HMAC-SHA256 challenge/response authentication.
pub struct Authenticator(Hmac<Sha256>);

impl Authenticator {
    /// Build an authenticator from a shared secret string.
    pub fn new(secret: &str) -> Self {
        let hashed_secret = Sha256::new().chain_update(secret).finalize();
        Self(Hmac::new_from_slice(&hashed_secret).expect("HMAC accepts any key size"))
    }

    /// Generate a hex-encoded HMAC response to a UUID challenge.
    pub fn answer(&self, challenge: &Uuid) -> String {
        let mut hmac = self.0.clone();
        hmac.update(challenge.as_bytes());
        hex::encode(hmac.finalize().into_bytes())
    }

    /// Validate a hex-encoded HMAC response to a UUID challenge.
    ///
    /// ```
    /// use wormhole::auth::Authenticator;
    /// use uuid::Uuid;
    ///
    /// let auth = Authenticator::new("s3cr3t");
    /// let challenge = Uuid::new_v4();
    ///
    /// assert!(auth.validate(&challenge, &auth.answer(&challenge)));
    /// assert!(!auth.validate(&challenge, "wrong-answer"));
    /// ```
    pub fn validate(&self, challenge: &Uuid, tag: &str) -> bool {
        if let Ok(tag) = hex::decode(tag) {
            let mut hmac = self.0.clone();
            hmac.update(challenge.as_bytes());
            hmac.verify_slice(&tag).is_ok()
        } else {
            false
        }
    }

    /// Server-side: issue a challenge and verify the client's response.
    pub async fn server_handshake<T: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: &mut Delimited<T>,
    ) -> Result<()> {
        let challenge = Uuid::new_v4();
        stream.send(ServerMessage::Challenge(challenge)).await?;
        match stream.recv_timeout().await? {
            Some(ClientMessage::Authenticate(tag)) => {
                ensure!(self.validate(&challenge, &tag), "invalid secret provided");
                Ok(())
            }
            _ => bail!("server requires a secret, but none was provided"),
        }
    }

    /// Client-side: receive a challenge and respond with the HMAC answer.
    pub async fn client_handshake<T: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: &mut Delimited<T>,
    ) -> Result<()> {
        let challenge = match stream.recv_timeout().await? {
            Some(ServerMessage::Challenge(challenge)) => challenge,
            _ => bail!("expected an authentication challenge from server"),
        };
        let tag = self.answer(&challenge);
        stream.send(ClientMessage::Authenticate(tag)).await?;
        Ok(())
    }
}
