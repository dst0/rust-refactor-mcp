use std::collections::HashSet;
pub fn is_import_used(names: &[String], used_ids: &HashSet<String>) -> bool {
    for name in names {
        if used_ids.contains(name) {
            return true;
        }
        if name == "Spanned" && used_ids.contains("span") {
            return true;
        }
        if name == "Visit" {
            return true;
        }
        if name == "Deserialize" || name == "Serialize" {
            return true;
        }
    }
    false
}
