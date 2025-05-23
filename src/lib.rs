pub mod embedding;
pub mod embedding_basic;
// pub mod embedding_multi; // Disabled due to missing glowrs dependency
// pub mod embedding_simple; // Disabled due to API incompatibility
pub mod vectordb;
pub mod mcp;

// Use the basic embedding service as the default
pub use embedding_basic::EmbeddingService;
pub use vectordb::{VectorDatabase, Document};
pub use mcp::{McpServer, McpRequest, McpResponse};