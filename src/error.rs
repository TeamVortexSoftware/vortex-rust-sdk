use std::fmt;

/// Error types for Vortex SDK operations
#[derive(Debug)]
pub enum VortexError {
    /// Invalid API key format or content
    InvalidApiKey(String),
    /// Cryptographic operation failed
    CryptoError(String),
    /// HTTP request failed
    HttpError(String),
    /// API returned an error
    ApiError(String),
    /// JSON serialization/deserialization failed
    SerializationError(String),
    /// Invalid request
    InvalidRequest(String),
}

impl fmt::Display for VortexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VortexError::InvalidApiKey(msg) => write!(f, "Invalid API key: {}", msg),
            VortexError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            VortexError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            VortexError::ApiError(msg) => write!(f, "API error: {}", msg),
            VortexError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            VortexError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
        }
    }
}

impl std::error::Error for VortexError {}
