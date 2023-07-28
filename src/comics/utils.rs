use regex::Regex;

pub fn slugify(title: &str) -> String {
    // replace separators into underscores
    let re = Regex::new(r"[-/]").unwrap();
    let title = re.replace_all(title.trim(), "_");

    // remove all characters that are not underscores, letters, numbers, or whitespaces
    let re = Regex::new(r"[^_\p{L}\p{N}\s]").unwrap();
    let title = re.replace_all(&title, "");

    // Replace all underscores and whitespace by a single underscore
    let re = Regex::new(r"[_\s]+").unwrap();
    let title = re.replace_all(&title, "_");

    title.to_lowercase()
}
