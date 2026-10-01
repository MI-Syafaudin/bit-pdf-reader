use crate::cancellation::CancellationToken;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum JobStatus {
    #[default]
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobProgress {
    pub current: usize,
    pub total: usize,
    pub percentage: f32,
    pub message: String,
    pub eta_seconds: Option<u64>,
}

impl Default for JobProgress {
    fn default() -> Self {
        Self {
            current: 0,
            total: 1,
            percentage: 0.0,
            message: "Queued".to_string(),
            eta_seconds: None,
        }
    }
}

impl JobProgress {
    pub fn update(&mut self, current: usize, total: usize, message: impl Into<String>) {
        self.current = current;
        self.total = total.max(1);
        self.percentage = ((current as f32 / self.total as f32) * 100.0).min(100.0);
        self.message = message.into();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobInfo {
    pub id: String,
    pub job_type: String,
    pub status: JobStatus,
    pub input_path: String,
    pub output_path: Option<String>,
    pub progress: JobProgress,
    pub error_message: Option<String>,
    pub created_at_unix: u64,
    pub completed_at_unix: Option<u64>,
}

impl JobInfo {
    pub fn new(
        id: impl Into<String>,
        job_type: impl Into<String>,
        input_path: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: id.into(),
            job_type: job_type.into(),
            status: JobStatus::Queued,
            input_path: input_path.into(),
            output_path: None,
            progress: JobProgress::default(),
            error_message: None,
            created_at_unix: now,
            completed_at_unix: None,
        }
    }

    pub fn mark_completed(&mut self, output_path: Option<String>) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.status = JobStatus::Completed;
        self.output_path = output_path;
        self.completed_at_unix = Some(now);
        self.progress.percentage = 100.0;
        self.progress.message = "Completed successfully".to_string();
    }

    pub fn mark_failed(&mut self, error: impl Into<String>) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.status = JobStatus::Failed;
        self.error_message = Some(error.into());
        self.completed_at_unix = Some(now);
    }

    pub fn mark_cancelled(&mut self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.status = JobStatus::Cancelled;
        self.completed_at_unix = Some(now);
        self.progress.message = "Cancelled by user".to_string();
    }
}

pub struct ActiveJob {
    pub info: JobInfo,
    pub cancel_token: CancellationToken,
}

impl ActiveJob {
    pub fn new(
        id: impl Into<String>,
        job_type: impl Into<String>,
        input_path: impl Into<String>,
    ) -> Self {
        Self {
            info: JobInfo::new(id, job_type, input_path),
            cancel_token: CancellationToken::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_lifecycle() {
        let mut job = JobInfo::new("job-1", "merge", "input.pdf");
        assert_eq!(job.status, JobStatus::Queued);
        assert!(!job.status.is_finished());

        job.progress.update(5, 10, "Processing page 5 of 10");
        assert_eq!(job.progress.percentage, 50.0);

        job.mark_completed(Some("output.pdf".to_string()));
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.status.is_finished());
        assert_eq!(job.progress.percentage, 100.0);
    }

    #[test]
    fn test_job_cancellation() {
        let active = ActiveJob::new("job-2", "ocr", "scan.pdf");
        assert!(!active.cancel_token.is_cancelled());

        active.cancel_token.cancel();
        assert!(active.cancel_token.is_cancelled());
    }
}
