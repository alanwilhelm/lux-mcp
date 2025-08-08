use tracing::{debug, info};
use uuid::Uuid;

use super::context::Role;
use super::manager::ThreadManager;

pub struct ContextReconstructor;

impl ContextReconstructor {
    pub fn reconstruct(manager: &ThreadManager, thread_id: &Uuid) -> Option<String> {
        let context = manager.get_thread(thread_id)?;

        let mut history = String::new();

        // Add header
        history.push_str(&format!("=== Conversation Thread: {} ===\n", thread_id));
        history.push_str(&format!(
            "Started: {:?} ago | Tool: {} | Turns: {}\n\n",
            context.created_at.elapsed(),
            context.tool_name,
            context.turns.len()
        ));

        // Add initial files if any
        if !context.initial_files.is_empty() {
            history.push_str("Files in context:\n");
            for file in &context.initial_files {
                history.push_str(&format!("  - {}\n", file));
            }
            history.push_str("\n");
        }

        // Add conversation turns
        for (i, turn) in context.turns.iter().enumerate() {
            // Role and content
            let role_str = match turn.role {
                Role::User => "USER",
                Role::Assistant => "ASSISTANT",
                Role::System => "SYSTEM",
            };

            history.push_str(&format!(
                "[Turn {}] {} (via {}):\n",
                i + 1,
                role_str,
                turn.tool_used.as_ref().unwrap_or(&"unknown".to_string())
            ));

            // Truncate very long content
            let content = if turn.content.len() > 500 {
                format!("{}... [truncated]", &turn.content[..500])
            } else {
                turn.content.clone()
            };
            history.push_str(&format!("{}\n", content));

            // Add synthesis snapshot if available
            if let Some(synthesis) = &turn.synthesis_snapshot {
                history.push_str(&format!(
                    "  📊 Understanding: {}\n",
                    synthesis.current_understanding
                ));
                history.push_str(&format!(
                    "  📊 Confidence: {:.1}%\n",
                    synthesis.confidence_score * 100.0
                ));

                // Add top insights
                if !synthesis.key_insights.is_empty() {
                    history.push_str("  📊 Key Insights:\n");
                    for insight in synthesis.key_insights.iter().take(3) {
                        history.push_str(&format!("     - {}\n", insight.insight));
                    }
                }
            }

            // Add quality metrics if available
            if let Some(metrics) = &turn.quality_metrics {
                if metrics.circular_reasoning_score > 0.5 {
                    history.push_str(&format!(
                        "  ⚠️ Circular reasoning detected: {:.1}%\n",
                        metrics.circular_reasoning_score * 100.0
                    ));
                }
                if metrics.distractor_fixation_score > 0.5 {
                    history.push_str(&format!(
                        "  ⚠️ Distractor fixation: {:.1}%\n",
                        metrics.distractor_fixation_score * 100.0
                    ));
                }
                history.push_str(&format!(
                    "  📈 Quality: coherence={:.1}%, depth={:.1}%\n",
                    metrics.coherence_score * 100.0,
                    metrics.depth_score * 100.0
                ));
            }

            history.push_str("\n");
        }

        // Add summary
        history.push_str(&format!(
            "=== End of Thread ({}  turns) ===\n",
            context.turns.len()
        ));

        debug!(
            "Reconstructed context for thread {} ({} chars)",
            thread_id,
            history.len()
        );
        Some(history)
    }

    pub fn reconstruct_compact(manager: &ThreadManager, thread_id: &Uuid) -> Option<String> {
        let context = manager.get_thread(thread_id)?;

        let mut history = String::new();

        // Just the essential conversation without metrics
        for turn in &context.turns {
            let role_str = match turn.role {
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::System => "System",
            };

            history.push_str(&format!("{}: {}\n", role_str, turn.content));

            // Only add critical synthesis info
            if let Some(synthesis) = &turn.synthesis_snapshot {
                if synthesis.confidence_score > 0.8 {
                    history.push_str(&format!(
                        "Understanding: {}\n",
                        synthesis.current_understanding
                    ));
                }
            }
        }

        Some(history)
    }

    pub fn get_token_estimate(content: &str) -> usize {
        // Rough estimate: 1 token per 4 characters
        content.len() / 4
    }

    pub fn reconstruct_within_limit(
        manager: &ThreadManager,
        thread_id: &Uuid,
        max_tokens: usize,
    ) -> Option<String> {
        let context = manager.get_thread(thread_id)?;

        let mut history = String::new();
        let mut token_count = 0;

        // Process turns in reverse order (most recent first)
        for turn in context.turns.iter().rev() {
            let turn_text = format!(
                "{}: {}\n",
                match turn.role {
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                    Role::System => "System",
                },
                turn.content
            );

            let turn_tokens = Self::get_token_estimate(&turn_text);

            if token_count + turn_tokens > max_tokens {
                // Add truncation notice
                history.insert_str(0, "[Earlier conversation truncated for context limit]\n\n");
                break;
            }

            // Insert at beginning to maintain chronological order
            history.insert_str(0, &turn_text);
            token_count += turn_tokens;
        }

        info!(
            "Reconstructed {} tokens of context for thread {}",
            token_count, thread_id
        );
        Some(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::threading::manager::ConversationTurn;
    use std::time::Instant;

    #[test]
    fn test_basic_reconstruction() {
        let manager = ThreadManager::new();
        let id = manager.create_thread("test");

        let turn1 = ConversationTurn {
            role: Role::User,
            content: "Hello".to_string(),
            tool_used: Some("confer".to_string()),
            synthesis_snapshot: None,
            quality_metrics: None,
            timestamp: Instant::now(),
        };

        let turn2 = ConversationTurn {
            role: Role::Assistant,
            content: "Hi there!".to_string(),
            tool_used: Some("confer".to_string()),
            synthesis_snapshot: None,
            quality_metrics: None,
            timestamp: Instant::now(),
        };

        manager.add_turn(&id, turn1);
        manager.add_turn(&id, turn2);

        let reconstructed = ContextReconstructor::reconstruct(&manager, &id);
        assert!(reconstructed.is_some());

        let content = reconstructed.unwrap();
        assert!(content.contains("Hello"));
        assert!(content.contains("Hi there!"));
        assert!(content.contains("USER"));
        assert!(content.contains("ASSISTANT"));
    }

    #[test]
    fn test_token_limited_reconstruction() {
        let manager = ThreadManager::new();
        let id = manager.create_thread("test");

        // Add multiple turns
        for i in 0..10 {
            let turn = ConversationTurn {
                role: if i % 2 == 0 {
                    Role::User
                } else {
                    Role::Assistant
                },
                content: format!("Message number {}", i),
                tool_used: Some("test".to_string()),
                synthesis_snapshot: None,
                quality_metrics: None,
                timestamp: Instant::now(),
            };
            manager.add_turn(&id, turn);
        }

        // Reconstruct with token limit
        let reconstructed = ContextReconstructor::reconstruct_within_limit(&manager, &id, 50);
        assert!(reconstructed.is_some());

        let content = reconstructed.unwrap();
        // Should contain recent messages but not all
        assert!(content.contains("Message number 9"));
        assert!(!content.contains("Message number 0"));
        assert!(content.contains("[Earlier conversation truncated"));
    }
}
