/// Strip FTS5 special syntax to prevent query errors from user input.
/// Removes: AND, OR, NOT, NEAR, quotes, asterisks, carets, parentheses.
pub fn sanitize_fts_query(input: &str) -> String {
    let result = input
        .replace('"', "")
        .replace('*', "")
        .replace('^', "")
        .replace('(', "")
        .replace(')', "");

    // Remove FTS5 boolean operators (case-sensitive in FTS5)
    let words: Vec<&str> = result
        .split_whitespace()
        .filter(|w| !matches!(*w, "AND" | "OR" | "NOT" | "NEAR"))
        .collect();

    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_plain_text() {
        assert_eq!(sanitize_fts_query("hello world"), "hello world");
    }

    #[test]
    fn sanitize_strips_fts_operators() {
        assert_eq!(sanitize_fts_query("hello AND world"), "hello world");
        assert_eq!(sanitize_fts_query("test*"), "test");
        assert_eq!(sanitize_fts_query("\"exact match\""), "exact match");
    }

    #[test]
    fn sanitize_empty() {
        assert_eq!(sanitize_fts_query(""), "");
        assert_eq!(sanitize_fts_query("   "), "");
    }
}
