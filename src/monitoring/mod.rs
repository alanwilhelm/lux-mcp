use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

mod circular_reasoning;
use circular_reasoning::{CircularReasoningDetector, CircularPattern};

mod distractor_fixation;
use distractor_fixation::{DistractorFixationDetector, DistractorPattern};

mod quality_degradation;
use quality_degradation::{QualityDegradationDetector, DegradationAnalysis, DegradationPattern};

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
    circular_detector: CircularReasoningDetector,
    distractor_detector: DistractorFixationDetector,
    quality_degradation_detector: QualityDegradationDetector,
}

impl MetacognitiveMonitor {
    pub fn new() -> Self {
        Self {
            thought_history: VecDeque::with_capacity(10),
            intervention_history: Vec::new(),
            quality_scores: VecDeque::with_capacity(5),
            circular_detector: CircularReasoningDetector::new(),
            distractor_detector: DistractorFixationDetector::new(),
            quality_degradation_detector: QualityDegradationDetector::new(),
        }
    }
    
    pub fn reset_session(&mut self) {
        self.thought_history.clear();
        self.intervention_history.clear();
        self.quality_scores.clear();
        self.distractor_detector.reset_session();
        self.quality_degradation_detector.reset_session();
    }

    pub fn analyze_thought(&mut self, thought: &str, thought_number: usize) -> MonitoringSignals {
        // Set original query on first substantial thought
        if !self.distractor_detector.has_original_concepts() && !thought.trim().is_empty() {
            self.distractor_detector.set_original_query(thought);
        }
        
        // Calculate metrics BEFORE adding to history
        let circular_score = self.detect_circular_reasoning(thought);
        let distractor_alert = self.detect_distractor_fixation(thought);
        
        // Use advanced quality degradation analysis
        let degradation_analysis = self.quality_degradation_detector.analyze_thought(thought, thought_number);
        let quality_trend = self.determine_quality_trend(&degradation_analysis);
        
        // Add to history AFTER calculating metrics
        self.thought_history.push_back(thought.to_string());
        if self.thought_history.len() > 10 {
            self.thought_history.pop_front();
        }
        
        // Store quality score for backward compatibility
        let quality_score = 1.0 - degradation_analysis.overall_score;
        self.quality_scores.push_back(quality_score);
        if self.quality_scores.len() > 5 {
            self.quality_scores.pop_front();
        }
        
        // Determine phase and intervention with degradation analysis
        let (phase, intervention) = self.determine_intervention_with_degradation(
            circular_score,
            distractor_alert,
            &degradation_analysis,
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
        // Use advanced circular reasoning detection
        if self.thought_history.is_empty() {
            return 0.0;
        }
        
        // Convert VecDeque to Vec for the detector
        let history: Vec<String> = self.thought_history.iter().cloned().collect();
        
        // Detect circular patterns
        let pattern = self.circular_detector.detect_pattern(current_thought, &history);
        
        // Convert pattern to score
        match pattern {
            CircularPattern::None => 0.0,
            CircularPattern::Direct { similarity_score, .. } => similarity_score,
            CircularPattern::Cyclic { cycle_strength, .. } => cycle_strength * 0.9, // Slightly lower weight for cycles
            CircularPattern::Conceptual { average_similarity } => average_similarity * 0.8, // Lower weight for conceptual loops
        }
    }

    fn detect_distractor_fixation(&mut self, thought: &str) -> bool {
        // Use advanced distractor fixation detection
        let (is_distracted, _pattern) = self.distractor_detector.detect_fixation(thought);
        is_distracted
    }

    fn determine_quality_trend(&self, analysis: &DegradationAnalysis) -> String {
        // Determine trend based on degradation patterns
        if analysis.degradation_patterns.is_empty() {
            return "stable".to_string();
        }
        
        // Check for severe degradation patterns
        let has_severe = analysis.degradation_patterns.iter().any(|p| {
            match p {
                DegradationPattern::CognitiveFatigue { .. } => true,
                DegradationPattern::ReasoningSimplification { depth_decline, .. } => depth_decline > &0.4,
                _ => false,
            }
        });
        
        if has_severe || analysis.overall_score > 0.6 {
            "degrading".to_string()
        } else if analysis.overall_score > 0.3 {
            "declining".to_string()
        } else {
            "stable".to_string()
        }
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
                "degrading".to_string()
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

    fn determine_intervention_with_degradation(
        &mut self,
        circular_score: f64,
        distractor_alert: bool,
        degradation_analysis: &DegradationAnalysis,
        thought_number: usize,
    ) -> (String, Option<String>) {
        let mut phase = "exploration".to_string();
        let mut intervention = None;

        // Check circular reasoning first (highest priority)
        if circular_score > 0.5 {
            phase = "overthinking".to_string();
            intervention = Some("Consider breaking out of this loop with a new perspective.".to_string());
            self.intervention_history.push(InterventionRecord {
                thought_number,
                intervention_type: "circular_reasoning".to_string(),
                reason: format!("High circular score: {:.2}", circular_score),
            });
        } 
        // Check distractor fixation
        else if distractor_alert {
            phase = "distracted".to_string();
            intervention = Some("Refocus on the core problem statement.".to_string());
            self.intervention_history.push(InterventionRecord {
                thought_number,
                intervention_type: "distractor_fixation".to_string(),
                reason: "Excessive detail or tangential content detected".to_string(),
            });
        } 
        // Check quality degradation patterns
        else if !degradation_analysis.degradation_patterns.is_empty() {
            // Find the most severe pattern
            let most_severe = degradation_analysis.degradation_patterns.iter()
                .max_by_key(|p| match p {
                    DegradationPattern::CognitiveFatigue { .. } => 4,
                    DegradationPattern::ReasoningSimplification { .. } => 3,
                    DegradationPattern::CoherenceBreakdown { .. } => 2,
                    DegradationPattern::VocabularyDecline { .. } => 1,
                });
            
            if let Some(pattern) = most_severe {
                match pattern {
                    DegradationPattern::CognitiveFatigue { .. } => {
                        phase = "fatigue".to_string();
                        if let Some(recommendation) = degradation_analysis.recommendations.first() {
                            intervention = Some(recommendation.clone());
                        } else {
                            intervention = Some("Cognitive fatigue detected. Consider taking a break or concluding your analysis.".to_string());
                        }
                        self.intervention_history.push(InterventionRecord {
                            thought_number,
                            intervention_type: "cognitive_fatigue".to_string(),
                            reason: pattern.description(),
                        });
                    }
                    DegradationPattern::ReasoningSimplification { .. } => {
                        phase = "simplifying".to_string();
                        intervention = Some("Your reasoning is becoming simplified. Return to deeper analysis.".to_string());
                        self.intervention_history.push(InterventionRecord {
                            thought_number,
                            intervention_type: "reasoning_simplification".to_string(),
                            reason: pattern.description(),
                        });
                    }
                    DegradationPattern::CoherenceBreakdown { .. } => {
                        phase = "fragmenting".to_string();
                        intervention = Some("Your thoughts are becoming fragmented. Focus on clear connections between ideas.".to_string());
                        self.intervention_history.push(InterventionRecord {
                            thought_number,
                            intervention_type: "coherence_breakdown".to_string(),
                            reason: pattern.description(),
                        });
                    }
                    DegradationPattern::VocabularyDecline { .. } => {
                        phase = "repetitive".to_string();
                        intervention = Some("Your vocabulary is becoming repetitive. Try to express ideas with more variety.".to_string());
                        self.intervention_history.push(InterventionRecord {
                            thought_number,
                            intervention_type: "vocabulary_decline".to_string(),
                            reason: pattern.description(),
                        });
                    }
                }
            }
        }

        (phase, intervention)
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

        if circular_score > 0.5 {  // Lowered threshold to match updated detection
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
        } else if quality_trend == "degrading" {
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


    pub fn get_status(&self) -> MonitoringStatus {
        let latest_quality = self.quality_scores.back().cloned().unwrap_or(0.5);
        let quality_trend = if self.quality_scores.len() >= 3 {
            // Calculate trend without mutating
            let recent: Vec<_> = self.quality_scores.iter().cloned().collect();
            let avg_first_half = recent[..recent.len()/2].iter().sum::<f64>() / (recent.len()/2) as f64;
            let avg_second_half = recent[recent.len()/2..].iter().sum::<f64>() / (recent.len() - recent.len()/2) as f64;
            
            if avg_second_half < avg_first_half * 0.8 {
                "degrading".to_string()
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
        match self.intervention_history.last() {
            None => "exploration".to_string(),
            Some(last_intervention) => match last_intervention.intervention_type.as_str() {
                "circular_reasoning" => "overthinking".to_string(),
                "distractor_fixation" => "distracted".to_string(),
                "quality_degradation" => "fatigue".to_string(),
                _ => "development".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_circular_reasoning_detection() {
        let mut monitor = MetacognitiveMonitor::new();
        
        // First thought
        let signals1 = monitor.analyze_thought("Understanding recursion requires understanding recursion", 1);
        assert!(signals1.circular_score < 0.5); // First occurrence shouldn't be circular
        
        // Similar thought - should trigger circular reasoning
        let signals2 = monitor.analyze_thought("To understand recursion, you must understand recursion", 2);
        assert!(signals2.circular_score > 0.5); // Should detect similarity (adjusted for conceptual detection)
        assert!(signals2.intervention.is_some());
    }
    
    #[test]
    fn test_quality_degradation() {
        let mut monitor = MetacognitiveMonitor::new();
        
        // Good quality thought
        monitor.analyze_thought("TCP/IP is a protocol suite that enables reliable network communication through packet switching and hierarchical addressing", 1);
        
        // Degrading quality
        monitor.analyze_thought("TCP/IP is like, you know, network stuff", 2);
        monitor.analyze_thought("It's just networking", 3);
        
        let signals = monitor.analyze_thought("Network things", 4);
        assert_eq!(signals.quality_trend, "degrading");
    }
    
    #[test]
    fn test_session_reset() {
        let mut monitor = MetacognitiveMonitor::new();
        
        // Add some thoughts
        monitor.analyze_thought("First thought", 1);
        monitor.analyze_thought("Second thought", 2);
        
        // Reset session
        monitor.reset_session();
        
        // Same thought should not be circular after reset
        let signals = monitor.analyze_thought("First thought", 1);
        assert!(signals.circular_score < 0.5);
    }
}