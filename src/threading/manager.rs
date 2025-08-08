use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::context::{QualityMetrics, Role};
use lux_synthesis::SynthesisState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: Role,
    pub content: String,
    pub tool_used: Option<String>,
    pub synthesis_snapshot: Option<SynthesisState>,
    pub quality_metrics: Option<QualityMetrics>,
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadContext {
    pub thread_id: Uuid,
    pub tool_name: String,
    pub turns: Vec<ConversationTurn>,
    pub initial_files: Vec<String>,
    #[serde(skip, default = "Instant::now")]
    pub created_at: Instant,
    #[serde(skip, default = "Instant::now")]
    pub last_accessed: Instant,
}

pub struct ThreadManager {
    threads: Arc<Mutex<HashMap<Uuid, ThreadContext>>>,
    ttl: Duration,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(3 * 60 * 60)) // 3 hours default
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            threads: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    pub fn create_thread(&self, tool_name: &str) -> Uuid {
        let thread_id = Uuid::new_v4();
        let context = ThreadContext {
            thread_id,
            tool_name: tool_name.to_string(),
            turns: Vec::new(),
            initial_files: Vec::new(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
        };

        let mut threads = self.threads.lock();
        threads.insert(thread_id, context);

        info!("Created new thread {} for tool {}", thread_id, tool_name);
        thread_id
    }

    pub fn get_thread(&self, id: &Uuid) -> Option<ThreadContext> {
        let mut threads = self.threads.lock();

        if let Some(context) = threads.get_mut(id) {
            // Check if expired
            let age = Instant::now().duration_since(context.last_accessed);
            if age > self.ttl {
                debug!("Thread {} has expired (age: {:?})", id, age);
                return None;
            }

            context.last_accessed = Instant::now();
            Some(context.clone())
        } else {
            debug!("Thread {} not found", id);
            None
        }
    }

    pub fn add_turn(&self, id: &Uuid, turn: ConversationTurn) -> bool {
        let mut threads = self.threads.lock();

        if let Some(context) = threads.get_mut(id) {
            debug!(
                "Adding turn to thread {}: {} content from {}",
                id,
                turn.content.len(),
                turn.tool_used.as_ref().unwrap_or(&"unknown".to_string())
            );

            context.turns.push(turn);
            context.last_accessed = Instant::now();
            true
        } else {
            warn!("Cannot add turn to non-existent thread {}", id);
            false
        }
    }

    pub fn add_files(&self, id: &Uuid, files: Vec<String>) -> bool {
        let mut threads = self.threads.lock();

        if let Some(context) = threads.get_mut(id) {
            context.initial_files.extend(files);
            context.last_accessed = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn attach_synthesis(&self, thread_id: &Uuid, synthesis: SynthesisState) {
        let mut threads = self.threads.lock();

        if let Some(context) = threads.get_mut(thread_id) {
            if let Some(last_turn) = context.turns.last_mut() {
                last_turn.synthesis_snapshot = Some(synthesis);
                debug!("Attached synthesis to thread {}", thread_id);
            }
        }
    }

    pub fn attach_quality_metrics(&self, thread_id: &Uuid, metrics: QualityMetrics) {
        let mut threads = self.threads.lock();

        if let Some(context) = threads.get_mut(thread_id) {
            if let Some(last_turn) = context.turns.last_mut() {
                last_turn.quality_metrics = Some(metrics);
                debug!("Attached quality metrics to thread {}", thread_id);
            }
        }
    }

    pub fn get_synthesis_history(&self, thread_id: &Uuid) -> Vec<SynthesisState> {
        let threads = self.threads.lock();

        if let Some(context) = threads.get(thread_id) {
            context
                .turns
                .iter()
                .filter_map(|turn| turn.synthesis_snapshot.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_quality_trajectory(&self, thread_id: &Uuid) -> Vec<QualityMetrics> {
        let threads = self.threads.lock();

        if let Some(context) = threads.get(thread_id) {
            context
                .turns
                .iter()
                .filter_map(|turn| turn.quality_metrics.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn cleanup_expired(&self) -> usize {
        let mut threads = self.threads.lock();
        let now = Instant::now();
        let before = threads.len();

        threads.retain(|id, context| {
            let age = now.duration_since(context.last_accessed);
            if age > self.ttl {
                debug!("Removing expired thread {} (age: {:?})", id, age);
                false
            } else {
                true
            }
        });

        let removed = before - threads.len();
        if removed > 0 {
            info!("Cleaned up {} expired threads", removed);
        }
        removed
    }

    pub fn thread_count(&self) -> usize {
        self.threads.lock().len()
    }

    /// Restore a thread from a saved context
    pub fn restore_thread(&self, thread_id: &str, context: ThreadContext) -> Result<()> {
        let mut threads = self.threads.lock();
        let id = Uuid::parse_str(thread_id)?;
        threads.insert(id, context);
        info!("Restored thread: {}", thread_id);
        Ok(())
    }

    /// List all active thread IDs
    pub fn list_active_threads(&self) -> Vec<String> {
        let threads = self.threads.lock();
        threads.keys().map(|id| id.to_string()).collect()
    }

    pub fn get_stats(&self) -> ThreadStats {
        let threads = self.threads.lock();
        let now = Instant::now();

        let mut oldest_age = Duration::from_secs(0);
        let mut total_turns = 0;
        let mut total_age = Duration::from_secs(0);

        for context in threads.values() {
            let age = now.duration_since(context.created_at);
            total_age += age;
            total_turns += context.turns.len();
            if age > oldest_age {
                oldest_age = age;
            }
        }

        let count = threads.len();
        let avg_age = if count > 0 {
            total_age / count as u32
        } else {
            Duration::from_secs(0)
        };

        let avg_turns = if count > 0 {
            total_turns as f32 / count as f32
        } else {
            0.0
        };

        ThreadStats {
            total_threads: count,
            oldest_thread_age: oldest_age,
            average_thread_age: avg_age,
            average_turns_per_thread: avg_turns,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThreadStats {
    pub total_threads: usize,
    pub oldest_thread_age: Duration,
    pub average_thread_age: Duration,
    pub average_turns_per_thread: f32,
}

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_thread_creation() {
        let manager = ThreadManager::new();
        let id = manager.create_thread("test_tool");
        assert!(manager.get_thread(&id).is_some());
    }

    #[test]
    fn test_turn_addition() {
        let manager = ThreadManager::new();
        let id = manager.create_thread("test");

        let turn = ConversationTurn {
            role: Role::User,
            content: "Test message".to_string(),
            tool_used: None,
            synthesis_snapshot: None,
            quality_metrics: None,
            timestamp: Instant::now(),
        };

        assert!(manager.add_turn(&id, turn));

        let context = manager.get_thread(&id).unwrap();
        assert_eq!(context.turns.len(), 1);
        assert_eq!(context.turns[0].content, "Test message");
    }

    #[test]
    fn test_thread_expiration() {
        let manager = ThreadManager::with_ttl(Duration::from_millis(100));
        let id = manager.create_thread("test");

        thread::sleep(Duration::from_millis(200));

        let expired = manager.cleanup_expired();
        assert_eq!(expired, 1);
        assert!(manager.get_thread(&id).is_none());
    }

    #[test]
    fn test_file_attachment() {
        let manager = ThreadManager::new();
        let id = manager.create_thread("test");

        let files = vec!["file1.rs".to_string(), "file2.rs".to_string()];
        assert!(manager.add_files(&id, files));

        let context = manager.get_thread(&id).unwrap();
        assert_eq!(context.initial_files.len(), 2);
    }
}
