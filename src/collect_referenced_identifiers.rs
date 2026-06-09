use syn::visit::Visit;
use crate::identcollector::IdentCollector;
use std::collections::HashSet;
use syn::Item;
pub fn collect_referenced_identifiers(items: &[Item]) -> HashSet<String> {
    let mut visitor = IdentCollector {
        found: HashSet::new(),
    };
    for item in items {
        if matches!(item, Item::Use(_)) {
            continue;
        }
        visitor.visit_item(item);
    }
    visitor.found
}
