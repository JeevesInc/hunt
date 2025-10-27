use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

/// Supported file extensions
const SUPPORTED_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx"];

/// Discover source files in the given directories, skipping ignored directories during traversal
pub fn discover_source_files(source_dirs: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let ignore_patterns = crate::ignore::load_ignore_patterns();
    let mut all_files = Vec::new();
    
    for source_dir in source_dirs {
        let walker = WalkDir::new(source_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // For directories, check if we should skip them entirely
                if e.file_type().is_dir() {
                    let path_str = e.path().to_string_lossy();
                    // If path should be ignored, return false to skip traversing into it
                    !ignore_patterns.should_ignore(&path_str)
                } else {
                    // For files, always include them (we'll filter later)
                    true
                }
            });
        
        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip files we can't read
            };
            
            // Only process files, not directories
            if !entry.file_type().is_file() {
                continue;
            }
            
            let path = entry.path();
            
            // Check if file has supported extension
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if SUPPORTED_EXTENSIONS.contains(&ext_str) {
                        let file_path = path.to_string_lossy().to_string();
                        // Final check: make sure the file path itself isn't ignored (for glob patterns like *.log)
                        if !ignore_patterns.should_ignore(&file_path) {
                            all_files.push(file_path);
                        }
                    }
                }
            }
        }
    }
    
    Ok(all_files)
}

/// Check which translation keys are used in source files
pub fn check_translation_usage(
    translations: &std::collections::HashMap<String, Value>, 
    source_files: &[String]
) -> HashSet<String> {
    let pb = create_progress_bar();
    pb.set_message("The lion is on the hunt…");
    pb.enable_steady_tick(std::time::Duration::from_millis(50));
    
    let compiled_patterns = compile_regex_patterns(translations);
    let base_prefixes = extract_base_prefixes(translations);
    let dynamic_patterns = compile_dynamic_patterns(&base_prefixes);
    
    // Check both exact matches and dynamic patterns in a single pass through files
    let used_keys = find_used_keys_combined(
        &compiled_patterns, 
        &dynamic_patterns, 
        &base_prefixes,
        source_files
    );
    
    pb.finish_and_clear();
    used_keys
}

/// Create a progress bar with consistent styling
fn create_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb
}

/// Compile regex patterns for all translation keys
fn compile_regex_patterns(
    translations: &std::collections::HashMap<String, Value>
) -> Vec<(String, Regex)> {
    let mut compiled_patterns: Vec<(String, Regex)> = Vec::new();
    for (key, _value) in translations.iter() {
        let escaped_key = regex::escape(key);
        let pattern = format!(r"\b{}\b", escaped_key);
        if let Ok(re) = Regex::new(&pattern) {
            compiled_patterns.push((key.clone(), re));
        }
    }
    compiled_patterns
}

/// Extract base prefixes from translation keys (e.g., "expenseCategory" from "expenseCategory.foo")
/// Returns a map of prefix -> set of keys that start with that prefix
fn extract_base_prefixes(
    translations: &std::collections::HashMap<String, Value>
) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
    let mut prefix_map: std::collections::HashMap<String, std::collections::HashSet<String>> = 
        std::collections::HashMap::new();
    
    for key in translations.keys() {
        // Find the base prefix - everything before the last dot, or the key itself if no dot
        if let Some(dot_pos) = key.rfind('.') {
            let prefix = key[..dot_pos].to_string();
            prefix_map
                .entry(prefix)
                .or_insert_with(std::collections::HashSet::new)
                .insert(key.clone());
        }
    }
    
    prefix_map
}

/// Compile regex patterns for dynamic key usage (e.g., "expenseCategory.${...}")
fn compile_dynamic_patterns(
    prefixes: &std::collections::HashMap<String, std::collections::HashSet<String>>
) -> Vec<(String, Regex)> {
    let mut patterns: Vec<(String, Regex)> = Vec::new();
    
    for (prefix, _keys) in prefixes.iter() {
        let escaped_prefix = regex::escape(prefix);
        
        // Pattern to match dynamic key construction:
        // - Template literals: `expenseCategory.${var}` or `Reimbursement:expenseCategory.${var}`
        // - String literals: "expenseCategory.${var}" or 'expenseCategory.${var}'
        // - Function calls: t(`expenseCategory.${var}`, ...) or t("expenseCategory.${var}", ...)
        // - With namespace: "Reimbursement:expenseCategory.${var}"
        // 
        // Pattern matches:
        // - prefix.${...}
        // - Namespace:prefix.${...}
        // - 'prefix.${...}' or "prefix.${...}" or `prefix.${...}`
        //
        // Uses a flexible pattern that matches prefix followed by .${ and some content
        
        // Match: prefix.${variable} or Namespace:prefix.${variable}
        // This covers template literals, string literals, and function arguments
        let pattern = format!(
            r#"{}\.\$\{{[^}}]+\}}"#,
            escaped_prefix
        );
        
        if let Ok(re) = Regex::new(&pattern) {
            patterns.push((prefix.clone(), re));
        }
    }
    
    patterns
}

/// Find used keys by scanning source files (checks both exact matches and dynamic patterns in one pass)
fn find_used_keys_combined(
    exact_patterns: &[(String, Regex)], 
    dynamic_patterns: &[(String, Regex)],
    base_prefixes: &std::collections::HashMap<String, std::collections::HashSet<String>>,
    source_files: &[String]
) -> HashSet<String> {
    let mut used_keys = HashSet::new();
    let mut found_prefixes = HashSet::new();
    // Cache prefixes where all keys have been found via exact matches (avoid recalculating)
    let mut prefixes_complete = HashSet::new();
    
    // Build reverse lookup: key -> prefix (for optimization)
    let mut key_to_prefix: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for (prefix, keys) in base_prefixes {
        for key in keys {
            key_to_prefix.insert(key.clone(), prefix.clone());
        }
    }
    
    // Single pass through all files - check both exact and dynamic patterns
    for file_path in source_files {
        if let Ok(content) = fs::read_to_string(file_path) {
            // STEP 1: Check for exact key matches FIRST
            // (Skip keys whose prefix was already found dynamically - we'll mark them all anyway)
            for (key, pattern) in exact_patterns {
                // Skip if key already found or its prefix was found dynamically
                if used_keys.contains(key) {
                    continue;
                }
                
                // Optimization: Skip exact check if prefix was already found dynamically
                // (all keys with that prefix will be marked as used later)
                if let Some(prefix) = key_to_prefix.get(key) {
                    if found_prefixes.contains(prefix) {
                        continue;
                    }
                }
                
                if pattern.is_match(&content) {
                    used_keys.insert(key.clone());
                    
                    // Check if this was the last key for this prefix
                    if let Some(prefix) = key_to_prefix.get(key) {
                        if !prefixes_complete.contains(prefix) {
                            if let Some(keys_with_prefix) = base_prefixes.get(prefix) {
                                if keys_with_prefix.iter().all(|k| used_keys.contains(k)) {
                                    prefixes_complete.insert(prefix.clone());
                                }
                            }
                        }
                    }
                }
            }
            
            // STEP 2: Check dynamic patterns for prefixes that aren't complete yet
            for (prefix, pattern) in dynamic_patterns {
                // Skip if already found dynamically
                if found_prefixes.contains(prefix) {
                    continue;
                }
                
                // Skip if all keys with this prefix are already found via exact matches
                if prefixes_complete.contains(prefix) {
                    continue;
                }
                
                // Check if all keys are now found (check once per file instead of per-pattern)
                let all_keys_found = if let Some(keys_with_prefix) = base_prefixes.get(prefix) {
                    keys_with_prefix.iter().all(|key| used_keys.contains(key))
                } else {
                    false
                };
                
                if all_keys_found {
                    prefixes_complete.insert(prefix.clone());
                    continue;
                }
                
                // Only run regex if we still need to check
                if pattern.is_match(&content) {
                    found_prefixes.insert(prefix.clone());
                }
            }
        }
    }
    
    // Mark all keys with dynamically found prefixes as used
    for prefix in found_prefixes {
        if let Some(keys_with_prefix) = base_prefixes.get(&prefix) {
            for key in keys_with_prefix {
                used_keys.insert(key.clone());
            }
        }
    }
    
    used_keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    fn create_temp_translations() -> std::collections::HashMap<String, serde_json::Value> {
        let mut map = std::collections::HashMap::new();
        map.insert("hello.world".to_string(), json!("Hello World"));
        map.insert("foo.bar".to_string(), json!("Foo Bar"));
        map
    }
    
    #[test]
    fn test_compile_regex_patterns() {
        let translations = create_temp_translations();
        let patterns = compile_regex_patterns(&translations);
        
        assert_eq!(patterns.len(), 2);
    }
    
    #[test]
    fn test_extract_base_prefixes() {
        let mut translations = std::collections::HashMap::new();
        translations.insert("expenseCategory.foo".to_string(), json!("Foo"));
        translations.insert("expenseCategory.bar".to_string(), json!("Bar"));
        translations.insert("status.open".to_string(), json!("Open"));
        translations.insert("status.closed".to_string(), json!("Closed"));
        
        let prefixes = extract_base_prefixes(&translations);
        
        assert!(prefixes.contains_key("expenseCategory"));
        assert!(prefixes.contains_key("status"));
        assert_eq!(prefixes.get("expenseCategory").unwrap().len(), 2);
        assert_eq!(prefixes.get("status").unwrap().len(), 2);
    }
    
    #[test]
    fn test_dynamic_pattern_matching() {
        let mut translations = std::collections::HashMap::new();
        translations.insert("expenseCategory.foo".to_string(), json!("Foo"));
        translations.insert("expenseCategory.bar".to_string(), json!("Bar"));
        translations.insert("status.open".to_string(), json!("Open"));
        
        let prefixes = extract_base_prefixes(&translations);
        let patterns = compile_dynamic_patterns(&prefixes);
        
        // Test that pattern matches dynamic usage
        let test_code1 = "const key = `expenseCategory.${variable}`;";
        let test_code2 = "t('status.${item.status}', { ns: 'App' });";
        let test_code3 = "Reimbursement:expenseCategory.${reimbursement.expenseCategory}";
        
        let expense_category_pattern = patterns.iter()
            .find(|(p, _)| p == "expenseCategory")
            .map(|(_, re)| re);
        let status_pattern = patterns.iter()
            .find(|(p, _)| p == "status")
            .map(|(_, re)| re);
        
        if let Some(pattern) = expense_category_pattern {
            assert!(pattern.is_match(test_code1), "Should match template literal");
            assert!(pattern.is_match(test_code3), "Should match namespace format");
        }
        
        if let Some(pattern) = status_pattern {
            assert!(pattern.is_match(test_code2), "Should match function call with t()");
        }
    }
}


