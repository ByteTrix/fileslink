// Utilities for building and parsing file URLs

/// Build URL path in the format: "<unique_id>_<url_safe_filename>"
/// - Replaces spaces in filename with underscores for URL safety
pub fn build_url_path(unique_id: &str, file_name: &str) -> String {
    let safe = file_name.replace(' ', "_");
    format!("{}_{}", unique_id, safe)
}

/// Extract the unique id from a path of the form "<unique_id>_<filename>".
/// Falls back to the full string if no underscore is present (backward compatibility).
pub fn extract_id_from_path(path: &str) -> &str {
    if let Some(pos) = path.find('_') {
        &path[..pos]
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_path_spaces_replaced() {
        let p = build_url_path("abc123", "My File Name.pdf");
        assert_eq!(p, "abc123_My_File_Name.pdf");
    }

    #[test]
    fn test_build_url_path_underscores_kept() {
        let p = build_url_path("id8", "hello_world.txt");
        assert_eq!(p, "id8_hello_world.txt");
    }

    #[test]
    fn test_extract_id_from_path_with_filename() {
        let id = extract_id_from_path("ZvOWMhv1_report.pdf");
        assert_eq!(id, "ZvOWMhv1");
    }

    #[test]
    fn test_extract_id_from_path_no_underscore() {
        let id = extract_id_from_path("legacyid");
        assert_eq!(id, "legacyid");
    }
}
