use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Pending,
    Leased,
    Completed,
    Failed,
}

impl JobState {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobState::Pending => "pending",
            JobState::Leased => "leased",
            JobState::Completed => "completed",
            JobState::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(JobState::Pending),
            "leased" => Some(JobState::Leased),
            "completed" => Some(JobState::Completed),
            "failed" => Some(JobState::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub queue: String,
    pub name: String,
    pub payload: Vec<u8>,
    pub state: JobState,
    pub priority: i32,
    pub attempt: u32,
    pub worker_id: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub failure_reason: Option<String>,
    pub workflow_run_id: String,
    pub activity_id: String,
    pub lease_renewals: u32,
}

#[derive(Debug, Error)]
pub enum JobError {
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: JobState, to: JobState },
}

impl Job {
    pub fn new(id: String, queue: String, name: String, payload: Vec<u8>, priority: i32) -> Self {
        let now = Utc::now();
        Self {
            id,
            queue,
            name,
            payload,
            state: JobState::Pending,
            priority,
            attempt: 1,
            worker_id: None,
            lease_expires_at: None,
            created_at: now,
            updated_at: now,
            failure_reason: None,
            workflow_run_id: String::new(),
            activity_id: String::new(),
            lease_renewals: 0,
        }
    }

    pub fn transition_to(&mut self, to: JobState) -> Result<(), JobError> {
        let valid = matches!(
            (self.state, to),
            (JobState::Pending, JobState::Leased)
                | (JobState::Leased, JobState::Completed)
                | (JobState::Leased, JobState::Failed)
                | (JobState::Leased, JobState::Pending)
        );
        if !valid {
            return Err(JobError::InvalidTransition {
                from: self.state,
                to,
            });
        }
        self.state = to;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_to_leased_ok() {
        let mut job = Job::new("id".into(), "default".into(), "task".into(), vec![], 0);
        assert!(job.transition_to(JobState::Leased).is_ok());
    }

    #[test]
    fn pending_to_completed_invalid() {
        let mut job = Job::new("id".into(), "default".into(), "task".into(), vec![], 0);
        assert!(job.transition_to(JobState::Completed).is_err());
    }
}
