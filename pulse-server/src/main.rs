// Pulse — Bayanihan Emergency Network
// Copyright (C) 2026 Maverick Fabroa (@mavyfaby)
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod config;
pub mod tcp;

use std::process::exit;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::load().unwrap_or_else(|e| {
        eprintln!("[main.config] Failed to load configuration: {}", e);
        exit(1)
    });

    if let Err(e) = tcp::server::start(&config).await {
        eprintln!("[main.tcp] Server error: {e}");
        exit(1);
    }
}
