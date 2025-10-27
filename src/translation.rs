use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Load translation files from a path (can be a file or directory)
pub fn load_translations(path: &str) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let path = Path::new(path);

    if path.is_dir() {
        load_translations_from_dir(path)
    } else if path.is_file() {
        load_translation_file(path.to_str().unwrap())
    } else {
        Err(format!("Path does not exist: {}", path.display()).into())
    }
}

/// Load and merge all JSON files from a directory
fn load_translations_from_dir(
    dir: &Path,
) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let mut all_translations = HashMap::new();
    let mut found_files = false;

    // Get all JSON files in the directory
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(path_str) = path.to_str() {
                let translations = load_translation_file(path_str)?;

                // Merge with existing translations (later files override earlier ones)
                for (key, value) in translations {
                    all_translations.insert(key, value);
                }

                found_files = true;
            }
        }
    }

    if !found_files {
        return Err(format!("No JSON files found in directory: {}", dir.display()).into());
    }

    Ok(all_translations)
}

/// Load and flatten translation keys from a single JSON file
fn load_translation_file(
    file_path: &str,
) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let json: Value = serde_json::from_str(&content)?;

    Ok(flatten_json(json, String::new()))
}

/// Flatten a nested JSON structure into dot-notation keys
pub fn flatten_json(value: Value, prefix: String) -> HashMap<String, Value> {
    let mut result = HashMap::new();

    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key
                } else {
                    format!("{}.{}", prefix, key)
                };

                let sub_keys = flatten_json(val, new_prefix);
                result.extend(sub_keys);
            }
        }
        Value::String(_) => {
            result.insert(prefix, value);
        }
        Value::Number(_) => {
            result.insert(prefix, value);
        }
        Value::Bool(_) => {
            result.insert(prefix, value);
        }
        Value::Null => {
            result.insert(prefix, value);
        }
        Value::Array(arr) => {
            for (i, val) in arr.into_iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                let sub_keys = flatten_json(val, new_prefix);
                result.extend(sub_keys);
            }
        }
    }

    result
}

/// Remove unused keys from translation files while preserving order
pub fn remove_unused_keys(
    translation_path: &str,
    unused_keys: &[String],
    used_keys: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(translation_path);

    if path.is_dir() {
        remove_unused_from_directory(path, unused_keys, used_keys)
    } else if path.is_file() {
        remove_unused_from_file(path, unused_keys, used_keys)
    } else {
        Err(format!("Path does not exist: {}", path.display()).into())
    }
}

/// Remove unused keys from a single JSON file
fn remove_unused_from_file(
    file_path: &Path,
    unused_keys: &[String],
    used_keys: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let json: Value = serde_json::from_str(&content)?;

    let cleaned_json = remove_keys_from_value(json, unused_keys, used_keys)?;

    // Write back with pretty formatting and trailing newline (standard for code files)
    let updated_content = serde_json::to_string_pretty(&cleaned_json)?;
    fs::write(file_path, format!("{}\n", updated_content))?;

    Ok(())
}

/// Remove unused keys from directory of JSON files
fn remove_unused_from_directory(
    dir: &Path,
    unused_keys: &[String],
    used_keys: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            remove_unused_from_file(&path, unused_keys, used_keys)?;
        }
    }

    Ok(())
}

/// Recursively remove unused keys from a JSON value
fn remove_keys_from_value(
    value: Value,
    unused_keys: &[String],
    used_keys: &HashSet<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let unused_set: HashSet<&str> = unused_keys.iter().map(|s| s.as_str()).collect();

    fn should_keep_key(
        key_path: &str,
        used_keys: &HashSet<String>,
        unused_set: &HashSet<&str>,
    ) -> bool {
        // Check if this exact key is used
        if used_keys.contains(key_path) {
            return true;
        }

        // Check if any child key is used (e.g., if "user.name" is used, keep "user")
        for used_key in used_keys {
            if used_key.starts_with(&format!("{}.", key_path)) || used_key == key_path {
                return true;
            }
        }

        // Key is not in unused list, so keep it
        !unused_set.contains(key_path)
    }

    fn remove_recursive(
        value: Value,
        prefix: String,
        used_keys: &HashSet<String>,
        unused_set: &HashSet<&str>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        match value {
            Value::Object(map) => {
                let mut cleaned_map = serde_json::Map::new();

                for (key, val) in map {
                    let current_path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    // Recursively clean nested values
                    let cleaned_value =
                        remove_recursive(val, current_path.clone(), used_keys, unused_set)?;

                    // Only keep this key if it or any child is used
                    if should_keep_key(&current_path, used_keys, unused_set) {
                        cleaned_map.insert(key, cleaned_value);
                    }
                }

                Ok(Value::Object(cleaned_map))
            }
            Value::Array(arr) => {
                let mut cleaned_arr = Vec::new();

                for (i, val) in arr.into_iter().enumerate() {
                    let item_path = if prefix.is_empty() {
                        format!("[{}]", i)
                    } else {
                        format!("{}[{}]", prefix, i)
                    };

                    let cleaned_value =
                        remove_recursive(val, item_path.clone(), used_keys, unused_set)?;

                    // Only keep array items if they or their children are used
                    if should_keep_key(&item_path, used_keys, unused_set) {
                        cleaned_arr.push(cleaned_value);
                    } else {
                        // Keep the slot to preserve array structure, but we'll filter empty arrays
                        cleaned_arr.push(cleaned_value);
                    }
                }

                // Filter out null values from cleaned arrays
                let filtered_arr: Vec<Value> =
                    cleaned_arr.into_iter().filter(|v| !v.is_null()).collect();

                Ok(Value::Array(filtered_arr))
            }
            _ => Ok(value),
        }
    }

    remove_recursive(value, String::new(), used_keys, &unused_set)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_simple_object() {
        let json = serde_json::json!({
            "hello": "world",
            "foo": "bar"
        });
        let result = flatten_json(json, String::new());

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("hello"));
        assert!(result.contains_key("foo"));
    }

    #[test]
    fn test_flatten_nested_object() {
        let json = serde_json::json!({
            "user": {
                "name": "John",
                "age": 30
            }
        });
        let result = flatten_json(json, String::new());

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("user.name"));
        assert!(result.contains_key("user.age"));
    }

    #[test]
    fn test_flatten_array() {
        let json = serde_json::json!({
            "items": ["a", "b"]
        });
        let result = flatten_json(json, String::new());

        assert!(result.contains_key("items[0]"));
        assert!(result.contains_key("items[1]"));
    }
}
