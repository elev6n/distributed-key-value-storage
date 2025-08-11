use std::time::Duration;

/// Configureation parameters for the DHT node
#[derive(Debug, Clone)]
pub struct DhtConfig {
    /// Replication factor (k in Kademlia)
    pub replication: ReplicationConfig,
    /// Maximum size of each k-bucket
    pub kbucket_size: usize,
    /// Connection pool settings
    pub connection_pool: ConnectionPoolConfig,
    /// Storage settings
    pub storage: StorageConfig,
    /// Timeout for network operations
    pub operation_timeout: Duration,
    /// Interval for maintenance tasks (health checks, replication etc.)
    pub maintenance_interval: Duration,
    pub health_check: HealthCheckConfig,
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Max connections per peer
    pub max_connections_per_peer: usize,
    /// Max idle time for connections
    pub max_idle_time: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Maximum number of key-value pairs to store
    pub max_entries: usize,
    /// Default time-to-live for stored values (in seconds)
    pub default_ttl: u64,
    /// Interval for checking expired values (in seconds)
    pub expiration_check_interval: u64,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between peer health checks
    pub interval: Duration,
    /// Timeout for health check request
    pub timeout: Duration,
    /// Number of failed attempts before marking peer as dead
    pub max_failures: u8
}

/// Replication configuration
#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    pub factor: usize,
    /// Interval between replication checks
    pub check_interval: Duration,
    /// Number of parallel replication requests
    pub parallelism: usize
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            replication: ReplicationConfig {
                factor: 5,
                check_interval: Duration::from_secs(60),
                parallelism: 3,
            },
            kbucket_size: 20,
            connection_pool: ConnectionPoolConfig {
                max_connections_per_peer: 3,
                max_idle_time: Duration::from_secs(300),
                connect_timeout: Duration::from_secs(3),
            },
            storage: StorageConfig {
                max_entries: 10_000,
                default_ttl: 3600,
                expiration_check_interval: 60,
            },
            operation_timeout: Duration::from_secs(3),
            maintenance_interval: Duration::from_secs(30),
            health_check: HealthCheckConfig {
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(3),
                max_failures: 2,
            },
        }
    }
}