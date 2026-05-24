// Pulse — Bayanihan Emergency Network
// Copyright (C) 2026 Maverick Fabroa (@mavyfaby)
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{sync::Arc, time::Duration};

use tokio::net::TcpListener;
use tokio::{io::AsyncReadExt, sync::Semaphore, time::timeout};
use tracing::{debug, error, info};

use crate::{config::AppConfig, tcp::error::TcpServerError};

/// Starts the TCP server.
pub async fn start(config: &AppConfig) -> Result<(), TcpServerError> {
    // Construct the address
    let address = (config.tcp.host.as_str(), config.tcp.port);

    // Bind to the address
    let listener = TcpListener::bind(address).await.map_err(|e| {
        TcpServerError::BindError(format!("{}:{}", config.tcp.host, config.tcp.port), e)
    })?;

    // Print the address
    match listener.local_addr() {
        Ok(addr) => {
            info!("[TCP] Server listening on {}", addr);
            info!(
                "[TCP] Server only allows {} connections",
                config.tcp.max_connections
            );
        }
        Err(err) => {
            error!("[TCP] Server started, but failed to read local address: {err}")
        }
    }

    // Construct initialization values
    let read_timeout = Duration::from_secs(config.tcp.read_timeout_seconds);
    let connection_limit = Arc::new(Semaphore::new(config.tcp.max_connections));

    // Start accepting connections
    loop {
        // Accept a connection
        let (mut socket, peer) = match listener.accept().await {
            Ok(connection) => connection,
            Err(e) => {
                error!("[TCP] Failed to accept connection: {e}");
                continue;
            }
        };

        // Acquire a permit for this connection so the server never exceeds the configured limit.
        let permit = match connection_limit.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(e) => {
                error!("[TCP] Connection limit reached; rejecting {peer}: {e}");
                continue;
            }
        };

        // Spawn a task to handle the connection
        tokio::spawn(async move {
            // Keep the semaphore permit alive until this connection task finishes.
            let _permit = permit;

            // Log the connection
            info!("[TCP] A new connection from {peer}");

            // Create a buffer for reading data
            // This creates a 1024-byte buffer to hold incoming data
            // Initialize the buffer with zeros with u8 type
            let mut buf = vec![0u8; 1024];

            // Start a loop to read data from the socket
            loop {
                // Read data from the socket and store it in the {buf} buffer variable
                match timeout(read_timeout, socket.read(&mut buf)).await {
                    // Client closed the connection gracefully
                    Ok(Ok(0)) => {
                        debug!("[TCP] Connection closed by {peer}");
                        break;
                    }
                    // Received n bytes — echo back to the client
                    Ok(Ok(n)) => {
                        debug!("[TCP] {peer}: {}", String::from_utf8_lossy(&buf[..n]));

                        // TODO: Replace with real packet handling (CBOR parsing, signature verification, etc.)
                    }
                    // Read failed — log and drop the connection
                    Ok(Err(e)) => {
                        error!("[TCP] Read error from {peer}: {e}");
                        break;
                    }
                    Err(_) => {
                        debug!("[TCP] Connection from {peer} timed out after being idle");
                        break;
                    }
                }
            }
        });
    }
}
