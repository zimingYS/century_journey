pub mod identifier;
pub mod loader;
pub mod registry;

// Note: cache and plugin have been moved to content/tag/.
// TagPlugin (with BlockRegistry dependency) is now TagContentPlugin in content::tag::plugin.
// CachedTagCache is now in content::tag::cache.
