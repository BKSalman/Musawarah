use regex::Regex;

pub fn slugify(title: &str) -> String {
    let re = Regex::new(r"[-/]").expect("valid regex for dash and slash");
    let title = re.replace_all(title.trim(), "_");

    let re = Regex::new(r"[^_\p{L}\p{N}\s]").expect(
        "valid regex for characters that are not underscores, letters, numbers, or whitespaces",
    );
    let title = re.replace_all(&title, "");

    let re = Regex::new(r"[_\s]+").expect("valid regex for multiple underscores and whitespaces");
    let title = re.replace_all(&title, "_");

    title.to_lowercase()
}
