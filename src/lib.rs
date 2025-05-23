pub mod embedding_basic;
pub mod mcp;
pub mod vectordb;

// Use the basic embedding service as the default
pub use embedding_basic::EmbeddingService;
pub use mcp::{McpRequest, McpResponse, McpServer};
pub use vectordb::{Document, VectorDatabase};
