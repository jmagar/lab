//! `OpenAiClient` — async methods against OpenAI (or any OpenAI-compatible server).
//!
//! Auth is `Authorization: Bearer <api-key>`. Base URL defaults to `https://api.openai.com`
//! but any OpenAI-compatible server (Ollama, vLLM, LiteLLM, etc.) works by overriding it.

use crate::core::{Auth, HttpClient};

use super::error::OpenAiError;
use super::types::{
    ChatCompletionRequest, ChatCompletionResponse, EmbeddingsRequest, EmbeddingsResponse,
    ModelsResponse,
};

/// Client for the `OpenAI` (or OpenAI-compatible) API.
pub struct OpenAiClient {
    http: HttpClient,
}

impl OpenAiClient {
    /// Build a client.
    ///
    /// ```no_run
    /// use lab_apis::core::Auth;
    /// use lab_apis::openai::OpenAiClient;
    /// let c = OpenAiClient::new("https://api.openai.com", Auth::Bearer { token: "sk-...".into() }).unwrap();
    /// ```
    ///
    /// # Errors
    /// Returns [`OpenAiError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, OpenAiError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// List available models (`GET /v1/models`).
    ///
    /// Used as the health probe — if this endpoint responds the server is reachable
    /// and the API key has at least read scope.
    ///
    /// # Errors
    /// Returns `OpenAiError::Api` on HTTP failure.
    pub async fn list_models(&self) -> Result<ModelsResponse, OpenAiError> {
        Ok(self.http.get_json("/v1/models").await?)
    }

    /// Ping the models endpoint as a cheap health probe.
    ///
    /// # Errors
    /// Returns `OpenAiError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), OpenAiError> {
        self.list_models().await.map(|_| ())
    }

    /// Create a chat completion (`POST /v1/chat/completions`).
    ///
    /// # Errors
    /// Returns `OpenAiError::Api` on HTTP or decode failure.
    pub async fn chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, OpenAiError> {
        Ok(self.http.post_json("/v1/chat/completions", request).await?)
    }

    /// Create embeddings (`POST /v1/embeddings`).
    ///
    /// # Errors
    /// Returns `OpenAiError::Api` on HTTP or decode failure.
    pub async fn create_embeddings(
        &self,
        request: &EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, OpenAiError> {
        Ok(self.http.post_json("/v1/embeddings", request).await?)
    }
}
