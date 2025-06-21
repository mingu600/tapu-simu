//! # Standardized Builder Patterns
//! 
//! This module provides common traits and patterns for all builders
//! in the Tapu Simu system, ensuring consistency and validation.

use std::fmt::Debug;

/// Common builder trait that all builders should implement
pub trait Builder<T> {
    /// Error type returned when building fails
    type Error: Debug;

    /// Build the final object
    /// 
    /// This consumes the builder and either returns the built object
    /// or an error if the builder state is invalid.
    fn build(self) -> Result<T, Self::Error>;

    /// Validate the current builder state without consuming it
    /// 
    /// This allows checking if the builder can successfully build
    /// without actually building the object.
    fn validate(&self) -> Result<(), Self::Error>;
}

/// Trait for builders that can be reset to initial state
pub trait ResettableBuilder<T>: Builder<T> {
    /// Reset the builder to its initial state
    fn reset(&mut self);
}

/// Trait for builders that support incremental validation
pub trait ValidatingBuilder<T>: Builder<T> {
    /// Validation context type
    type Context;

    /// Validate a specific aspect of the builder
    fn validate_aspect(&self, context: &Self::Context) -> Result<(), Self::Error>;

    /// Get validation warnings (non-fatal issues)
    fn get_warnings(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Trait for builders that can clone their configuration
pub trait CloneableBuilder<T>: Builder<T> + Clone {
    /// Create a new builder with the same configuration
    fn clone_config(&self) -> Self {
        self.clone()
    }
}

/// Common error types for builders
#[derive(Debug, Clone, PartialEq)]
pub enum BuilderError {
    /// Missing required field
    MissingRequired { field: String },
    /// Invalid value for field
    InvalidValue { field: String, value: String, reason: String },
    /// Validation failed
    ValidationFailed { reason: String },
    /// Configuration conflict
    ConfigConflict { field1: String, field2: String, reason: String },
    /// Data dependency not available
    DataError { resource: String, reason: String },
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::MissingRequired { field } => {
                write!(f, "Missing required field: {}", field)
            }
            BuilderError::InvalidValue { field, value, reason } => {
                write!(f, "Invalid value '{}' for field '{}': {}", value, field, reason)
            }
            BuilderError::ValidationFailed { reason } => {
                write!(f, "Validation failed: {}", reason)
            }
            BuilderError::ConfigConflict { field1, field2, reason } => {
                write!(f, "Configuration conflict between '{}' and '{}': {}", field1, field2, reason)
            }
            BuilderError::DataError { resource, reason } => {
                write!(f, "Data error for resource '{}': {}", resource, reason)
            }
        }
    }
}

impl std::error::Error for BuilderError {}

/// Helper trait for fluent validation chaining
pub trait ValidationChain<T> {
    /// Chain validation with another validation
    fn and_then<F>(self, f: F) -> Result<T, BuilderError>
    where
        F: FnOnce(T) -> Result<T, BuilderError>;

    /// Chain validation with a warning (doesn't fail build)
    fn with_warning<F>(self, f: F) -> Result<T, BuilderError>
    where
        F: FnOnce(&T) -> Option<String>;
}

impl<T> ValidationChain<T> for Result<T, BuilderError> {
    fn and_then<F>(self, f: F) -> Result<T, BuilderError>
    where
        F: FnOnce(T) -> Result<T, BuilderError>,
    {
        self.and_then(f)
    }

    fn with_warning<F>(self, f: F) -> Result<T, BuilderError>
    where
        F: FnOnce(&T) -> Option<String>,
    {
        match self {
            Ok(value) => {
                if let Some(warning) = f(&value) {
                    // In a real implementation, we'd store warnings somewhere
                    eprintln!("Warning: {}", warning);
                }
                Ok(value)
            }
            Err(e) => Err(e),
        }
    }
}

/// Validation context for common validation scenarios
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Whether to be strict about unknown fields/values
    pub strict_mode: bool,
    /// Whether to collect warnings
    pub collect_warnings: bool,
    /// Maximum validation depth
    pub max_depth: u32,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            strict_mode: true,
            collect_warnings: true,
            max_depth: 10,
        }
    }
}

/// Macro for implementing basic validation
#[macro_export]
macro_rules! validate_required {
    ($field:expr, $field_name:expr) => {
        match $field {
            Some(ref value) => Ok(value),
            None => Err(BuilderError::MissingRequired {
                field: $field_name.to_string(),
            }),
        }
    };
}

/// Macro for implementing value validation
#[macro_export]
macro_rules! validate_value {
    ($value:expr, $field_name:expr, $condition:expr, $reason:expr) => {
        if $condition {
            Ok($value)
        } else {
            Err(BuilderError::InvalidValue {
                field: $field_name.to_string(),
                value: format!("{:?}", $value),
                reason: $reason.to_string(),
            })
        }
    };
}