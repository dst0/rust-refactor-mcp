use syn::Item;
pub fn item_type(item: &Item) -> &'static str {
    match item {
        Item::Struct(_) => "struct",
        Item::Enum(_) => "enum",
        Item::Fn(_) => "fn",
        Item::Trait(_) => "trait",
        Item::Impl(_) => "impl",
        _ => "item",
    }
}
