// Utilities for building and parsing file URLs

/// Build URL path in the format: "<unique_id>_<url_safe_filename>"
/// - Replaces spaces in filename with underscores for URL safety
pub fn build_url_path(unique_id: &str, file_name: &str) -> String {
    let safe = file_name.replace(' ', "_");
    format!("{}_{}", unique_id, safe)
}

/// Extract the unique id from a path of the form "<unique_id>_<filename>".
/// Since unique_id is always 8 characters (from nanoid!(8)), we can extract it reliably.
/// Falls back to checking for underscore at position 8 if present.
pub fn extract_id_from_path(path: &str) -> &str {
    // Check if there's an underscore at position 8 (after 8-char nanoid)
    if path.len() > 8 && path.chars().nth(8) == Some('_') {
        &path[..8]
    } else if let Some(pos) = path.find('_') {
        // Fallback: use first underscore (backward compatibility for old links)
        &path[..pos]
    } else {
        // No underscore: treat entire path as ID (backward compatibility)
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
    fn test_extract_id_with_underscores_in_filename() {
        // 8-char nanoid + underscore + filename with underscores
        let id = extract_id_from_path("K_zO5rG8_270507.mp4");
        assert_eq!(id, "K_zO5rG8");
    }

    #[test]
    fn test_extract_id_from_path_with_multiple_underscores() {
        let id = extract_id_from_path("rFgFEY12_file_name_with_many_underscores.txt");
        assert_eq!(id, "rFgFEY12");
    }

    #[test]
    fn test_extract_id_from_path_no_underscore() {
        let id = extract_id_from_path("legacyid");
        assert_eq!(id, "legacyid");
    }
}
