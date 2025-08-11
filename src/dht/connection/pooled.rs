//! A module for managing pooled TCP connections in a DHT network.
//!
//! The [`PooledConnection`] struct provides a wrapper around [`TcpStream`] that
//! automatically returns the connection to the pool when dropped.

use std::{
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    sync::OwnedSemaphorePermit,
};

use crate::dht::connection::ConnectionPool;

/// A TCP connection that returns itself to the [`ConnectionPool`] when dropped.
///
/// This wrapper provides transparent access to the underlying [`TcpStream`]
/// through [`Deref`] and [`DerefMut`] implementations, while ensuring proper
/// connection pooling behavior.
///
/// # Examples
///
/// ```no_run
/// use rust_p2p_node::dht::connection::ConnectionPool;
/// use tokio::io::AsyncWriteExt;
/// use std::net::SocketAddr;
///
/// #[tokio::main]
/// async fn main() {
///     let pool = ConnectionPool::new(5, std::time::Duration::from_secs(30));
///     let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
///     
///     // Get a connection from the pool
///     let mut conn = pool.get_connection(addr).await.unwrap();
///     
///     // Use the connection as a regular TcpStream
///     conn.write_all(b"hello").await.unwrap();
///     
///     // Connection automatically returns to pool when dropped
/// }
/// ```
pub struct PooledConnection {
    inner: Option<PooledConnectionInner>,
}

/// Internal representation of a pooled connection
struct PooledConnectionInner {
    stream: TcpStream,
    addr: SocketAddr,
    pool: ConnectionPool,
    _permit: OwnedSemaphorePermit,
}

impl PooledConnection {
    /// Crates a new pooled connection.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream to wrap
    /// * `addr` - The remote address of the connection
    /// * `pool` - The connection pool to return to
    /// * `permit` - Semaphore permit tracking pool capacity
    ///
    /// # Note
    ///
    /// This is typically called internally by [`ConnectionPool`]. Most users
    /// should use [`ConnectionPool::get_connection`] instead.
    pub fn new(
        stream: TcpStream,
        addr: SocketAddr,
        pool: ConnectionPool,
        permit: OwnedSemaphorePermit,
    ) -> Self {
        Self {
            inner: Some(PooledConnectionInner {
                stream,
                addr,
                pool,
                _permit: permit,
            }),
        }
    }

    // /// Explicitly close the connection without returning it to the pool.
    // ///
    // /// This is useful when you know the connection is in a bad state and
    // /// shouldn't be reused.
    // pub async fn close(mut self) -> std::io::Result<()> {
    //     if let Some(inner) = self.inner.take() {
    //         inner.stream.shutdown().await
    //     } else {
    //         Ok(())
    //     }
    // }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            let pool = inner.pool.clone();
            let addr = inner.addr;
            let stream = inner.stream;

            tokio::spawn(async move {
                pool.return_connection(addr, stream).await;
            });
        }
    }
}

impl AsyncRead for PooledConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut self.inner {
            Some(inner) => std::pin::Pin::new(&mut inner.stream).poll_read(cx, buf),
            None => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection already returned to pool",
            ))),
        }
    }
}

impl AsyncWrite for PooledConnection {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match &mut self.inner {
            Some(inner) => std::pin::Pin::new(&mut inner.stream).poll_write(cx, buf),
            None => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection already returned to pool",
            ))),
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut self.inner {
            Some(inner) => std::pin::Pin::new(&mut inner.stream).poll_flush(cx),
            None => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection already returned to pool",
            ))),
        }
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut self.inner {
            Some(inner) => std::pin::Pin::new(&mut inner.stream).poll_shutdown(cx),
            None => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection already returned to pool",
            ))),
        }
    }
}

impl Deref for PooledConnection {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        &self
            .inner
            .as_ref()
            .expect("PooledConnection is empty")
            .stream
    }
}

impl DerefMut for PooledConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self
            .inner
            .as_mut()
            .expect("PooledConnection is empty")
            .stream
    }
}
