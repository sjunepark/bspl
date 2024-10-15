use crate::error::ParseIntError;
use crate::SmesError;
use scraper::element_ref::Text;

pub(crate) fn join_text_nodes(node: Text) -> String {
    node.collect()
}

/// Parse a string into a number, ignoring commas.
///
/// Returns 0 if the string is empty.
pub(crate) fn parse_comma_sep_digit(s: &str) -> Result<i64, SmesError> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(0);
    }
    let s = s.replace(",", "");
    Ok(s.parse::<i64>().map_err(|e| ParseIntError {
        source: Some(Box::new(e)),
        value: s,
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::{Html, Selector};

    #[test]
    fn join_text_nodes_should_work() {
        tracing_setup::span!("tests");
        let html = Html::parse_fragment("<p>This is <strong>important</strong> text.</p>");
        let text_nodes = html
            .select(&Selector::parse("p").unwrap())
            .map(|node| join_text_nodes(node.text()))
            .collect::<Vec<_>>();
        assert_eq!(text_nodes, vec!["This is important text."]);
    }
}
