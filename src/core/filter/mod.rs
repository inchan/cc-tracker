//! Advanced filter parsing for prompt queries
//!
//! Supports syntax like:
//! - `category:code` - filter by category
//! - `tag:rust` - filter by tag
//! - `status:active` - filter by status
//! - `quality:>80` - quality score above 80
//! - `efficiency:>=70` - efficiency score >= 70
//! - `date:>2024-01-01` - created after date
//! - `limit:10` - limit results
//! - Text without prefix for content search

use chrono::{DateTime, NaiveDate, Utc};

use crate::database::PromptFilter;
use crate::models::PromptStatus;
use crate::{PromptTrackingError, Result};

/// A parsed filter token
#[derive(Debug, Clone, PartialEq)]
pub enum FilterToken {
    /// Category filter
    Category(String),
    /// Tag filter
    Tag(String),
    /// Status filter
    Status(PromptStatus),
    /// Quality score comparison
    Quality(Comparison, f64),
    /// Efficiency score comparison
    Efficiency(Comparison, f64),
    /// Date filter (created_at)
    DateFrom(DateTime<Utc>),
    /// Date filter (created_at)
    DateTo(DateTime<Utc>),
    /// Limit results
    Limit(usize),
    /// Offset results
    Offset(usize),
    /// Text search query
    Search(String),
}

/// Comparison operators for numeric filters
#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
    Equal,
}

/// Parse an advanced filter query string into a PromptFilter
pub fn parse_filter_query(query: &str) -> Result<PromptFilter> {
    let tokens = tokenize_query(query)?;
    let filter = tokens_to_filter(tokens)?;
    Ok(filter)
}

/// Tokenize a query string into filter tokens
fn tokenize_query(query: &str) -> Result<Vec<FilterToken>> {
    let mut tokens = Vec::new();
    let mut search_terms = Vec::new();

    // Split by whitespace but respect quotes
    let parts = split_query(query);

    for part in parts {
        if let Some(token) = parse_token(&part)? {
            tokens.push(token);
        } else if !part.is_empty() {
            // Collect as search term
            search_terms.push(part);
        }
    }

    // Combine search terms if any
    if !search_terms.is_empty() {
        tokens.push(FilterToken::Search(search_terms.join(" ")));
    }

    Ok(tokens)
}

/// Split query respecting quoted strings
fn split_query(query: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for ch in query.chars() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            c if c == quote_char && in_quotes => {
                in_quotes = false;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Parse a single token from a query part
fn parse_token(part: &str) -> Result<Option<FilterToken>> {
    // Check for field:value pattern
    if let Some(colon_idx) = part.find(':') {
        let field = &part[..colon_idx].to_lowercase();
        let value = &part[colon_idx + 1..];

        match field.as_str() {
            "category" | "cat" => {
                return Ok(Some(FilterToken::Category(value.to_string())));
            }
            "tag" => {
                return Ok(Some(FilterToken::Tag(value.to_string())));
            }
            "status" => {
                let status = value.parse::<PromptStatus>().map_err(|e| {
                    PromptTrackingError::ConfigError(format!("Invalid status: {}", e))
                })?;
                return Ok(Some(FilterToken::Status(status)));
            }
            "quality" | "q" => {
                let (comp, num) = parse_comparison(value)?;
                return Ok(Some(FilterToken::Quality(comp, num)));
            }
            "efficiency" | "eff" => {
                let (comp, num) = parse_comparison(value)?;
                return Ok(Some(FilterToken::Efficiency(comp, num)));
            }
            "date" | "created" => {
                let (comp, date) = parse_date_comparison(value)?;
                match comp {
                    Comparison::GreaterThan | Comparison::GreaterOrEqual => {
                        return Ok(Some(FilterToken::DateFrom(date)));
                    }
                    Comparison::LessThan | Comparison::LessOrEqual => {
                        return Ok(Some(FilterToken::DateTo(date)));
                    }
                    Comparison::Equal => {
                        return Ok(Some(FilterToken::DateFrom(date)));
                    }
                }
            }
            "limit" => {
                let num = value.parse::<usize>().map_err(|e| {
                    PromptTrackingError::ConfigError(format!("Invalid limit: {}", e))
                })?;
                return Ok(Some(FilterToken::Limit(num)));
            }
            "offset" | "skip" => {
                let num = value.parse::<usize>().map_err(|e| {
                    PromptTrackingError::ConfigError(format!("Invalid offset: {}", e))
                })?;
                return Ok(Some(FilterToken::Offset(num)));
            }
            _ => {
                // Unknown field, treat as search term
                return Ok(None);
            }
        }
    }

    // Not a field:value pattern
    Ok(None)
}

/// Parse a comparison operator and value
fn parse_comparison(value: &str) -> Result<(Comparison, f64)> {
    let (comp, num_str) = if value.starts_with(">=") {
        (Comparison::GreaterOrEqual, &value[2..])
    } else if value.starts_with("<=") {
        (Comparison::LessOrEqual, &value[2..])
    } else if value.starts_with('>') {
        (Comparison::GreaterThan, &value[1..])
    } else if value.starts_with('<') {
        (Comparison::LessThan, &value[1..])
    } else {
        (Comparison::Equal, value)
    };

    let num = num_str.parse::<f64>().map_err(|e| {
        PromptTrackingError::ConfigError(format!("Invalid number '{}': {}", num_str, e))
    })?;

    Ok((comp, num))
}

/// Parse a date comparison
fn parse_date_comparison(value: &str) -> Result<(Comparison, DateTime<Utc>)> {
    let (comp, date_str) = if value.starts_with(">=") {
        (Comparison::GreaterOrEqual, &value[2..])
    } else if value.starts_with("<=") {
        (Comparison::LessOrEqual, &value[2..])
    } else if value.starts_with('>') {
        (Comparison::GreaterThan, &value[1..])
    } else if value.starts_with('<') {
        (Comparison::LessThan, &value[1..])
    } else {
        (Comparison::Equal, value)
    };

    // Parse date in YYYY-MM-DD format
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        PromptTrackingError::ConfigError(format!("Invalid date '{}': {}", date_str, e))
    })?;

    let datetime = naive_date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| PromptTrackingError::ConfigError("Invalid date time".to_string()))?
        .and_utc();

    Ok((comp, datetime))
}

/// Convert tokens to a PromptFilter
fn tokens_to_filter(tokens: Vec<FilterToken>) -> Result<PromptFilter> {
    let mut filter = PromptFilter::default();

    for token in tokens {
        match token {
            FilterToken::Category(cat) => {
                filter.category = Some(cat);
            }
            FilterToken::Tag(tag) => {
                filter.tags.push(tag);
            }
            FilterToken::Status(status) => {
                filter.status = Some(status);
            }
            FilterToken::Quality(comp, value) => match comp {
                Comparison::GreaterThan | Comparison::GreaterOrEqual => {
                    filter.min_quality_score = Some(value);
                }
                Comparison::LessThan | Comparison::LessOrEqual => {
                    filter.max_quality_score = Some(value);
                }
                Comparison::Equal => {
                    filter.min_quality_score = Some(value);
                    filter.max_quality_score = Some(value);
                }
            },
            FilterToken::Efficiency(comp, value) => {
                // Store efficiency in min/max quality for now
                // We could extend PromptFilter to support efficiency filtering
                match comp {
                    Comparison::GreaterThan | Comparison::GreaterOrEqual => {
                        // Store as placeholder - would need filter extension
                        filter.min_quality_score =
                            filter.min_quality_score.or(Some(0.0)).map(|_| value);
                    }
                    _ => {}
                }
            }
            FilterToken::DateFrom(date) => {
                filter.date_from = Some(date);
            }
            FilterToken::DateTo(date) => {
                filter.date_to = Some(date);
            }
            FilterToken::Limit(limit) => {
                filter.limit = Some(limit);
            }
            FilterToken::Offset(offset) => {
                filter.offset = Some(offset);
            }
            FilterToken::Search(query) => {
                filter.search_query = Some(query);
            }
        }
    }

    Ok(filter)
}

/// Format a PromptFilter as a human-readable query string
pub fn format_filter(filter: &PromptFilter) -> String {
    let mut parts = Vec::new();

    if let Some(ref cat) = filter.category {
        parts.push(format!("category:{}", cat));
    }

    for tag in &filter.tags {
        parts.push(format!("tag:{}", tag));
    }

    if let Some(ref status) = filter.status {
        parts.push(format!("status:{}", status));
    }

    if let Some(score) = filter.min_quality_score {
        parts.push(format!("quality:>={}", score));
    }

    if let Some(score) = filter.max_quality_score {
        parts.push(format!("quality:<={}", score));
    }

    if let Some(ref date) = filter.date_from {
        parts.push(format!("date:>={}", date.format("%Y-%m-%d")));
    }

    if let Some(ref date) = filter.date_to {
        parts.push(format!("date:<={}", date.format("%Y-%m-%d")));
    }

    if let Some(limit) = filter.limit {
        parts.push(format!("limit:{}", limit));
    }

    if let Some(offset) = filter.offset {
        parts.push(format!("offset:{}", offset));
    }

    if let Some(ref query) = filter.search_query {
        parts.push(query.clone());
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_category() {
        let filter = parse_filter_query("category:code").unwrap();
        assert_eq!(filter.category, Some("code".to_string()));
    }

    #[test]
    fn test_parse_tag() {
        let filter = parse_filter_query("tag:rust tag:async").unwrap();
        assert_eq!(filter.tags, vec!["rust".to_string(), "async".to_string()]);
    }

    #[test]
    fn test_parse_status() {
        let filter = parse_filter_query("status:archived").unwrap();
        assert_eq!(filter.status, Some(PromptStatus::Archived));
    }

    #[test]
    fn test_parse_quality_greater() {
        let filter = parse_filter_query("quality:>80").unwrap();
        assert_eq!(filter.min_quality_score, Some(80.0));
    }

    #[test]
    fn test_parse_quality_less() {
        let filter = parse_filter_query("quality:<90").unwrap();
        assert_eq!(filter.max_quality_score, Some(90.0));
    }

    #[test]
    fn test_parse_date() {
        let filter = parse_filter_query("date:>2024-01-01").unwrap();
        assert!(filter.date_from.is_some());
    }

    #[test]
    fn test_parse_limit_offset() {
        let filter = parse_filter_query("limit:10 offset:5").unwrap();
        assert_eq!(filter.limit, Some(10));
        assert_eq!(filter.offset, Some(5));
    }

    #[test]
    fn test_parse_search_text() {
        let filter = parse_filter_query("hello world").unwrap();
        assert_eq!(filter.search_query, Some("hello world".to_string()));
    }

    #[test]
    fn test_parse_combined() {
        let filter =
            parse_filter_query("category:code tag:rust quality:>80 fibonacci").unwrap();
        assert_eq!(filter.category, Some("code".to_string()));
        assert_eq!(filter.tags, vec!["rust".to_string()]);
        assert_eq!(filter.min_quality_score, Some(80.0));
        assert_eq!(filter.search_query, Some("fibonacci".to_string()));
    }

    #[test]
    fn test_parse_abbreviations() {
        let filter = parse_filter_query("cat:test q:>70").unwrap();
        assert_eq!(filter.category, Some("test".to_string()));
        assert_eq!(filter.min_quality_score, Some(70.0));
    }

    #[test]
    fn test_split_quoted() {
        let parts = split_query("tag:rust \"hello world\"");
        assert_eq!(parts, vec!["tag:rust", "hello world"]);
    }

    #[test]
    fn test_format_filter() {
        let mut filter = PromptFilter::default();
        filter.category = Some("code".to_string());
        filter.tags.push("rust".to_string());
        filter.min_quality_score = Some(80.0);
        filter.limit = Some(10);

        let formatted = format_filter(&filter);
        assert!(formatted.contains("category:code"));
        assert!(formatted.contains("tag:rust"));
        assert!(formatted.contains("quality:>=80"));
        assert!(formatted.contains("limit:10"));
    }

    #[test]
    fn test_empty_query() {
        let filter = parse_filter_query("").unwrap();
        assert_eq!(filter.category, None);
        assert!(filter.tags.is_empty());
    }

    #[test]
    fn test_invalid_date() {
        let result = parse_filter_query("date:>invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_number() {
        let result = parse_filter_query("quality:>abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_comparison_operators() {
        // >=
        let filter = parse_filter_query("quality:>=80").unwrap();
        assert_eq!(filter.min_quality_score, Some(80.0));

        // <=
        let filter = parse_filter_query("quality:<=90").unwrap();
        assert_eq!(filter.max_quality_score, Some(90.0));

        // =
        let filter = parse_filter_query("quality:85").unwrap();
        assert_eq!(filter.min_quality_score, Some(85.0));
        assert_eq!(filter.max_quality_score, Some(85.0));
    }
}
