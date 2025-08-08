use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

use crate::monitoring::MetacognitiveMonitor;

use super::context::QualityMetrics;
use super::manager::ThreadManager;

/// Quality metrics aggregated across a thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadQualityMetrics {
    pub thread_id: String,
    pub overall_quality: f32,
    pub circular_reasoning_score: f32,
    pub distractor_fixation_score: f32,
    pub quality_degradation_score: f32,
    pub intervention_count: usize,
    pub last_updated: DateTime<Utc>,
    pub quality_trend: QualityTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityTrend {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

/// Manages quality metrics for threads
pub struct ThreadQualityManager {
    thread_manager: Arc<ThreadManager>,
    metrics_cache: Arc<Mutex<HashMap<String, ThreadQualityMetrics>>>,
    quality_history: Arc<Mutex<HashMap<String, Vec<QualitySnapshot>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    pub timestamp: DateTime<Utc>,
    pub overall_quality: f32,
    pub tool_name: String,
    pub intervention_triggered: bool,
}

impl ThreadQualityManager {
    pub fn new(thread_manager: Arc<ThreadManager>) -> Self {
        Self {
            thread_manager,
            metrics_cache: Arc::new(Mutex::new(HashMap::new())),
            quality_history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Update quality metrics from a monitoring session
    pub fn update_from_monitor(
        &self,
        thread_id: &str,
        monitor: &MetacognitiveMonitor,
        tool_name: &str,
    ) -> Result<()> {
        // Calculate quality scores from monitor
        let circular_score = self.calculate_circular_score(monitor);
        let distractor_score = self.calculate_distractor_score(monitor);
        let degradation_score = self.calculate_degradation_score(monitor);
        let overall_quality =
            self.calculate_overall_quality(circular_score, distractor_score, degradation_score);

        // Get intervention count
        let intervention_count = monitor.get_intervention_count();

        // Determine quality trend
        let trend = self.determine_trend(thread_id, overall_quality);

        // Create metrics
        let metrics = ThreadQualityMetrics {
            thread_id: thread_id.to_string(),
            overall_quality,
            circular_reasoning_score: circular_score,
            distractor_fixation_score: distractor_score,
            quality_degradation_score: degradation_score,
            intervention_count,
            last_updated: Utc::now(),
            quality_trend: trend,
        };

        // Store metrics
        let mut cache = self.metrics_cache.lock().unwrap();
        cache.insert(thread_id.to_string(), metrics.clone());

        // Add to history
        let snapshot = QualitySnapshot {
            timestamp: Utc::now(),
            overall_quality,
            tool_name: tool_name.to_string(),
            intervention_triggered: intervention_count > 0,
        };

        let mut history = self.quality_history.lock().unwrap();
        history
            .entry(thread_id.to_string())
            .or_insert_with(Vec::new)
            .push(snapshot);

        // Update thread context with quality metrics
        self.update_thread_context(thread_id, &metrics)?;

        info!(
            "Updated quality metrics for thread {}: overall={:.2}, trend={:?}",
            thread_id, overall_quality, metrics.quality_trend
        );

        Ok(())
    }

    /// Calculate circular reasoning score (0.0 = no circular, 1.0 = highly circular)
    fn calculate_circular_score(&self, monitor: &MetacognitiveMonitor) -> f32 {
        // In a real implementation, this would analyze the monitor's circular reasoning detector
        // For now, return a placeholder based on intervention count
        let interventions = monitor.get_intervention_count();
        (interventions as f32 * 0.2).min(1.0)
    }

    /// Calculate distractor fixation score (0.0 = focused, 1.0 = highly distracted)
    fn calculate_distractor_score(&self, monitor: &MetacognitiveMonitor) -> f32 {
        // In a real implementation, this would analyze the monitor's distractor detector
        // For now, return a placeholder
        0.15
    }

    /// Calculate quality degradation score (0.0 = high quality, 1.0 = degraded)
    fn calculate_degradation_score(&self, monitor: &MetacognitiveMonitor) -> f32 {
        // In a real implementation, this would analyze the monitor's quality detector
        // For now, return a placeholder
        0.10
    }

    /// Calculate overall quality (0.0 = poor, 1.0 = excellent)
    fn calculate_overall_quality(&self, circular: f32, distractor: f32, degradation: f32) -> f32 {
        // Invert scores (since high circular/distractor/degradation is bad)
        let quality = 1.0 - ((circular + distractor + degradation) / 3.0);
        quality.max(0.0).min(1.0)
    }

    /// Determine quality trend based on history
    fn determine_trend(&self, thread_id: &str, current_quality: f32) -> QualityTrend {
        let history = self.quality_history.lock().unwrap();

        if let Some(snapshots) = history.get(thread_id) {
            if snapshots.len() < 2 {
                return QualityTrend::Unknown;
            }

            // Compare with average of last 3 snapshots
            let recent_count = snapshots.len().min(3);
            let recent_avg: f32 = snapshots
                .iter()
                .rev()
                .take(recent_count)
                .map(|s| s.overall_quality)
                .sum::<f32>()
                / recent_count as f32;

            let diff = current_quality - recent_avg;

            if diff > 0.1 {
                QualityTrend::Improving
            } else if diff < -0.1 {
                QualityTrend::Degrading
            } else {
                QualityTrend::Stable
            }
        } else {
            QualityTrend::Unknown
        }
    }

    /// Update thread context with quality metrics
    fn update_thread_context(&self, thread_id: &str, metrics: &ThreadQualityMetrics) -> Result<()> {
        // Create quality metrics for thread context
        let quality = QualityMetrics {
            circular_reasoning_score: metrics.circular_reasoning_score as f32,
            distractor_fixation_score: metrics.distractor_fixation_score as f32,
            coherence_score: metrics.overall_quality as f32,
            depth_score: 0.5,
            perplexity: 20.0,
        };

        // Update thread metadata
        // TODO: Implement update_thread_metadata in ThreadManager
        // self.thread_manager.update_thread_metadata(
        //     thread_id,
        //     "quality_metrics",
        //     &serde_json::to_value(metrics)?,
        // )?;

        // Update thread quality in context
        // TODO: Parse thread_id to Uuid and update
        // if let Some(mut context) = self.thread_manager.get_thread(thread_id) {
        //     context.quality_metrics = Some(quality);
        //     // Note: In a real implementation, we'd update the thread in the manager
        // }

        Ok(())
    }

    /// Get quality metrics for a thread
    pub fn get_thread_quality(&self, thread_id: &str) -> Option<ThreadQualityMetrics> {
        let cache = self.metrics_cache.lock().unwrap();
        cache.get(thread_id).cloned()
    }

    /// Get quality history for a thread
    pub fn get_quality_history(&self, thread_id: &str) -> Vec<QualitySnapshot> {
        let history = self.quality_history.lock().unwrap();
        history.get(thread_id).cloned().unwrap_or_default()
    }

    /// Check if intervention is needed based on quality
    pub fn needs_intervention(&self, thread_id: &str) -> bool {
        if let Some(metrics) = self.get_thread_quality(thread_id) {
            // Trigger intervention if quality is poor or degrading rapidly
            metrics.overall_quality < 0.4
                || matches!(metrics.quality_trend, QualityTrend::Degrading)
        } else {
            false
        }
    }

    /// Generate quality report for a thread
    pub fn generate_quality_report(&self, thread_id: &str) -> Option<String> {
        let metrics = self.get_thread_quality(thread_id)?;
        let history = self.get_quality_history(thread_id);

        Some(format!(
            "Quality Report for Thread {}\n\
             ================================\n\
             Overall Quality: {:.2}/1.0 ({})\n\
             Circular Reasoning: {:.2}\n\
             Distractor Fixation: {:.2}\n\
             Quality Degradation: {:.2}\n\
             Interventions: {}\n\
             Trend: {:?}\n\
             History: {} snapshots\n\
             Last Updated: {}",
            thread_id,
            metrics.overall_quality,
            if metrics.overall_quality > 0.7 {
                "Good"
            } else if metrics.overall_quality > 0.4 {
                "Fair"
            } else {
                "Poor"
            },
            metrics.circular_reasoning_score,
            metrics.distractor_fixation_score,
            metrics.quality_degradation_score,
            metrics.intervention_count,
            metrics.quality_trend,
            history.len(),
            metrics.last_updated.format("%Y-%m-%d %H:%M:%S UTC")
        ))
    }
}

/// Integration helper for quality monitoring
pub struct QualityThreadIntegration {
    quality_manager: Arc<ThreadQualityManager>,
    thread_manager: Arc<ThreadManager>,
}

impl QualityThreadIntegration {
    pub fn new(thread_manager: Arc<ThreadManager>) -> Self {
        let quality_manager = Arc::new(ThreadQualityManager::new(thread_manager.clone()));
        Self {
            quality_manager,
            thread_manager,
        }
    }

    /// Process quality metrics for a tool call
    pub fn process_quality(
        &self,
        thread_id: &str,
        monitor: &MetacognitiveMonitor,
        tool_name: &str,
    ) -> Result<()> {
        self.quality_manager
            .update_from_monitor(thread_id, monitor, tool_name)?;

        // Check if intervention is needed
        if self.quality_manager.needs_intervention(thread_id) {
            warn!("Quality intervention needed for thread {}", thread_id);
            // In a real implementation, this could trigger automatic interventions
        }

        Ok(())
    }

    /// Get quality context for reconstruction
    pub fn get_quality_context(&self, thread_id: &str) -> Option<String> {
        self.quality_manager.generate_quality_report(thread_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_manager_creation() {
        let thread_manager = Arc::new(ThreadManager::new());
        let quality_manager = ThreadQualityManager::new(thread_manager);
        assert!(quality_manager.metrics_cache.lock().is_ok());
    }

    #[test]
    fn test_quality_trend_detection() {
        let thread_manager = Arc::new(ThreadManager::new());
        let quality_manager = ThreadQualityManager::new(thread_manager);

        // Test unknown trend for new thread
        let trend = quality_manager.determine_trend("test-thread", 0.8);
        assert!(matches!(trend, QualityTrend::Unknown));
    }
}
