use lbug::SystemConfig;

/// Builder for database configuration, wrapping `lbug::SystemConfig`.
#[derive(Debug, Clone)]
pub struct GraphConfig {
    pub(crate) buffer_pool_size: Option<u64>,
    pub(crate) max_num_threads: Option<u64>,
    pub(crate) compression: Option<bool>,
    pub(crate) read_only: Option<bool>,
    pub(crate) max_db_size: Option<u64>,
}

impl GraphConfig {
    /// Create a new default configuration.
    pub fn new() -> Self {
        Self {
            buffer_pool_size: None,
            max_num_threads: None,
            compression: None,
            read_only: None,
            max_db_size: None,
        }
    }

    /// Set the buffer pool size in bytes.
    pub fn buffer_pool_size(mut self, size: u64) -> Self {
        self.buffer_pool_size = Some(size);
        self
    }

    /// Set the maximum number of threads.
    pub fn max_num_threads(mut self, n: u64) -> Self {
        self.max_num_threads = Some(n);
        self
    }

    /// Enable or disable compression.
    pub fn compression(mut self, enabled: bool) -> Self {
        self.compression = Some(enabled);
        self
    }

    /// Open the database in read-only mode.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);
        self
    }

    /// Set the maximum database size in bytes.
    pub fn max_db_size(mut self, size: u64) -> Self {
        self.max_db_size = Some(size);
        self
    }

    /// Convert to the underlying `lbug::SystemConfig`.
    pub(crate) fn to_system_config(&self) -> SystemConfig {
        let mut cfg = SystemConfig::default();
        if let Some(v) = self.buffer_pool_size {
            cfg = cfg.buffer_pool_size(v);
        }
        if let Some(v) = self.max_num_threads {
            cfg = cfg.max_num_threads(v);
        }
        if let Some(v) = self.compression {
            cfg = cfg.enable_compression(v);
        }
        if let Some(v) = self.read_only {
            cfg = cfg.read_only(v);
        }
        if let Some(v) = self.max_db_size {
            cfg = cfg.max_db_size(v);
        }
        cfg
    }
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_overrides() {
        let cfg = GraphConfig::new();
        assert!(cfg.buffer_pool_size.is_none());
        assert!(cfg.max_num_threads.is_none());
        assert!(cfg.compression.is_none());
        assert!(cfg.read_only.is_none());
        assert!(cfg.max_db_size.is_none());
    }

    #[test]
    fn builder_sets_buffer_pool_size() {
        let cfg = GraphConfig::new().buffer_pool_size(1024 * 1024);
        assert_eq!(cfg.buffer_pool_size, Some(1024 * 1024));
    }

    #[test]
    fn builder_sets_max_num_threads() {
        let cfg = GraphConfig::new().max_num_threads(4);
        assert_eq!(cfg.max_num_threads, Some(4));
    }

    #[test]
    fn builder_sets_compression() {
        let cfg = GraphConfig::new().compression(true);
        assert_eq!(cfg.compression, Some(true));
    }

    #[test]
    fn builder_sets_read_only() {
        let cfg = GraphConfig::new().read_only(true);
        assert_eq!(cfg.read_only, Some(true));
    }

    #[test]
    fn builder_sets_max_db_size() {
        let cfg = GraphConfig::new().max_db_size(10 * 1024 * 1024);
        assert_eq!(cfg.max_db_size, Some(10 * 1024 * 1024));
    }

    #[test]
    fn builder_chains() {
        let cfg = GraphConfig::new()
            .buffer_pool_size(2048)
            .max_num_threads(8)
            .compression(false)
            .read_only(false)
            .max_db_size(5000);
        assert_eq!(cfg.buffer_pool_size, Some(2048));
        assert_eq!(cfg.max_num_threads, Some(8));
        assert_eq!(cfg.compression, Some(false));
        assert_eq!(cfg.read_only, Some(false));
        assert_eq!(cfg.max_db_size, Some(5000));
    }

    #[test]
    fn to_system_config_does_not_panic() {
        let cfg = GraphConfig::new().buffer_pool_size(4096).max_num_threads(2);
        let _sys = cfg.to_system_config();
    }

    #[test]
    fn default_impl_matches_new() {
        let a = GraphConfig::new();
        let b = GraphConfig::default();
        assert_eq!(a.buffer_pool_size, b.buffer_pool_size);
        assert_eq!(a.max_num_threads, b.max_num_threads);
    }

    #[test]
    fn config_is_clone() {
        let cfg = GraphConfig::new().buffer_pool_size(100);
        let cfg2 = cfg.clone();
        assert_eq!(cfg2.buffer_pool_size, Some(100));
    }
}
