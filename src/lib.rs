pub mod crawler;
pub mod embedding_basic;
pub mod enhanced_vectordb;
pub mod mcp;
pub mod project_manager;
pub mod vectordb;

// Use the basic embedding service as the default
pub use embedding_basic::EmbeddingService;
pub use enhanced_vectordb::EnhancedVectorDbService;
pub use mcp::CodeRagServer;
pub use vectordb::{Document, VectorDatabase};
