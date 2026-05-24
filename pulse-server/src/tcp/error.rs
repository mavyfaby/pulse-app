// Pulse — Bayanihan Emergency Network
// Copyright (C) 2026 Maverick Fabroa (@mavyfaby)
// SPDX-License-Identifier: AGPL-3.0-or-later

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TcpServerError {
    #[error("[TCP] Failed to bind to {0}: {1}")]
    BindError(String, std::io::Error),

    #[error("[TCP] Connection limit reached: {0}")]
    ConnectionLimitError(String),
}
