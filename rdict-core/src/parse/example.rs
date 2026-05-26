use super::{ExamplePair, inner_text};
use scraper::ElementRef;

#[rustfmt::skip]
mod selectors {
    use std::sync::LazyLock;
    use crate::selector;
    use scraper::Selector;

    pub static EXAMPLE_MODULE: LazyLock<Selector> = selector!("div.blng_sents_part.dict-module");
    pub static EXAMPLE_ITEM:   LazyLock<Selector> = selector!("li.mcols-layout");
    pub static EXAMPLE_SOURCE:      LazyLock<Selector> = selector!("div.sen-eng");
    pub static EXAMPLE_TARGET:      LazyLock<Selector> = selector!("div.sen-ch");
}

/// Extract example sentence pairs from the `blng_sents_part` dict-module.
///
/// `sen-eng` maps to `source`, `sen-ch` maps to `target`.
/// Each direction decides which language is source vs target.
#[must_use]
pub fn extract_examples(body: &ElementRef) -> Vec<ExamplePair> {
    let mut examples = Vec::new();
    if let Some(module) = body.select(&selectors::EXAMPLE_MODULE).next() {
        for element in module.select(&selectors::EXAMPLE_ITEM) {
            let source = element
                .select(&selectors::EXAMPLE_SOURCE)
                .next()
                .map(|el| inner_text(&el))
                .unwrap_or_default();

            let target = element
                .select(&selectors::EXAMPLE_TARGET)
                .next()
                .map(|el| inner_text(&el))
                .unwrap_or_default();

            if !source.is_empty() && !target.is_empty() {
                examples.push(ExamplePair { source, target });
            }
        }
    }
    examples
}
