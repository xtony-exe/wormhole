//! Server implementation for the `wormhole` service.
//!
//! By THINKING TEAM — authored by XTONY.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{io, ops::RangeInclusive, sync::Arc, time::Duration};

use anyhow::Result;
use dashmap::DashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, timeout};
use tracing::{info, info_span, warn, Instrument};
use uuid::Uuid;

use crate::auth::Authenticator;
use crate::shared::{ClientMessage, Delimited, ServerMessage, CONTROL_PORT};

/// State structure for the wormhole server.
pub struct Server {
    /// Range of TCP ports that can be forwarded.
    port_range: RangeInclusive<u16>,

    /// Optional secret used to authenticate clients.
    auth: Option<Authenticator>,

    /// Concurrent map of IDs to pending incoming connections.
    conns: Arc<DashMap<Uuid, TcpStream>>,

    /// IP address where the control server binds.
    bind_addr: IpAddr,

    /// IP address where tunnels listen.
    bind_tunnels: IpAddr,

    /// Maximum allowed simultaneous client tunnels.
    max_clients: usize,

    /// Current active client count.
    client_count: Arc<AtomicUsize>,
}

impl Server {
    /// Create a new server with a port range and optional secret.
    pub fn new(port_range: RangeInclusive<u16>, secret: Option<&str>) -> Self {
        assert!(!port_range.is_empty(), "must provide at least one port");
        Server {
            port_range,
            conns: Arc::new(DashMap::new()),
            auth: secret.map(Authenticator::new),
            bind_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            bind_tunnels: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            max_clients: 100,
            client_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Set the IP address to bind the control server on.
    pub fn set_bind_addr(&mut self, bind_addr: IpAddr) {
        self.bind_addr = bind_addr;
    }

    /// Set the IP address where tunnels will listen.
    pub fn set_bind_tunnels(&mut self, bind_tunnels: IpAddr) {
        self.bind_tunnels = bind_tunnels;
    }

    /// Set the maximum number of simultaneous client tunnels.
    pub fn set_max_clients(&mut self, max_clients: usize) {
        self.max_clients = max_clients;
    }

    /// Start the server, accepting client connections.
    pub async fn listen(self) -> Result<()> {
        let this = Arc::new(self);
        let listener = TcpListener::bind((this.bind_addr, CONTROL_PORT)).await?;
        info!(addr = ?this.bind_addr, "wormhole server listening on control port");

        loop {
            let (stream, addr) = listener.accept().await?;
            let this = Arc::clone(&this);

            // Reject if over capacity
            if this.client_count.load(Ordering::Relaxed) >= this.max_clients {
                warn!(?addr, "max clients reached, rejecting connection");
                continue;
            }

            this.client_count.fetch_add(1, Ordering::Relaxed);
            let count = Arc::clone(&this.client_count);

            tokio::spawn(
                async move {
                    info!("incoming connection");
                    if let Err(err) = this.handle_connection(stream).await {
                        warn!(%err, "connection exited with error");
                    } else {
                        info!("connection exited");
                    }
                    count.fetch_sub(1, Ordering::Relaxed);
                }
                .instrument(info_span!("control", ?addr)),
            );
        }
    }

    async fn create_listener(&self, port: u16) -> Result<TcpListener, &'static str> {
        let try_bind = |port: u16| async move {
            TcpListener::bind((self.bind_tunnels, port))
                .await
                .map_err(|err| match err.kind() {
                    io::ErrorKind::AddrInUse => "port already in use",
                    io::ErrorKind::PermissionDenied => "permission denied",
                    _ => "failed to bind to port",
                })
        };
        if port > 0 {
            // Client requested a specific port
            if !self.port_range.contains(&port) {
                return Err("client port number not in allowed range");
            }
            try_bind(port).await
        } else {
            // Try up to 150 random ports — 99.999% success at 85% utilization
            for _ in 0..150 {
                let port = fastrand::u16(self.port_range.clone());
                if let Ok(listener) = try_bind(port).await {
                    return Ok(listener);
                }
            }
            Err("failed to find an available port")
        }
    }

    async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        let mut stream = Delimited::new(stream);

        // Authentication handshake
        if let Some(auth) = &self.auth {
            if let Err(err) = auth.server_handshake(&mut stream).await {
                warn!(%err, "authentication failed");
                stream
                    .send(ServerMessage::Error(err.to_string()))
                    .await?;
                return Ok(());
            }
        }

        match stream.recv_timeout().await? {
            Some(ClientMessage::Authenticate(_)) => {
                warn!("unexpected authenticate message");
                Ok(())
            }
            Some(ClientMessage::Hello(port)) => {
                let listener = match self.create_listener(port).await {
                    Ok(l) => l,
                    Err(err) => {
                        stream.send(ServerMessage::Error(err.into())).await?;
                        return Ok(());
                    }
                };
                let host = listener.local_addr()?.ip();
                let port = listener.local_addr()?.port();
                info!(?host, ?port, "new tunnel client registered");
                stream.send(ServerMessage::Hello(port)).await?;

                loop {
                    // Heartbeat to detect dead clients
                    if stream.send(ServerMessage::Heartbeat).await.is_err() {
                        return Ok(());
                    }
                    const TIMEOUT: Duration = Duration::from_millis(500);
                    if let Ok(result) = timeout(TIMEOUT, listener.accept()).await {
                        let (stream2, addr) = result?;
                        info!(?addr, ?port, "new inbound connection");

                        let id = Uuid::new_v4();
                        let conns = Arc::clone(&self.conns);
                        conns.insert(id, stream2);

                        // Expire stale connections after 10s to prevent leaks
                        tokio::spawn(async move {
                            sleep(Duration::from_secs(10)).await;
                            if conns.remove(&id).is_some() {
                                warn!(%id, "removed stale connection");
                            }
                        });
                        stream.send(ServerMessage::Connection(id)).await?;
                    }
                }
            }
            Some(ClientMessage::Accept(id)) => {
                info!(%id, "forwarding accepted connection");
                match self.conns.remove(&id) {
                    Some((_, mut stream2)) => {
                        let mut parts = stream.into_parts();
                        debug_assert!(
                            parts.write_buf.is_empty(),
                            "framed write buffer not empty"
                        );
                        stream2.write_all(&parts.read_buf).await?;
                        tokio::io::copy_bidirectional(&mut parts.io, &mut stream2).await?;
                    }
                    None => warn!(%id, "no pending connection found for this id"),
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
}
