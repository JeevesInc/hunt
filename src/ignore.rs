use std::collections::HashSet;
use std::path::Path;

/// Default ignore patterns for common build and dependency directories
const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "target",
    ".next",
    ".nuxt",
    ".cache",
    "coverage",
    ".idea",
    ".vscode",
    ".DS_Store",
    "*.log",
    // Test directories
    "__tests__",
    "tests",
    "test",
    // Test file patterns
    "*.test.js",
    "*.test.jsx",
    "*.test.ts",
    "*.test.tsx",
    "*.spec.js",
    "*.spec.jsx",
    "*.spec.ts",
    "*.spec.tsx",
    "*.snap",
];

/// Compiled ignore patterns for efficient matching
pub struct IgnorePatterns {
    exact_matches: HashSet<String>,
    glob_regexes: Vec<regex::Regex>,
}

impl IgnorePatterns {
    /// Check if a file path should be ignored
    pub fn should_ignore(&self, file_path: &str) -> bool {
        let path = Path::new(file_path);
        let path_str = file_path;
        
        // Fast path: check exact directory/file name matches first
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                let component_str = name.to_string_lossy();
                if self.exact_matches.contains(component_str.as_ref()) {
                    return true;
                }
            }
        }
        
        // Check glob patterns (slower, but pre-compiled)
        for pattern in &self.glob_regexes {
            // Check filename first (most common case)
            if let Some(file_name) = path.file_name() {
                if pattern.is_match(&file_name.to_string_lossy()) {
                    return true;
                }
            }
            // Check full path as fallback
            if pattern.is_match(path_str) {
                return true;
            }
        }
        
        false
    }
}

/// Load default ignore patterns
pub fn load_ignore_patterns() -> IgnorePatterns {
    let mut exact_matches = HashSet::new();
    let mut glob_regexes = Vec::new();
    
    // Load default patterns
    for pattern in DEFAULT_IGNORE_PATTERNS {
        if pattern.contains('*') {
            // Compile glob pattern to regex once
            let regex_pattern = pattern
                .replace(".", "\\.")
                .replace("*", ".*");
            if let Ok(re) = regex::Regex::new(&regex_pattern) {
                glob_regexes.push(re);
            }
        } else {
            exact_matches.insert(pattern.to_string());
        }
    }
    
    IgnorePatterns {
        exact_matches,
        glob_regexes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_patterns() {
        let patterns = load_ignore_patterns();
        assert!(patterns.should_ignore("src/node_modules/foo.js"));
        assert!(patterns.should_ignore(".git/config"));
    }
    
    #[test]
    fn test_should_ignore_path() {
        let patterns = load_ignore_patterns();
        
        assert!(patterns.should_ignore("src/node_modules/foo.js"));
        assert!(patterns.should_ignore("app.log"));
        assert!(!patterns.should_ignore("src/components/Button.tsx"));
    }
}

