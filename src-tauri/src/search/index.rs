use crate::db::models::PageEntry;
use crate::search::fuzzy::{text_to_pinyin, FuzzyEngine};

#[derive(Debug, Clone)]
pub struct ScoredEntry {
    pub entry: PageEntry,
    pub score: u16,
}

pub struct SearchIndex {
    engine: FuzzyEngine,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            engine: FuzzyEngine::new(),
        }
    }

    /// Score entries against a query, returning sorted results
    pub fn search(&mut self, query: &str, entries: &[PageEntry]) -> Vec<ScoredEntry> {
        if query.trim().is_empty() {
            return entries
                .iter()
                .map(|e| ScoredEntry {
                    entry: e.clone(),
                    score: 0,
                })
                .collect();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<ScoredEntry> = Vec::new();

        for entry in entries {
            let mut max_score: u16 = 0;

            // Score against name
            if let Some(s) = self.engine.score(&query_lower, &entry.name) {
                max_score = max_score.max(s + 500); // name match bonus
            }

            // Score against pinyin full
            if !entry.pinyin_full.is_empty() {
                if let Some(s) = self.engine.score(&query_lower, &entry.pinyin_full) {
                    max_score = max_score.max(s + 300);
                }
            }

            // Score against pinyin initials
            if !entry.pinyin_initials.is_empty() {
                if let Some(s) = self.engine.score(&query_lower, &entry.pinyin_initials) {
                    max_score = max_score.max(s + 200);
                }
            }

            // Score against URL
            if let Some(s) = self.engine.score(&query_lower, &entry.url) {
                max_score = max_score.max(s);
            }

            // Filter out weak false-positive matches
            if max_score >= 400 {
                results.push(ScoredEntry {
                    entry: entry.clone(),
                    score: max_score,
                });
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }

    /// Score a single string against a query, used for open tab titles
    pub fn search_one(&mut self, query: &str, target: &str) -> Option<u16> {
        self.engine.score(&query.to_lowercase(), &target.to_lowercase())
    }

    /// Compute pinyin representations for a given name and URL
    pub fn compute_pinyin(name: &str) -> (String, String) {
        text_to_pinyin(name)
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}
