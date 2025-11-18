//! Prompt quality and efficiency analysis
//!
//! Provides scoring algorithms for prompt quality and efficiency metrics.

use chrono::Utc;

use crate::config::QualityWeights;
use crate::models::{EfficiencyMetrics, Prompt, QualityScore};
use crate::Result;

/// Quality analyzer for prompts
pub struct QualityAnalyzer {
    weights: QualityWeights,
}

/// Efficiency analyzer for prompts
pub struct EfficiencyAnalyzer {
    /// Average tokens for normalization
    avg_tokens: f64,
    /// Average execution time for normalization
    avg_time_ms: f64,
    /// Average cost for normalization
    avg_cost: f64,
}

impl Default for QualityAnalyzer {
    fn default() -> Self {
        Self {
            weights: QualityWeights {
                clarity: 0.3,
                completeness: 0.3,
                specificity: 0.2,
                guidance: 0.2,
            },
        }
    }
}

impl QualityAnalyzer {
    /// Create analyzer with custom weights
    pub fn new(weights: QualityWeights) -> Self {
        Self { weights }
    }

    /// Analyze prompt quality and return score
    pub fn analyze(&self, prompt: &Prompt) -> Result<QualityScore> {
        let clarity = self.calculate_clarity(&prompt.content);
        let completeness = self.calculate_completeness(prompt);
        let specificity = self.calculate_specificity(&prompt.content);
        let guidance = self.calculate_guidance(prompt);

        // Calculate weighted total score
        let total_score = (clarity * self.weights.clarity
            + completeness * self.weights.completeness
            + specificity * self.weights.specificity
            + guidance * self.weights.guidance)
            * 100.0;

        Ok(QualityScore {
            prompt_id: prompt.id.clone(),
            total_score: total_score.clamp(0.0, 100.0),
            clarity: clarity * 100.0,
            completeness: completeness * 100.0,
            specificity: specificity * 100.0,
            guidance: guidance * 100.0,
            analyzed_at: Utc::now(),
        })
    }

    /// Calculate clarity score (0.0 - 1.0)
    /// Based on sentence structure, keyword density, and readability
    fn calculate_clarity(&self, content: &str) -> f64 {
        let mut score = 0.0;

        // Word count factor (optimal range: 10-100 words)
        let word_count = content.split_whitespace().count();
        let word_score = if word_count >= 10 && word_count <= 100 {
            1.0
        } else if word_count < 10 {
            word_count as f64 / 10.0
        } else {
            (200.0 - word_count as f64).max(0.0) / 100.0
        };
        score += word_score * 0.3;

        // Sentence structure (has proper sentences)
        let sentence_count = content.matches('.').count()
            + content.matches('?').count()
            + content.matches('!').count();
        let sentence_score = if sentence_count > 0 {
            (sentence_count as f64 / (word_count as f64 / 15.0).max(1.0)).min(1.0)
        } else {
            0.5 // Single instruction is okay
        };
        score += sentence_score * 0.3;

        // Action words (verbs that indicate clear intent)
        let action_words = [
            "create", "write", "implement", "build", "design", "analyze",
            "explain", "describe", "list", "compare", "convert", "generate",
            "fix", "debug", "test", "review", "optimize", "refactor",
        ];
        let lower = content.to_lowercase();
        let action_count = action_words
            .iter()
            .filter(|w| lower.contains(*w))
            .count();
        let action_score = (action_count as f64 / 2.0).min(1.0);
        score += action_score * 0.4;

        score.clamp(0.0, 1.0)
    }

    /// Calculate completeness score (0.0 - 1.0)
    /// Based on context, examples, and constraints
    fn calculate_completeness(&self, prompt: &Prompt) -> f64 {
        let mut score = 0.0;
        let content = &prompt.content.to_lowercase();

        // Has context
        if prompt.metadata.context.is_some()
            || content.contains("context")
            || content.contains("background")
        {
            score += 0.25;
        }

        // Has examples
        if content.contains("example")
            || content.contains("for instance")
            || content.contains("such as")
            || content.contains("e.g.")
        {
            score += 0.25;
        }

        // Has constraints or requirements
        if content.contains("must")
            || content.contains("should")
            || content.contains("require")
            || content.contains("constraint")
            || content.contains("limit")
        {
            score += 0.25;
        }

        // Has expected output format
        if content.contains("format")
            || content.contains("output")
            || content.contains("return")
            || content.contains("result")
        {
            score += 0.25;
        }

        // Bonus for tags (indicates well-categorized)
        if !prompt.tags.is_empty() {
            score += 0.1 * (prompt.tags.len() as f64).min(2.0) / 2.0;
        }

        // Bonus for category
        if prompt.category.is_some() {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    /// Calculate specificity score (0.0 - 1.0)
    /// Based on numeric specifications, format definitions, and technical terms
    fn calculate_specificity(&self, content: &str) -> f64 {
        let mut score = 0.0;

        // Numeric specifications
        let has_numbers = content.chars().any(|c| c.is_ascii_digit());
        if has_numbers {
            score += 0.25;
        }

        // Technical terms (programming languages, frameworks, etc.)
        let tech_terms = [
            "function", "class", "method", "api", "endpoint", "database",
            "query", "table", "index", "cache", "async", "thread",
            "memory", "performance", "algorithm", "data structure",
        ];
        let lower = content.to_lowercase();
        let tech_count = tech_terms.iter().filter(|t| lower.contains(*t)).count();
        score += (tech_count as f64 / 3.0).min(0.35);

        // Format specifications
        if content.contains("json")
            || content.contains("xml")
            || content.contains("csv")
            || content.contains("yaml")
            || content.contains("markdown")
        {
            score += 0.2;
        }

        // Specific language/framework mentions
        let languages = [
            "rust", "python", "javascript", "typescript", "java", "go",
            "ruby", "php", "swift", "kotlin", "c++", "c#",
        ];
        let lang_count = languages.iter().filter(|l| lower.contains(*l)).count();
        if lang_count > 0 {
            score += 0.2;
        }

        score.clamp(0.0, 1.0)
    }

    /// Calculate guidance score (0.0 - 1.0)
    /// Based on structure and step-by-step instructions
    fn calculate_guidance(&self, prompt: &Prompt) -> f64 {
        let mut score: f64 = 0.0;
        let content = &prompt.content.to_lowercase();

        // Step-by-step indicators
        if content.contains("step")
            || content.contains("first")
            || content.contains("then")
            || content.contains("finally")
            || content.contains("1.")
            || content.contains("2.")
        {
            score += 0.3;
        }

        // Bullet points or numbered lists
        if content.contains("- ")
            || content.contains("* ")
            || content.matches(|c: char| c.is_ascii_digit()).count() > 2
        {
            score += 0.2;
        }

        // Clear structure indicators
        if content.contains("input")
            || content.contains("output")
            || content.contains("parameter")
            || content.contains("argument")
        {
            score += 0.25;
        }

        // Role or persona specification
        if content.contains("you are")
            || content.contains("act as")
            || content.contains("role")
        {
            score += 0.25;
        }

        score.clamp(0.0, 1.0)
    }
}

impl Default for EfficiencyAnalyzer {
    fn default() -> Self {
        Self {
            avg_tokens: 500.0,
            avg_time_ms: 3000.0,
            avg_cost: 0.01,
        }
    }
}

impl EfficiencyAnalyzer {
    /// Create analyzer with custom averages for normalization
    pub fn new(avg_tokens: f64, avg_time_ms: f64, avg_cost: f64) -> Self {
        Self {
            avg_tokens,
            avg_time_ms,
            avg_cost,
        }
    }

    /// Analyze prompt efficiency
    pub fn analyze(&self, prompt: &Prompt) -> Result<EfficiencyMetrics> {
        let token_efficiency = self.calculate_token_efficiency(prompt);
        let time_efficiency = self.calculate_time_efficiency(prompt);
        let cost_efficiency = self.calculate_cost_efficiency(prompt);

        // Weighted efficiency score
        // Token usage: 50%, Execution time: 30%, Cost: 20%
        let efficiency_score =
            token_efficiency * 0.5 + time_efficiency * 0.3 + cost_efficiency * 0.2;

        Ok(EfficiencyMetrics {
            prompt_id: prompt.id.clone(),
            efficiency_score: efficiency_score * 100.0,
            token_efficiency: token_efficiency * 100.0,
            time_efficiency: time_efficiency * 100.0,
            cost_efficiency: cost_efficiency * 100.0,
            calculated_at: Utc::now(),
        })
    }

    /// Update averages based on historical data
    pub fn update_averages(&mut self, avg_tokens: f64, avg_time_ms: f64, avg_cost: f64) {
        self.avg_tokens = avg_tokens;
        self.avg_time_ms = avg_time_ms;
        self.avg_cost = avg_cost;
    }

    /// Calculate token efficiency (0.0 - 1.0)
    fn calculate_token_efficiency(&self, prompt: &Prompt) -> f64 {
        let total_tokens = prompt.metadata.input_tokens.unwrap_or(0)
            + prompt.metadata.output_tokens.unwrap_or(0);

        if total_tokens == 0 {
            return 0.5; // Default for unknown
        }

        // Lower tokens = higher efficiency (inverse relationship)
        let ratio = total_tokens as f64 / self.avg_tokens;

        // Score decreases as token usage increases
        (2.0 - ratio).clamp(0.0, 1.0)
    }

    /// Calculate time efficiency (0.0 - 1.0)
    fn calculate_time_efficiency(&self, prompt: &Prompt) -> f64 {
        let time_ms = prompt.metadata.execution_time_ms.unwrap_or(0);

        if time_ms == 0 {
            return 0.5; // Default for unknown
        }

        // Lower time = higher efficiency
        let ratio = time_ms as f64 / self.avg_time_ms;

        (2.0 - ratio).clamp(0.0, 1.0)
    }

    /// Calculate cost efficiency (0.0 - 1.0)
    fn calculate_cost_efficiency(&self, prompt: &Prompt) -> f64 {
        let cost = prompt.metadata.estimated_cost.unwrap_or(0.0);

        if cost == 0.0 {
            return 0.5; // Default for unknown
        }

        // Lower cost = higher efficiency
        let ratio = cost / self.avg_cost;

        (2.0 - ratio).clamp(0.0, 1.0)
    }
}

/// Aggregate analysis results
#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    pub total_prompts: usize,
    pub avg_quality_score: f64,
    pub avg_efficiency_score: f64,
    pub top_categories: Vec<(String, usize)>,
    pub common_tags: Vec<(String, usize)>,
}

/// Generate analysis summary from prompts
pub fn generate_summary(
    prompts: &[Prompt],
    quality_scores: &[QualityScore],
    efficiency_metrics: &[EfficiencyMetrics],
) -> AnalysisSummary {
    use std::collections::HashMap;

    // Calculate averages
    let avg_quality = if quality_scores.is_empty() {
        0.0
    } else {
        quality_scores.iter().map(|s| s.total_score).sum::<f64>() / quality_scores.len() as f64
    };

    let avg_efficiency = if efficiency_metrics.is_empty() {
        0.0
    } else {
        efficiency_metrics
            .iter()
            .map(|m| m.efficiency_score)
            .sum::<f64>()
            / efficiency_metrics.len() as f64
    };

    // Count categories
    let mut category_counts: HashMap<String, usize> = HashMap::new();
    for prompt in prompts {
        if let Some(ref cat) = prompt.category {
            *category_counts.entry(cat.clone()).or_insert(0) += 1;
        }
    }
    let mut top_categories: Vec<_> = category_counts.into_iter().collect();
    top_categories.sort_by(|a, b| b.1.cmp(&a.1));
    top_categories.truncate(5);

    // Count tags
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for prompt in prompts {
        for tag in &prompt.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut common_tags: Vec<_> = tag_counts.into_iter().collect();
    common_tags.sort_by(|a, b| b.1.cmp(&a.1));
    common_tags.truncate(10);

    AnalysisSummary {
        total_prompts: prompts.len(),
        avg_quality_score: avg_quality,
        avg_efficiency_score: avg_efficiency,
        top_categories,
        common_tags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PromptMetadata;

    fn create_test_prompt(content: &str) -> Prompt {
        let mut prompt = Prompt::new(content.to_string());
        prompt.content_hash = "test_hash".to_string();
        prompt
    }

    #[test]
    fn test_quality_analyzer_default() {
        let analyzer = QualityAnalyzer::default();
        let prompt = create_test_prompt("Write a function that sorts an array");
        let score = analyzer.analyze(&prompt).unwrap();

        assert!(score.total_score >= 0.0 && score.total_score <= 100.0);
        assert!(score.clarity >= 0.0 && score.clarity <= 100.0);
        assert!(score.completeness >= 0.0 && score.completeness <= 100.0);
        assert!(score.specificity >= 0.0 && score.specificity <= 100.0);
        assert!(score.guidance >= 0.0 && score.guidance <= 100.0);
    }

    #[test]
    fn test_quality_clarity_high() {
        let analyzer = QualityAnalyzer::default();
        let prompt = create_test_prompt(
            "Create a Python function that takes a list of integers and returns the sorted list in ascending order.",
        );
        let score = analyzer.analyze(&prompt).unwrap();

        // Should have good clarity due to action word and clear structure
        assert!(score.clarity > 30.0);
    }

    #[test]
    fn test_quality_completeness_high() {
        let analyzer = QualityAnalyzer::default();
        let mut prompt = create_test_prompt(
            "Context: Building a REST API. Create a function that validates user input. \
             Must handle null values. Return JSON format.",
        );
        prompt.category = Some("code-generation".to_string());
        prompt.tags = vec!["api".to_string(), "validation".to_string()];

        let score = analyzer.analyze(&prompt).unwrap();

        // Should have good completeness due to context, constraints, and format
        assert!(score.completeness > 50.0);
    }

    #[test]
    fn test_quality_specificity_high() {
        let analyzer = QualityAnalyzer::default();
        let prompt = create_test_prompt(
            "Write a Rust function that creates a database connection pool with 10 connections.",
        );
        let score = analyzer.analyze(&prompt).unwrap();

        // Should have good specificity due to technical terms and numbers
        assert!(score.specificity > 40.0);
    }

    #[test]
    fn test_quality_guidance_high() {
        let analyzer = QualityAnalyzer::default();
        let prompt = create_test_prompt(
            "Step 1: Parse the input JSON. Step 2: Validate the data. \
             Step 3: Return the result. Input: JSON string. Output: ValidationResult.",
        );
        let score = analyzer.analyze(&prompt).unwrap();

        // Should have good guidance due to step-by-step instructions
        assert!(score.guidance > 50.0);
    }

    #[test]
    fn test_efficiency_analyzer_default() {
        let analyzer = EfficiencyAnalyzer::default();
        let mut prompt = create_test_prompt("Test prompt");
        prompt.metadata.input_tokens = Some(100);
        prompt.metadata.output_tokens = Some(200);
        prompt.metadata.execution_time_ms = Some(1000);
        prompt.metadata.estimated_cost = Some(0.005);

        let metrics = analyzer.analyze(&prompt).unwrap();

        assert!(metrics.efficiency_score >= 0.0 && metrics.efficiency_score <= 100.0);
    }

    #[test]
    fn test_efficiency_high_tokens() {
        let analyzer = EfficiencyAnalyzer::default();
        let mut prompt = create_test_prompt("Test prompt");
        prompt.metadata.input_tokens = Some(1000);
        prompt.metadata.output_tokens = Some(1000);

        let metrics = analyzer.analyze(&prompt).unwrap();

        // High token usage should result in lower efficiency
        assert!(metrics.token_efficiency < 50.0);
    }

    #[test]
    fn test_efficiency_low_tokens() {
        let analyzer = EfficiencyAnalyzer::default();
        let mut prompt = create_test_prompt("Test prompt");
        prompt.metadata.input_tokens = Some(50);
        prompt.metadata.output_tokens = Some(50);

        let metrics = analyzer.analyze(&prompt).unwrap();

        // Low token usage should result in higher efficiency
        assert!(metrics.token_efficiency > 70.0);
    }

    #[test]
    fn test_analysis_summary() {
        let mut prompts = vec![
            create_test_prompt("First prompt"),
            create_test_prompt("Second prompt"),
        ];
        prompts[0].category = Some("code-generation".to_string());
        prompts[0].tags = vec!["rust".to_string()];
        prompts[1].category = Some("code-generation".to_string());
        prompts[1].tags = vec!["rust".to_string(), "api".to_string()];

        let quality_scores = vec![
            QualityScore {
                prompt_id: prompts[0].id.clone(),
                total_score: 80.0,
                clarity: 75.0,
                completeness: 85.0,
                specificity: 70.0,
                guidance: 80.0,
                analyzed_at: Utc::now(),
            },
            QualityScore {
                prompt_id: prompts[1].id.clone(),
                total_score: 90.0,
                clarity: 85.0,
                completeness: 95.0,
                specificity: 80.0,
                guidance: 90.0,
                analyzed_at: Utc::now(),
            },
        ];

        let efficiency_metrics = vec![
            EfficiencyMetrics {
                prompt_id: prompts[0].id.clone(),
                efficiency_score: 70.0,
                token_efficiency: 65.0,
                time_efficiency: 75.0,
                cost_efficiency: 70.0,
                calculated_at: Utc::now(),
            },
        ];

        let summary = generate_summary(&prompts, &quality_scores, &efficiency_metrics);

        assert_eq!(summary.total_prompts, 2);
        assert_eq!(summary.avg_quality_score, 85.0);
        assert_eq!(summary.avg_efficiency_score, 70.0);
        assert!(!summary.top_categories.is_empty());
        assert!(!summary.common_tags.is_empty());
    }

    #[test]
    fn test_custom_weights() {
        let weights = QualityWeights {
            clarity: 0.5,
            completeness: 0.3,
            specificity: 0.1,
            guidance: 0.1,
        };
        let analyzer = QualityAnalyzer::new(weights);
        let prompt = create_test_prompt("Write a function");
        let score = analyzer.analyze(&prompt).unwrap();

        assert!(score.total_score >= 0.0 && score.total_score <= 100.0);
    }
}
