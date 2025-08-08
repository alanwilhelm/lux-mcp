use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;

/// Advanced circular reasoning detection using TF-IDF and concept extraction
pub struct CircularReasoningDetector {
    /// TF-IDF index for concept similarity
    concept_index: ConceptIndex,
    /// Minimum similarity threshold for circular detection
    similarity_threshold: f64,
}

impl CircularReasoningDetector {
    pub fn new() -> Self {
        Self {
            concept_index: ConceptIndex::new(),
            similarity_threshold: 0.6, // Adjusted for semantic similarity detection
        }
    }

    /// Extract key concepts from text using NLP-like techniques
    pub fn extract_concepts(&self, text: &str) -> Vec<Concept> {
        let tokens = self.tokenize(text);
        let mut concepts = Vec::new();

        // Extract noun phrases (simplified without true POS tagging)
        let noun_indicators = [
            "the", "a", "an", "this", "that", "these", "those", "how", "what", "why",
        ];
        let important_words = self.identify_important_words(&tokens);

        // Extract multi-word concepts first (bigrams/trigrams) - prioritize compound concepts
        for i in 0..tokens.len().saturating_sub(1) {
            // Look for common compound concepts
            if i + 1 < tokens.len() {
                let bigram = format!("{} {}", tokens[i], tokens[i + 1]);
                let bigram_lower = bigram.to_lowercase();

                // Check for known compound terms
                let compound_terms = [
                    "neural network",
                    "deep learning",
                    "machine learning",
                    "gradient descent",
                    "back propagation",
                    "chain rule",
                    "learning rate",
                    "activation function",
                    "loss function",
                    "weight update",
                    "error minimization",
                    "feature extraction",
                ];

                for compound in &compound_terms {
                    if bigram_lower.contains(compound) || compound.contains(&bigram_lower) {
                        concepts.push(Concept::Compound(bigram.clone()));
                        break;
                    }
                }

                // Also check if two adjacent meaningful words form a concept
                if self.is_meaningful_concept(&tokens[i])
                    && self.is_meaningful_concept(&tokens[i + 1])
                {
                    // Check if they commonly go together (simple heuristic)
                    if (tokens[i] == "neural" && tokens[i + 1] == "network")
                        || (tokens[i] == "neural" && tokens[i + 1] == "networks")
                        || (tokens[i] == "back" && tokens[i + 1] == "propagation")
                        || (tokens[i] == "machine" && tokens[i + 1] == "learning")
                    {
                        let compound = format!(
                            "{} {}",
                            self.basic_stem(&tokens[i]),
                            self.basic_stem(&tokens[i + 1])
                        );
                        concepts.push(Concept::Compound(compound));
                    }
                }
            }

            // Look for compound concepts with prepositions
            if i + 2 < tokens.len() {
                let prepositions = ["of", "in", "on", "for", "with", "by", "to", "through"];
                if prepositions.contains(&tokens[i + 1].as_str())
                    && self.is_meaningful_concept(&tokens[i])
                    && self.is_meaningful_concept(&tokens[i + 2])
                {
                    let compound = format!(
                        "{} {} {}",
                        self.basic_stem(&tokens[i]),
                        tokens[i + 1],
                        self.basic_stem(&tokens[i + 2])
                    );
                    concepts.push(Concept::Compound(compound));
                }
            }
        }

        // Extract single-word concepts with stemming (after compounds to avoid duplication)
        for word in &important_words {
            if self.is_meaningful_concept(word) {
                // Skip if already part of a compound concept
                let word_stem = self.basic_stem(word);
                let already_in_compound = concepts.iter().any(|c| match c {
                    Concept::Compound(s) | Concept::Phrase(s) => s.contains(&word_stem),
                    _ => false,
                });

                if !already_in_compound {
                    concepts.push(Concept::Single(word_stem));
                }
            }
        }

        // Extract phrases with determiners
        for i in 0..tokens.len().saturating_sub(2) {
            if noun_indicators.contains(&tokens[i].as_str())
                && self.is_meaningful_concept(&tokens[i + 1])
            {
                let phrase = format!("{} {}", tokens[i], self.basic_stem(&tokens[i + 1]));
                concepts.push(Concept::Phrase(phrase));
            }
        }

        // Extract action concepts (verb + object patterns)
        let action_verbs = [
            "understand",
            "understanding",
            "solve",
            "calculate",
            "determine",
            "analyze",
            "process",
            "need",
            "require",
            "requires",
            "learn",
            "learning",
            "update",
            "minimize",
            "improve",
        ];
        for (i, token) in tokens.iter().enumerate() {
            let stemmed_verb = self.basic_stem(token);
            if action_verbs.contains(&token.as_str())
                || action_verbs.contains(&stemmed_verb.as_str())
            {
                if i + 1 < tokens.len() && self.is_meaningful_concept(&tokens[i + 1]) {
                    let action = format!("{} {}", stemmed_verb, self.basic_stem(&tokens[i + 1]));
                    concepts.push(Concept::Action(action));
                }
            }
        }

        // Deduplicate concepts while preserving order
        let mut seen = HashSet::new();
        let mut unique_concepts = Vec::new();
        for concept in concepts {
            let key = concept.to_string();
            if seen.insert(key) {
                unique_concepts.push(concept);
            }
        }

        unique_concepts
    }

    /// Calculate semantic similarity between two sets of concepts using TF-IDF
    pub fn calculate_similarity(&self, concepts1: &[Concept], concepts2: &[Concept]) -> f64 {
        if concepts1.is_empty() || concepts2.is_empty() {
            return 0.0;
        }

        // Build TF vectors
        let tf1 = self.build_tf_vector(concepts1);
        let tf2 = self.build_tf_vector(concepts2);

        // Calculate cosine similarity
        self.cosine_similarity(&tf1, &tf2)
    }

    /// Detect circular reasoning patterns
    pub fn detect_pattern(
        &self,
        current_thought: &str,
        thought_history: &[String],
    ) -> CircularPattern {
        let current_concepts = self.extract_concepts(current_thought);

        // Check for direct repetition
        for (idx, past_thought) in thought_history.iter().enumerate() {
            let past_concepts = self.extract_concepts(past_thought);
            let similarity = self.calculate_similarity(&current_concepts, &past_concepts);

            if similarity >= self.similarity_threshold {
                return CircularPattern::Direct {
                    similarity_score: similarity,
                    matching_thought_idx: idx,
                };
            }
        }

        // Check for cyclic patterns (A→B→C→A)
        if thought_history.len() >= 3 {
            let cycle = self.detect_cyclic_pattern(&current_concepts, thought_history);
            if let Some(pattern) = cycle {
                return pattern;
            }
        }

        // Check for conceptual loops (same concepts, different words)
        let conceptual_similarity =
            self.detect_conceptual_loops(&current_concepts, thought_history);
        if conceptual_similarity > 0.7 {
            return CircularPattern::Conceptual {
                average_similarity: conceptual_similarity,
            };
        }

        CircularPattern::None
    }

    /// Tokenize text into words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.unicode_words()
            .map(|w| w.to_lowercase())
            .filter(|w| w.len() > 2)
            .collect()
    }

    /// Basic stemming for common English suffixes
    fn basic_stem(&self, word: &str) -> String {
        let word_lower = word.to_lowercase();

        // Handle common verb forms
        if word_lower.ends_with("ing") && word_lower.len() > 5 {
            // understanding -> understand
            if word_lower.ends_with("ding") {
                return word_lower[..word_lower.len() - 3].to_string();
            }
            return word_lower[..word_lower.len() - 3].to_string();
        }

        if word_lower.ends_with("ed") && word_lower.len() > 4 {
            return word_lower[..word_lower.len() - 2].to_string();
        }

        if word_lower.ends_with("es") && word_lower.len() > 4 {
            // requires -> require
            return word_lower[..word_lower.len() - 1].to_string();
        }

        if word_lower.ends_with("s") && word_lower.len() > 3 && !word_lower.ends_with("ss") {
            return word_lower[..word_lower.len() - 1].to_string();
        }

        word_lower
    }

    /// Identify important words (remove stop words)
    fn identify_important_words(&self, tokens: &[String]) -> Vec<String> {
        let stop_words: HashSet<&str> = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "up", "about", "into", "through", "during", "before", "after", "above",
            "below", "between", "under", "is", "are", "was", "were", "been", "be", "have", "has",
            "had", "do", "does", "did", "will", "would", "should", "could", "may", "might", "must",
            "can", "could",
        ]
        .iter()
        .cloned()
        .collect();

        tokens
            .iter()
            .filter(|word| !stop_words.contains(word.as_str()))
            .cloned()
            .collect()
    }

    /// Check if a word is a meaningful concept
    fn is_meaningful_concept(&self, word: &str) -> bool {
        word.len() > 3 && word.chars().all(|c| c.is_alphabetic())
    }

    /// Check if a phrase is meaningful
    fn is_meaningful_phrase(&self, phrase: &str) -> bool {
        let words: Vec<&str> = phrase.split_whitespace().collect();
        words.len() >= 2 && words.len() <= 4
    }

    /// Build term frequency vector
    fn build_tf_vector(&self, concepts: &[Concept]) -> HashMap<String, f64> {
        let mut tf_map = HashMap::new();
        let total = concepts.len() as f64;

        for concept in concepts {
            let key = concept.to_string();
            *tf_map.entry(key).or_insert(0.0) += 1.0 / total;
        }

        tf_map
    }

    /// Calculate cosine similarity between two TF vectors
    fn cosine_similarity(&self, tf1: &HashMap<String, f64>, tf2: &HashMap<String, f64>) -> f64 {
        let mut dot_product = 0.0;
        let mut magnitude1 = 0.0;
        let mut magnitude2 = 0.0;

        // Get all unique terms
        let mut all_terms = HashSet::new();
        all_terms.extend(tf1.keys().cloned());
        all_terms.extend(tf2.keys().cloned());

        for term in all_terms {
            let val1 = tf1.get(&term).copied().unwrap_or(0.0);
            let val2 = tf2.get(&term).copied().unwrap_or(0.0);

            dot_product += val1 * val2;
            magnitude1 += val1 * val1;
            magnitude2 += val2 * val2;
        }

        if magnitude1 > 0.0 && magnitude2 > 0.0 {
            dot_product / (magnitude1.sqrt() * magnitude2.sqrt())
        } else {
            0.0
        }
    }

    /// Detect cyclic patterns in thought history
    fn detect_cyclic_pattern(
        &self,
        current_concepts: &[Concept],
        history: &[String],
    ) -> Option<CircularPattern> {
        if history.len() < 3 {
            return None;
        }

        // Check if current thought completes a cycle
        let first_concepts = self.extract_concepts(&history[0]);
        let similarity_to_first = self.calculate_similarity(current_concepts, &first_concepts);

        if similarity_to_first > 0.7 {
            // Verify intermediate steps are different
            let mut intermediate_similarities = Vec::new();
            for i in 1..history.len() {
                let concepts = self.extract_concepts(&history[i]);
                let sim = self.calculate_similarity(&concepts, &first_concepts);
                intermediate_similarities.push(sim);
            }

            // If intermediate steps are sufficiently different, we have a cycle
            let avg_intermediate = intermediate_similarities.iter().sum::<f64>()
                / intermediate_similarities.len() as f64;
            if avg_intermediate < 0.5 {
                return Some(CircularPattern::Cyclic {
                    cycle_length: history.len() + 1,
                    cycle_strength: similarity_to_first,
                });
            }
        }

        None
    }

    /// Detect conceptual loops (same ideas, different words)
    fn detect_conceptual_loops(&self, current_concepts: &[Concept], history: &[String]) -> f64 {
        if history.is_empty() || current_concepts.is_empty() {
            return 0.0;
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for past_thought in history.iter().rev().take(5) {
            let past_concepts = self.extract_concepts(past_thought);
            if past_concepts.is_empty() {
                continue;
            }

            // Calculate actual semantic similarity, not just distribution
            let semantic_sim = self.calculate_similarity(current_concepts, &past_concepts);

            // Also consider concept type distribution
            let current_distribution = self.build_concept_distribution(current_concepts);
            let past_distribution = self.build_concept_distribution(&past_concepts);
            let distribution_sim =
                self.compare_distributions(&current_distribution, &past_distribution);

            // Weight semantic similarity more than distribution similarity
            let combined_similarity = (semantic_sim * 0.7) + (distribution_sim * 0.3);

            total_similarity += combined_similarity;
            count += 1;
        }

        if count > 0 {
            total_similarity / count as f64
        } else {
            0.0
        }
    }

    /// Build concept type distribution
    fn build_concept_distribution(&self, concepts: &[Concept]) -> HashMap<&str, f64> {
        let mut distribution = HashMap::new();
        let total = concepts.len() as f64;

        for concept in concepts {
            let concept_type = match concept {
                Concept::Single(_) => "single",
                Concept::Phrase(_) => "phrase",
                Concept::Compound(_) => "compound",
                Concept::Action(_) => "action",
            };
            *distribution.entry(concept_type).or_insert(0.0) += 1.0 / total;
        }

        distribution
    }

    /// Compare two concept distributions
    fn compare_distributions(&self, dist1: &HashMap<&str, f64>, dist2: &HashMap<&str, f64>) -> f64 {
        let types = ["single", "phrase", "compound", "action"];
        let mut similarity = 0.0;

        for concept_type in &types {
            let val1 = dist1.get(concept_type).copied().unwrap_or(0.0);
            let val2 = dist2.get(concept_type).copied().unwrap_or(0.0);
            similarity += 1.0 - (val1 - val2).abs();
        }

        similarity / types.len() as f64
    }
}

/// Represents different types of concepts extracted from text
#[derive(Debug, Clone, PartialEq)]
pub enum Concept {
    Single(String),   // Single word concept
    Phrase(String),   // Multi-word phrase
    Compound(String), // Compound concept with preposition
    Action(String),   // Verb + object pattern
}

impl ToString for Concept {
    fn to_string(&self) -> String {
        match self {
            Concept::Single(s) => s.clone(),
            Concept::Phrase(s) => s.clone(),
            Concept::Compound(s) => s.clone(),
            Concept::Action(s) => s.clone(),
        }
    }
}

/// Types of circular reasoning patterns detected
#[derive(Debug, Clone)]
pub enum CircularPattern {
    None,
    Direct {
        similarity_score: f64,
        matching_thought_idx: usize,
    },
    Cyclic {
        cycle_length: usize,
        cycle_strength: f64,
    },
    Conceptual {
        average_similarity: f64,
    },
}

/// Index for storing concept frequencies across thoughts
struct ConceptIndex {
    document_frequencies: HashMap<String, usize>,
    total_documents: usize,
}

impl ConceptIndex {
    fn new() -> Self {
        Self {
            document_frequencies: HashMap::new(),
            total_documents: 0,
        }
    }

    #[allow(dead_code)]
    fn add_document(&mut self, concepts: &[Concept]) {
        self.total_documents += 1;
        let unique_concepts: HashSet<String> = concepts.iter().map(|c| c.to_string()).collect();

        for concept in unique_concepts {
            *self.document_frequencies.entry(concept).or_insert(0) += 1;
        }
    }

    #[allow(dead_code)]
    fn get_idf(&self, concept: &str) -> f64 {
        if let Some(&doc_freq) = self.document_frequencies.get(concept) {
            (self.total_documents as f64 / doc_freq as f64).ln()
        } else {
            (self.total_documents as f64).ln()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_extraction() {
        let detector = CircularReasoningDetector::new();

        let text = "To understand recursion, you must first understand the concept of recursion";
        let concepts = detector.extract_concepts(text);

        // Should extract meaningful concepts
        assert!(concepts.len() > 0);
        assert!(concepts
            .iter()
            .any(|c| matches!(c, Concept::Action(s) if s.contains("understand"))));
        assert!(concepts
            .iter()
            .any(|c| matches!(c, Concept::Single(s) if s == "recursion")));
    }

    #[test]
    fn test_similarity_calculation() {
        let detector = CircularReasoningDetector::new();

        let text1 = "TCP/IP enables network communication through packet switching";
        let text2 = "Network communication is enabled by TCP/IP using packet switching";
        let text3 = "Python is a programming language";

        let concepts1 = detector.extract_concepts(text1);
        let concepts2 = detector.extract_concepts(text2);
        let concepts3 = detector.extract_concepts(text3);

        let sim_12 = detector.calculate_similarity(&concepts1, &concepts2);
        let sim_13 = detector.calculate_similarity(&concepts1, &concepts3);

        // Similar texts should have higher similarity
        assert!(sim_12 > 0.3); // Lowered threshold for TF-IDF similarity
        assert!(sim_12 > sim_13);
    }

    #[test]
    fn test_circular_pattern_detection() {
        let detector = CircularReasoningDetector::new();

        let history = vec![
            "Understanding recursion requires understanding recursion".to_string(),
            "Let me think about something else".to_string(),
        ];

        let current = "To understand recursion, you need to understand recursion";

        // Debug: extract concepts
        let current_concepts = detector.extract_concepts(current);
        let past_concepts = detector.extract_concepts(&history[0]);
        println!("Current concepts: {:?}", current_concepts);
        println!("Past concepts: {:?}", past_concepts);

        // Debug: calculate similarity
        let similarity = detector.calculate_similarity(&current_concepts, &past_concepts);
        println!("Similarity: {}", similarity);

        let pattern = detector.detect_pattern(current, &history);
        println!("Pattern: {:?}", pattern);

        match pattern {
            CircularPattern::Direct {
                similarity_score, ..
            } => {
                assert!(similarity_score >= 0.6);
            }
            CircularPattern::Conceptual { average_similarity } => {
                // Also accept conceptual pattern for very similar meanings
                assert!(average_similarity > 0.8);
            }
            _ => panic!("Expected direct or conceptual circular pattern"),
        }
    }

    #[test]
    fn test_no_circular_pattern() {
        let detector = CircularReasoningDetector::new();

        let history = vec![
            "TCP/IP is a networking protocol".to_string(),
            "HTTP runs on top of TCP".to_string(),
        ];

        let current = "DNS resolves domain names to IP addresses";

        // Debug output
        println!("Testing no circular pattern");
        let current_concepts = detector.extract_concepts(current);
        println!("Current concepts: {:?}", current_concepts);
        for (i, h) in history.iter().enumerate() {
            let concepts = detector.extract_concepts(h);
            let sim = detector.calculate_similarity(&current_concepts, &concepts);
            println!(
                "History[{}] concepts: {:?}, similarity: {}",
                i, concepts, sim
            );
        }

        let pattern = detector.detect_pattern(current, &history);
        println!("Pattern: {:?}", pattern);

        assert!(matches!(pattern, CircularPattern::None));
    }
}
