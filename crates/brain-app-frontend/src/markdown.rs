use pulldown_cmark::{html, Options, Parser};

pub fn render_markdown(input: &str) -> String {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(input, options);
    let mut html_output = String::with_capacity(input.len() * 2);
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let result = render_markdown("**bold** and *italic*");
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_code_block() {
        let result = render_markdown("```rust\nfn main() {}\n```");
        assert!(result.contains("<code"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let result = render_markdown(md);
        assert!(result.contains("<table>"));
    }

    #[test]
    fn test_empty_input() {
        let result = render_markdown("");
        assert!(result.is_empty());
    }
}
