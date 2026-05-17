use nucleo::{Matcher, Utf32Str};

pub struct FuzzyEngine {
    matcher: Matcher,
    buf: Vec<char>,
}

impl FuzzyEngine {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(nucleo::Config::DEFAULT),
            buf: Vec::new(),
        }
    }

    pub fn score(&mut self, query: &str, target: &str) -> Option<u16> {
        if query.is_empty() || target.is_empty() {
            return None;
        }

        let query_lower = query.to_lowercase();
        let target_lower = target.to_lowercase();

        // Exact substring match — highest priority
        if target_lower.contains(&query_lower) {
            let pos = target_lower.find(&query_lower).unwrap_or(0);
            return Some(1000 - pos as u16);
        }

        // nucleo fuzzy match as fallback
        let mut q_buf: Vec<char> = query_lower.chars().collect();
        let mut t_buf: Vec<char> = target_lower.chars().collect();
        let query_str = Utf32Str::new(&query_lower, &mut q_buf);
        let target_str = Utf32Str::new(&target_lower, &mut t_buf);

        self.matcher.fuzzy_match(target_str, query_str)
    }
}

impl Default for FuzzyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Chinese text to pinyin full and initials forms
pub fn text_to_pinyin(text: &str) -> (String, String) {
    let pinyin_vec = pinyin::to_pinyin_vec(text, |p| p.with_tone());

    let full: Vec<String> = pinyin_vec
        .iter()
        .map(|s| {
            let plain: String = s
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect();
            plain.to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect();

    let initials: String = full
        .iter()
        .map(|s| s.chars().next().unwrap_or(' '))
        .collect();

    (full.join(" "), initials)
}
