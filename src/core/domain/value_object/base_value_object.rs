use crate::core::domain::error::{ProxmoxError, ProxmoxResult, ValidationError};
use async_trait::async_trait;
use futures::Future;
use std::sync::Arc;
use std::{backtrace::Backtrace, fmt::Display};
use tokio::sync::RwLock;

/// A trait representing a domain value object with built-in validation,
/// thread-safety, and async capabilities.
///
/// This trait provides a foundation for implementing domain value objects
/// that require validation, thread-safe access, and async operations.
///
/// # Type Parameters
///
/// * `Value`: The underlying type of the value object
///
/// # Safety
///
/// This trait ensures thread-safety through the use of `Arc<RwLock>` and
/// proper async/await patterns.
#[async_trait]
pub trait ValueObject: Send + Sync + 'static {
    /// The underlying type of the value
    type Value: Send + Sync + Clone + Display;

    /// The configuration type for validation
    type ValidationConfig: Send + Sync;

    /// Returns a reference to the internal thread-safe value
    ///
    /// # Returns
    ///
    /// A reference to the `Arc<RwLock>` containing the value
    fn value(&self) -> &Arc<RwLock<Self::Value>>;

    /// Returns the validation configuration for the value object
    /// This can be used to configure validation rules and settings
    /// for the value object.
    ///
    /// # Returns
    ///
    /// The validation configuration for the value object
    fn validation_config() -> Self::ValidationConfig;

    /// Validates the value according to domain rules
    ///
    /// # Arguments
    ///
    /// * `value` - The value to validate
    /// * `config` - The validation configuration for the value object
    ///
    /// # Returns
    ///
    /// * `Ok(())` if validation passes
    /// * `Err(ValidationError)` if validation fails
    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError>;

    /// Returns the value as a clone
    ///
    /// # Returns
    ///
    /// A clone of the value
    async fn as_inner(&self) -> Self::Value {
        self.value().read().await.clone()
    }

    /// Creates a new instance with the given value
    ///
    /// # Arguments
    ///
    /// * `value` - The value to wrap in the value object
    fn create(value: Self::Value) -> Self;

    /// Creates a new validated instance asynchronously
    ///
    /// # Arguments
    ///
    /// * `value` - The value to validate and wrap
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if creation and validation succeed
    /// * `Err(ProxmoxError)` if validation fails
    async fn new(value: Self::Value) -> ProxmoxResult<Self>
    where
        Self: Sized,
    {
        let config = Self::validation_config();
        Self::validate(&value, &config)
            .await
            .map_err(|e| ProxmoxError::Validation {
                source: e,
                backtrace: Backtrace::capture(),
            })?;
        Ok(Self::create(value))
    }

    /// Updates the value asynchronously with validation
    ///
    /// # Arguments
    ///
    /// * `new_value` - The new value to set
    ///
    /// # Returns
    ///
    /// * `Ok(())` if update succeeds
    /// * `Err(ProxmoxError)` if validation or concurrent access fails
    #[allow(dead_code)] // TBD: Right now, this is not used, but it could be useful in the future
    async fn update(&self, new_value: Self::Value) -> ProxmoxResult<()> {
        let config = Self::validation_config();
        Self::validate(&new_value, &config)
            .await
            .map_err(|e| ProxmoxError::Validation {
                source: e,
                backtrace: Backtrace::capture(),
            })?;

        let mut guard = self.value().write().await;
        *guard = new_value;
        Ok(())
    }

    /// Performs an async transformation on the value
    ///
    /// # Arguments
    ///
    /// * `f` - Async closure that transforms the value
    ///
    /// # Type Parameters
    ///
    /// * `T` - The return type of the transformation
    /// * `F` - The transformation function type
    /// * `Fut` - The future returned by the transformation
    ///
    /// # Returns
    ///
    /// * `Ok(T)` if transformation succeeds
    /// * `Err(ProxmoxError)` if transformation or concurrent access fails
    #[allow(dead_code)] // TBD: Right now, this is not used, but it could be useful in the future
    async fn transform<T, F, Fut>(&self, f: F) -> ProxmoxResult<T>
    where
        F: FnOnce(&Self::Value) -> Fut + Send + Sync,
        Fut: Future<Output = ProxmoxResult<T>> + Send,
        T: Send + Sync,
    {
        let guard = self.value().read().await;
        f(&guard).await
    }
}
