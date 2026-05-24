// Pulse — Bayanihan Emergency Network
// Copyright (C) 2026 Maverick Fabroa (@mavyfaby)
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod tcp;
pub mod config;

use std::process::exit;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let _config = config::load().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        exit(1)
    });
}
