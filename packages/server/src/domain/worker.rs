use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRecord {
    pub id: String,
    pub queues: Vec<String>,
    pub capabilities: Vec<String>,
    pub labels: HashMap<String, String>,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
    pub status: String,
    pub metadata: HashMap<String, String>,
}
