use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSignals {
    pub circular_score: f64,
    pub distractor_alert: bool,
    pub quality_trend: String,
    pub phase: String,
    pub intervention: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringStatus {
    pub cognitive_load: f64,
    pub current_phase: String,
    pub circular_reasoning_score: f64,
    pub distractor_fixation_score: f64,
    pub quality_metrics: QualityMetrics,
    pub intervention_history: Vec<InterventionRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub coherence: f64,
    pub information_density: f64,
    pub relevance: f64,
    pub trend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionRecord {
    pub thought_number: usize,
    pub intervention_type: String,
    pub reason: String,
}

pub struct MetacognitiveMonitor {
    thought_history: VecDeque<String>,
    intervention_history: Vec<InterventionRecord>,
    quality_scores: VecDeque<f64>,
}

impl MetacognitiveMonitor {
    pub fn new() -> Self {
        Self {
            thought_history: VecDeque::with_capacity(10),
            intervention_history: Vec::new(),
            quality_scores: VecDeque::with_capacity(5),
        }
    }
    
    pub fn reset_session(&mut self) {
        self.thought_history.clear();
        self.intervention_history.clear();
        self.quality_scores.clear();
    }

    pub fn analyze_thought(&mut self, thought: &str, thought_number: usize) -> MonitoringSignals {
        // Add to history
        self.thought_history.push_back(thought.to_string());
        if self.thought_history.len() > 10 {
            self.thought_history.pop_front();
        }

        // Calculate metrics
        let circular_score = self.detect_circular_reasoning(thought);
        let distractor_alert = self.detect_distractor_fixation(thought);
        let quality_trend = self.assess_quality_trend(thought);
        
        // Determine phase and intervention
        let (phase, intervention) = self.determine_intervention(
            circular_score,
            distractor_alert,
            &quality_trend,
            thought_number,
        );

        MonitoringSignals {
            circular_score,
            distractor_alert,
            quality_trend,
            phase,
            intervention,
        }
    }

    fn detect_circular_reasoning(&self, current_thought: &str) -> f64 {
        // Simplified circular reasoning detection
        // In a real implementation, this would use embeddings and semantic similarity
        let mut similarity_score = 0.0;
        let mut count = 0;

        for past_thought in self.thought_history.iter().rev().take(5) {
            let common_words = self.count_common_words(current_thought, past_thought);
            let total_words = current_thought.split_whitespace().count() + 
                             past_thought.split_whitespace().count();
            
            if total_words > 0 {
                similarity_score += (common_words * 2) as f64 / total_words as f64;
                count += 1;
            }
        }

        if count > 0 {
            similarity_score / count as f64
        } else {
            0.0
        }
    }

    fn detect_distractor_fixation(&self, thought: &str) -> bool {
        // Simplified distractor detection
        // Check if the thought contains too many details or tangential topics
        let word_count = thought.split_whitespace().count();
        let detail_words = ["specifically", "particularly", "detail", "minor", "tangent"];
        let detail_count = detail_words.iter()
            .filter(|&word| thought.to_lowercase().contains(word))
            .count();
        
        // Alert if high detail density or very long thoughts
        detail_count >= 2 || word_count > 150
    }

    fn assess_quality_trend(&mut self, thought: &str) -> String {
        // Simple quality assessment based on thought length and structure
        let quality_score = self.calculate_quality_score(thought);
        
        self.quality_scores.push_back(quality_score);
        if self.quality_scores.len() > 5 {
            self.quality_scores.pop_front();
        }

        // Determine trend
        if self.quality_scores.len() >= 3 {
            let recent: Vec<_> = self.quality_scores.iter().cloned().collect();
            let avg_first_half = recent[..recent.len()/2].iter().sum::<f64>() / (recent.len()/2) as f64;
            let avg_second_half = recent[recent.len()/2..].iter().sum::<f64>() / (recent.len() - recent.len()/2) as f64;
            
            if avg_second_half < avg_first_half * 0.8 {
                "declining".to_string()
            } else if avg_second_half > avg_first_half * 1.2 {
                "improving".to_string()
            } else {
                "stable".to_string()
            }
        } else {
            "stable".to_string()
        }
    }

    fn calculate_quality_score(&self, thought: &str) -> f64 {
        let word_count = thought.split_whitespace().count();
        let sentence_count = thought.matches('.').count() + thought.matches('!').count() + thought.matches('?').count();
        
        // Basic quality heuristics
        let mut score = 0.5;
        
        // Penalize very short or very long thoughts
        if word_count < 10 {
            score -= 0.2;
        } else if word_count > 200 {
            score -= 0.1;
        }
        
        // Reward structured thinking
        if sentence_count > 1 && sentence_count < word_count / 10 {
            score += 0.2;
        }
        
        // Check for reasoning indicators
        let reasoning_words = ["because", "therefore", "thus", "hence", "considering"];
        let reasoning_count = reasoning_words.iter()
            .filter(|&word| thought.to_lowercase().contains(word))
            .count();
        
        score += (reasoning_count as f64 * 0.1).min(0.3);
        
        score.clamp(0.0, 1.0)
    }

    fn determine_intervention(
        &mut self,
        circular_score: f64,
        distractor_alert: bool,
        quality_trend: &str,
        thought_number: usize,
    ) -> (String, Option<String>) {
        let mut phase = "exploration".to_string();
        let mut intervention = None;

        if circular_score > 0.85 {
            phase = "overthinking".to_string();
            intervention = Some("Consider breaking out of this loop with a new perspective.".to_string());
            self.intervention_history.push(InterventionRecord {
                thought_number,
                intervention_type: "circular_reasoning".to_string(),
                reason: format!("High circular score: {:.2}", circular_score),
            });
        } else if distractor_alert {
            phase = "distracted".to_string();
            intervention = Some("Refocus on the core problem statement.".to_string());
            self.intervention_history.push(InterventionRecord {
                thought_number,
                intervention_type: "distractor_fixation".to_string(),
                reason: "Excessive detail or tangential content detected".to_string(),
            });
        } else if quality_trend == "declining" {
            phase = "fatigue".to_string();
            intervention = Some("Quality declining. Consider consolidating insights and concluding.".to_string());
            self.intervention_history.push(InterventionRecord {
                thought_number,
                intervention_type: "quality_degradation".to_string(),
                reason: "Consistent decline in reasoning quality".to_string(),
            });
        }

        (phase, intervention)
    }

    fn count_common_words(&self, text1: &str, text2: &str) -> usize {
        let words1: std::collections::HashSet<_> = text1.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        let words2: std::collections::HashSet<_> = text2.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        
        words1.intersection(&words2).count()
    }

    pub fn get_status(&self) -> MonitoringStatus {
        let latest_quality = self.quality_scores.back().cloned().unwrap_or(0.5);
        let quality_trend = if self.quality_scores.len() >= 3 {
            // Calculate trend without mutating
            let recent: Vec<_> = self.quality_scores.iter().cloned().collect();
            let avg_first_half = recent[..recent.len()/2].iter().sum::<f64>() / (recent.len()/2) as f64;
            let avg_second_half = recent[recent.len()/2..].iter().sum::<f64>() / (recent.len() - recent.len()/2) as f64;
            
            if avg_second_half < avg_first_half * 0.8 {
                "declining".to_string()
            } else if avg_second_half > avg_first_half * 1.2 {
                "improving".to_string()
            } else {
                "stable".to_string()
            }
        } else {
            "insufficient_data".to_string()
        };

        MonitoringStatus {
            cognitive_load: self.calculate_cognitive_load(),
            current_phase: self.determine_current_phase(),
            circular_reasoning_score: 0.0, // Would be calculated from recent thoughts
            distractor_fixation_score: 0.0, // Would be calculated from recent thoughts
            quality_metrics: QualityMetrics {
                coherence: latest_quality,
                information_density: 0.5, // Placeholder
                relevance: 0.8, // Placeholder
                trend: quality_trend,
            },
            intervention_history: self.intervention_history.clone(),
        }
    }

    fn calculate_cognitive_load(&self) -> f64 {
        // Simple cognitive load based on thought complexity and history
        let thought_count = self.thought_history.len() as f64;
        let intervention_count = self.intervention_history.len() as f64;
        
        ((thought_count / 10.0) + (intervention_count / 5.0)).min(1.0)
    }

    fn determine_current_phase(&self) -> String {
        if self.intervention_history.is_empty() {
            "exploration".to_string()
        } else {
            match self.intervention_history.last().unwrap().intervention_type.as_str() {
                "circular_reasoning" => "overthinking".to_string(),
                "distractor_fixation" => "distracted".to_string(),
                "quality_degradation" => "fatigue".to_string(),
                _ => "development".to_string(),
            }
        }
    }
}