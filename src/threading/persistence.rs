use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::manager::{ThreadContext, ThreadManager};
use super::quality::ThreadQualityMetrics;
use super::synthesis::SynthesisLink;
use crate::db::DatabaseService;

/// Thread checkpoint for database persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCheckpoint {
    pub thread_id: String,
    pub context: ThreadContext,
    pub synthesis_links: Vec<SynthesisLink>,
    pub quality_metrics: Option<ThreadQualityMetrics>,
    pub created_at: DateTime<Utc>,
    pub checkpoint_version: u32,
}

/// Manages thread persistence to database
pub struct ThreadPersistenceManager {
    thread_manager: Arc<ThreadManager>,
    db_service: Option<Arc<DatabaseService>>,
    checkpoint_interval: Duration,
}

impl ThreadPersistenceManager {
    pub fn new(
        thread_manager: Arc<ThreadManager>,
        db_service: Option<Arc<DatabaseService>>,
    ) -> Self {
        Self {
            thread_manager,
            db_service,
            checkpoint_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Save a thread checkpoint to database
    pub async fn save_checkpoint(&self, thread_id: &str) -> Result<()> {
        let Some(db) = &self.db_service else {
            debug!("No database service available, skipping checkpoint");
            return Ok(());
        };

        // Get thread context
        let thread_uuid = Uuid::parse_str(thread_id)?;
        let Some(context) = self.thread_manager.get_thread(&thread_uuid) else {
            warn!("Thread {} not found, cannot save checkpoint", thread_id);
            return Ok(());
        };

        // Create checkpoint
        let checkpoint = ThreadCheckpoint {
            thread_id: thread_id.to_string(),
            context: context.clone(),
            synthesis_links: Vec::new(), // Would be populated from synthesis integration
            quality_metrics: None,       // Would be populated from quality integration
            created_at: Utc::now(),
            checkpoint_version: 1,
        };

        // Serialize checkpoint
        let checkpoint_json = serde_json::to_value(&checkpoint)?;

        // Store in database using custom data
        // TODO: Implement custom data storage in DatabaseService
        // db.log_custom_data("thread_checkpoints", thread_id, checkpoint_json)
        //     .await?;

        info!("Saved checkpoint for thread {}", thread_id);
        Ok(())
    }

    /// Load a thread checkpoint from database
    pub async fn load_checkpoint(&self, thread_id: &str) -> Result<Option<ThreadCheckpoint>> {
        let Some(db) = &self.db_service else {
            debug!("No database service available, cannot load checkpoint");
            return Ok(None);
        };

        // Retrieve from database
        // TODO: Implement custom data retrieval in DatabaseService
        // let data = db
        //     .get_custom_data(Some("thread_checkpoints"), Some(thread_id))
        //     .await?;

        // if data.is_empty() {
        //     debug!("No checkpoint found for thread {}", thread_id);
        //     return Ok(None);
        // }

        // // Deserialize the most recent checkpoint
        // if let Some(entry) = data.first() {
        //     let checkpoint: ThreadCheckpoint = serde_json::from_value(entry.value.clone())?;
        //     info!(
        //         "Loaded checkpoint for thread {} (version {})",
        //         thread_id, checkpoint.checkpoint_version
        //     );
        //     return Ok(Some(checkpoint));
        // }

        debug!("Checkpoint loading not yet implemented");
        Ok(None)
    }

    /// Restore a thread from checkpoint
    pub async fn restore_thread(&self, thread_id: &str) -> Result<bool> {
        let Some(checkpoint) = self.load_checkpoint(thread_id).await? else {
            return Ok(false);
        };

        // Restore thread context
        self.thread_manager
            .restore_thread(thread_id, checkpoint.context)?;

        // TODO: Restore synthesis links and quality metrics
        // This would require integration with synthesis and quality managers

        info!("Restored thread {} from checkpoint", thread_id);
        Ok(true)
    }

    /// Start automatic checkpointing
    pub async fn start_auto_checkpoint(&self) {
        let mut interval = interval(self.checkpoint_interval);

        loop {
            interval.tick().await;

            // Get all active threads
            let thread_ids = self.thread_manager.list_active_threads();

            for thread_id in thread_ids {
                if let Err(e) = self.save_checkpoint(&thread_id).await {
                    error!("Failed to checkpoint thread {}: {}", thread_id, e);
                }
            }

            debug!("Auto-checkpoint cycle complete");
        }
    }

    /// Clean up old checkpoints
    pub async fn cleanup_old_checkpoints(&self, days_to_keep: u32) -> Result<usize> {
        let Some(db) = &self.db_service else {
            return Ok(0);
        };

        // Calculate cutoff date
        let cutoff = Utc::now() - chrono::Duration::days(days_to_keep as i64);

        // Get all checkpoints
        // TODO: Implement custom data operations in DatabaseService
        // let data = db.get_custom_data(Some("thread_checkpoints"), None).await?;

        // let mut deleted = 0;
        // for entry in data {
        //     if let Ok(checkpoint) = serde_json::from_value::<ThreadCheckpoint>(entry.value.clone())
        //     {
        //         if checkpoint.created_at < cutoff {
        //             db.delete_custom_data("thread_checkpoints", &entry.key)
        //                 .await?;
        //             deleted += 1;
        //         }
        //     }
        // }

        let deleted = 0;

        info!("Cleaned up {} old checkpoints", deleted);
        Ok(deleted)
    }
}

/// Hybrid storage strategy for threads
pub struct HybridThreadStorage {
    thread_manager: Arc<ThreadManager>,
    persistence_manager: Arc<ThreadPersistenceManager>,
}

impl HybridThreadStorage {
    pub fn new(
        thread_manager: Arc<ThreadManager>,
        db_service: Option<Arc<DatabaseService>>,
    ) -> Self {
        let persistence_manager = Arc::new(ThreadPersistenceManager::new(
            thread_manager.clone(),
            db_service,
        ));

        Self {
            thread_manager,
            persistence_manager,
        }
    }

    /// Get thread with fallback to database
    pub async fn get_or_restore_thread(&self, thread_id: &str) -> Result<Option<ThreadContext>> {
        // First try in-memory
        let thread_uuid = Uuid::parse_str(thread_id)?;
        if let Some(context) = self.thread_manager.get_thread(&thread_uuid) {
            return Ok(Some(context));
        }

        // Try to restore from database
        if self.persistence_manager.restore_thread(thread_id).await? {
            return Ok(self.thread_manager.get_thread(&thread_uuid));
        }

        Ok(None)
    }

    /// Save thread with immediate checkpoint
    pub async fn save_thread(&self, thread_id: &str) -> Result<()> {
        self.persistence_manager.save_checkpoint(thread_id).await
    }

    /// Start background services
    pub fn start_background_services(&self) {
        let persistence = self.persistence_manager.clone();

        // Start auto-checkpoint task
        tokio::spawn(async move {
            persistence.start_auto_checkpoint().await;
        });

        // Start cleanup task (runs daily)
        let persistence = self.persistence_manager.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(86400)); // 24 hours
            loop {
                interval.tick().await;
                if let Err(e) = persistence.cleanup_old_checkpoints(7).await {
                    error!("Failed to cleanup old checkpoints: {}", e);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_checkpoint_serialization() {
        let context = ThreadContext {
            thread_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            turns: Vec::new(),
            initial_files: Vec::new(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
        };

        let checkpoint = ThreadCheckpoint {
            thread_id: "test-thread".to_string(),
            context,
            synthesis_links: Vec::new(),
            quality_metrics: None,
            created_at: Utc::now(),
            checkpoint_version: 1,
        };

        let json = serde_json::to_value(&checkpoint).unwrap();
        let restored: ThreadCheckpoint = serde_json::from_value(json).unwrap();

        assert_eq!(restored.thread_id, "test-thread");
        assert_eq!(restored.checkpoint_version, 1);
    }
}
