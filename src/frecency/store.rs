use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrecencyEntry {
    pub path: PathBuf,
    pub visit_count: u32,
    pub last_visited: u64, // Epoch seconds
}

impl FrecencyEntry {
    pub fn score(&self, now: u64) -> f64 {
        let age_secs = now.saturating_sub(self.last_visited);
        let weight = if age_secs < 3600 {
            4.0 // Visited in the last hour
        } else if age_secs < 86400 {
            2.0 // Visited in the last day
        } else if age_secs < 604800 {
            1.0 // Visited in the last week
        } else {
            0.5 // Older
        };

        (self.visit_count as f64) * weight
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrecencyStore {
    pub entries: Vec<FrecencyEntry>,
}

impl FrecencyStore {
    pub fn load_or_default() -> Self {
        let path = Self::store_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(store) = serde_json::from_str::<FrecencyStore>(&content) {
                return store;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    pub fn record_visit(&mut self, path: &Path) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if let Some(entry) = self.entries.iter_mut().find(|e| e.path == path) {
            entry.visit_count += 1;
            entry.last_visited = now;
        } else {
            self.entries.push(FrecencyEntry {
                path: path.to_path_buf(),
                visit_count: 1,
                last_visited: now,
            });
        }

        // Cap store to 500 entries, prune lowest scoring
        if self.entries.len() > 500 {
            self.entries
                .sort_by(|a, b| b.score(now).partial_cmp(&a.score(now)).unwrap());
            self.entries.truncate(500);
        }

        let _ = self.save();
    }

    /// Fuzzy search against stored paths with scores
    pub fn query(&self, pattern: &str) -> Vec<(PathBuf, i64, Vec<usize>)> {
        let matcher = SkimMatcherV2::default();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut results: Vec<(PathBuf, i64, Vec<usize>)> = Vec::new();

        for entry in &self.entries {
            let path_str = entry.path.to_string_lossy();
            if pattern.trim().is_empty() {
                let frecency_boost = (entry.score(now) * 10.0) as i64;
                results.push((entry.path.clone(), frecency_boost, Vec::new()));
            } else if let Some((fuzzy_score, indices)) = matcher.fuzzy_indices(&path_str, pattern) {
                let frecency_boost = (entry.score(now) * 5.0) as i64;
                let total_score = fuzzy_score + frecency_boost;
                results.push((entry.path.clone(), total_score, indices));
            }
        }

        results.sort_by_key(|a| std::cmp::Reverse(a.1));
        results
    }

    fn store_path() -> PathBuf {
        directories::ProjectDirs::from("org", "vit", "strata")
            .map(|proj| proj.data_local_dir().join("frecency.json"))
            .unwrap_or_else(|| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join(".local/share/strata/frecency.json"))
                    .unwrap_or_else(|_| PathBuf::from("/tmp/strata_frecency.json"))
            })
    }
}
