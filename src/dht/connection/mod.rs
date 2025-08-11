//! Connection pooling for DHT network connections.
//!
//! This module provides a [`ConnectionPool`] struct that manages reusable TCP connections
//! to other nodes in the DHT network.

pub mod pooled;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow};
use tokio::{
    net::TcpStream,
    sync::{Mutex, Semaphore},
    time::timeout,
};

use crate::dht::connection::pooled::PooledConnection;

/// A pool of TCP connections to DHT nodes.
///
/// Manages connections to other nodes, reusing them when possible and enforcing
/// limits on the number of connection per peer.
///
/// # Examples
///
/// ```no_run
/// use rust_p2p_node::dht::connection::ConnectionPool;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     // Create a pool with max 5 connections per peer and 30s idle timeout
///     let pool = ConnectionPool::new(5, Duration::from_secs(30));
///     
///     // Optionally start the background cleaner
///     pool.start_cleaner(Duration::from_secs(60));
/// }
/// ```
#[derive(Clone)]
pub struct ConnectionPool {
    inner: Arc<Mutex<HashMap<SocketAddr, Vec<ConnectionEntry>>>>,
    semaphore: Arc<Semaphore>,
    max_idle_time: Duration,
}

struct ConnectionEntry {
    stream: TcpStream,
    last_used: Instant,
}

impl ConnectionPool {
    /// Creates a new connection pool with the specified limits.
    ///
    /// # Arguments
    ///
    /// * `max_connections_per_peer` - Maximum number of simultaneous connections to a single peer
    /// * `max_idle_time` - How long to keep idle connections before discarding them
    pub fn new(max_connections_per_peer: usize, max_idle_time: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_connections_per_peer)),
            max_idle_time,
        }
    }

    /// Gets a connection to the specified address, either reusing an existing one
    /// or establishing a new connection.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Connection limit is reached
    /// - Connection attempt times out (5s)
    /// - Underlying IO error occurs
    pub async fn get_connection(&self, addr: SocketAddr) -> Result<PooledConnection> {
        if let Some(conn) = self.try_get_healthy_connection(addr).await? {
            return Ok(conn);
        }

        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .context("Failed to acquire semaphore permit")?;

        let stream = match timeout(Duration::from_secs(5), TcpStream::connect(addr)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => return Err(anyhow!("Connection timeout to {}", addr)),
        };

        stream.set_nodelay(true)?;

        Ok(PooledConnection::new(stream, addr, self.clone(), permit))
    }

    /// Starts a background task that periodically cleans up stale connections.
    ///
    /// The cleaner runs at the specified interval and removes connections that
    /// have exceeded the max idle time.
    pub fn start_cleaner(&self, interval: Duration) {
        let pool = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                pool.clean_stale_connections().await
            }
        });
    }

    async fn try_get_healthy_connection(
        &self,
        addr: SocketAddr,
    ) -> Result<Option<PooledConnection>> {
        let mut pool = self.inner.lock().await;

        if let Some(connections) = pool.get_mut(&addr) {
            while let Some(entry) = connections.pop() {
                if entry.last_used.elapsed() < self.max_idle_time
                    && entry.stream.peer_addr().is_ok()
                {
                    entry.stream.set_nodelay(true)?;

                    let permit = self
                        .semaphore
                        .clone()
                        .acquire_owned()
                        .await
                        .context("Failed to acquire semaphore permit")?;

                    return Ok(Some(PooledConnection::new(
                        entry.stream,
                        addr,
                        self.clone(),
                        permit,
                    )));
                }
            }
        }
        Ok(None)
    }

    async fn return_connection(&self, addr: SocketAddr, stream: TcpStream) {
        let _ = stream.set_nodelay(false);

        let mut pool = self.inner.lock().await;
        pool.entry(addr)
            .or_insert_with(Vec::new)
            .push(ConnectionEntry {
                stream,
                last_used: Instant::now(),
            });
    }

    async fn clean_stale_connections(&self) {
        let mut pool = self.inner.lock().await;
        for (_, connections) in pool.iter_mut() {
            connections.retain(|entry| entry.last_used.elapsed() < self.max_idle_time);
        }
    }
}

#[cfg(test)]
mod connection_pool_tests {
    use std::time::Duration;

    use tokio::{net::TcpListener, time::timeout};

    use crate::dht::connection::ConnectionPool;

    #[tokio::test]
    async fn test_connection_reuse() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            loop {
                let _ = listener.accept().await;
            }
        });

        let pool = ConnectionPool::new(5, Duration::from_secs(30));

        let conn1 = pool.get_connection(addr).await.unwrap();
        drop(conn1);

        let conn2 = pool.get_connection(addr).await.unwrap();
        drop(conn2);

        let pool_inner = pool.inner.lock().await;
        assert_eq!(pool_inner.get(&addr).unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_connection_limit() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            loop {
                let _ = listener.accept().await;
            }
        });

        let pool = ConnectionPool::new(2, Duration::from_secs(30));

        let conn1 = pool.get_connection(addr).await.unwrap();
        let conn2 = pool.get_connection(addr).await.unwrap();

        assert!(
            timeout(Duration::from_millis(100), pool.get_connection(addr))
                .await
                .is_err()
        );

        drop(conn1);

        let conn3 = pool.get_connection(addr).await.unwrap();
        drop(conn2);
        drop(conn3);
    }
}
