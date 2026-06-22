//! # Engine Asset V4
//!
//! 统一资源管道（Asset Pipeline）+ 运行时（Asset Runtime）。
//! 支持多资源来源、引用计数、热重载、流加载、内存管理、诊断统计。

pub mod cache;
pub mod dependency;
pub mod event;
pub mod handle;
pub mod identifier;
pub mod loader;
pub mod manager;
pub mod metadata;
pub mod path;
pub mod pipeline;
pub mod plugin;
pub mod processor;
pub mod registry;
pub mod resolver;
pub mod runtime;
pub mod service;
pub mod source;
pub mod state;

pub use cache::AssetCache;
pub use dependency::{DependencyGraph, DependencyTracker};
pub use handle::AssetHandle;
pub use identifier::AssetId;
pub use manager::AssetManager;
pub use pipeline::{AssetPipeline, AssetPipelineContext, AssetRequest, AssetResponse, AssetStage};
pub use plugin::AssetPlugin;
pub use processor::{AssetProcessor as AssetProcessorTrait, ProcessorChain};
pub use registry::AssetRegistry;
pub use resolver::{AssetResolver, DefaultResolver};
pub use runtime::{
    AssetDatabase, AssetJob, AssetRuntime, AssetRuntimePlugin, DiagnosticsService, EvictionPolicy,
    MemoryService, ReferenceEntry, ReloadService, RuntimeContext, RuntimeScheduler, RuntimeService,
    StreamPriority, StreamingService,
};
pub use service::ReloadService as AssetReloadService;
pub use service::{AssetService, QueryService, RegisterService};
pub use source::{
    AssetSource, FilesystemSource, MemorySource, ModSource, ResourcePackManager,
    SourceFileMetadata, SourceManager, SourceMetadata, SourcePriority, SourceRegistry,
};
pub use state::{AssetMetadata, AssetState};
