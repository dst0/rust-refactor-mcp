use syn::visit::Visit;
use syn::{ItemFn, Type};
pub struct NameVisitor {
    name: String,
    found: bool,
}
impl NameVisitor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            found: false,
        }
    }
    pub fn visit_fn(&mut self, item: &ItemFn) -> bool {
        self.found = false;
        self.visit_item_fn(item);
        self.found
    }
}
impl<'a> Visit<'a> for NameVisitor {
    fn visit_ident(&mut self, id: &'a syn::Ident) {
        if *id == self.name {
            self.found = true;
        }
        syn::visit::visit_ident(self, id);
    }
    fn visit_type(&mut self, ty: &'a Type) {
        if let Type::Path(tp) = ty {
            if let Some(seg) = tp.path.segments.last() {
                if seg.ident == self.name {
                    self.found = true;
                }
            }
        }
        syn::visit::visit_type(self, ty);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::{
        collect_referenced_identifiers, extract_entity, find_extracted_indices, item_type,
    };
    use crate::format_ty_name::format_ty_name;
    use crate::has_use_ref::has_use_ref;
    use syn::{File, Item, Type, UseTree};
    pub fn make_source(code: &str) -> File {
        syn::parse_file(code).expect("parse")
    }
    #[test]
    pub fn find_struct() {
        let source = "struct Foo { x: i32 }";
        let parsed = make_source(source);
        let indices = find_extracted_indices(&parsed, "Foo", None);
        assert!(!indices.is_empty());
    }
    #[test]
    pub fn find_enum() {
        let source = "enum Bar { A, B }";
        let parsed = make_source(source);
        let indices = find_extracted_indices(&parsed, "Bar", None);
        assert!(!indices.is_empty());
    }
    #[test]
    pub fn find_fn() {
        let source = "fn baz() {}";
        let parsed = make_source(source);
        let indices = find_extracted_indices(&parsed, "baz", None);
        assert!(!indices.is_empty());
    }
    #[test]
    pub fn find_trait() {
        let source = "trait Qux {}";
        let parsed = make_source(source);
        let indices = find_extracted_indices(&parsed, "Qux", None);
        assert!(!indices.is_empty());
    }
    #[test]
    pub fn find_struct_preferred_over_impl() {
        let source = "struct Foo;\nimpl Foo {}";
        let parsed = make_source(source);
        let indices = find_extracted_indices(&parsed, "Foo", None);
        assert!(indices.contains(&0));
    }
    #[test]
    pub fn collect_impls_multiple() {
        let source = "struct Foo {}\nimpl Foo { fn a() {} }\nimpl Default for Foo { fn default() -> Self { Foo } }";
        let parsed = make_source(source);
        let indices = find_extracted_indices(&parsed, "Foo", None);
        assert_eq!(indices.len(), 3);
    }
    #[test]
    pub fn item_type_struct() {
        let source = "struct Foo {}";
        let parsed = make_source(source);
        let item = &parsed.items[0];
        assert_eq!(item_type(item), "struct");
    }
    #[test]
    pub fn format_ty_simple() {
        let ty: Type = syn::parse_str("Foo").unwrap();
        assert_eq!(format_ty_name(&ty), "Foo");
    }
    #[test]
    pub fn name_visitor_finds_reference() {
        let source = "fn test() { let x: Foo = Foo; }";
        let parsed = make_source(source);
        let mut v = NameVisitor::new("Foo");
        for item in &parsed.items {
            if let Item::Fn(f) = item {
                assert!(v.visit_fn(f));
            }
        }
    }
    #[test]
    pub fn name_visitor_no_reference() {
        let source = "fn test() { let x: Bar = Bar; }";
        let parsed = make_source(source);
        let mut v = NameVisitor::new("Foo");
        for item in &parsed.items {
            if let Item::Fn(f) = item {
                assert!(!v.visit_fn(f));
            }
        }
    }
    #[test]
    pub fn extract_struct_basic() {
        let source = "struct Foo { x: i32 }\nfn bar() {}";
        let tmp = std::env::temp_dir().join("rust_refactor_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let result = extract_entity(
            &source,
            "Foo",
            tmp.to_str().unwrap(),
            None,
            None,
            None,
            true,
        )
        .unwrap();
        assert!(result.items_extracted.contains(&"struct: Foo".to_string()));
        assert!(result.new_file_path.ends_with("foo.rs"));
        std::fs::remove_dir_all(&tmp).ok();
    }
    #[test]
    pub fn extract_with_impl() {
        let source = "struct Foo { x: i32 }\nimpl Foo { fn new() -> Self { Foo { x: 0 } } }";
        let tmp = std::env::temp_dir().join("rust_refactor_test2");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let result = extract_entity(
            &source,
            "Foo",
            tmp.to_str().unwrap(),
            None,
            None,
            None,
            true,
        )
        .unwrap();
        assert_eq!(result.items_extracted.len(), 2);
        std::fs::remove_dir_all(&tmp).ok();
    }
    #[test]
    pub fn extract_enum() {
        let source = "enum Color { Red, Green, Blue }";
        let tmp = std::env::temp_dir().join("rust_refactor_test3");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let result = extract_entity(
            &source,
            "Color",
            tmp.to_str().unwrap(),
            None,
            None,
            None,
            true,
        )
        .unwrap();
        assert_eq!(result.items_extracted[0], "enum: Color");
        std::fs::remove_dir_all(&tmp).ok();
    }
    #[test]
    pub fn extract_not_found() {
        let source = "struct Foo {}";
        let tmp = std::env::temp_dir().join("rust_refactor_test4");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let err = extract_entity(
            &source,
            "Bar",
            tmp.to_str().unwrap(),
            None,
            None,
            None,
            true,
        )
        .unwrap_err();
        assert!(err.contains("not found"));
        std::fs::remove_dir_all(&tmp).ok();
    }
    #[test]
    pub fn extract_invalid_syntax() {
        let err = extract_entity("not rust !!!", "Foo", ".", None, None, None, true).unwrap_err();
        assert!(err.contains("Parse error"));
    }
    #[test]
    pub fn collect_referenced_ids() {
        let source = "use foo::Bar;\nfn test() { let x: Bar = Bar::new(); }";
        let parsed = make_source(source);
        let ids = collect_referenced_identifiers(&parsed.items);
        assert!(ids.contains("Bar"));
    }
    #[test]
    pub fn has_use_ref_positive() {
        let tree: UseTree = syn::parse_str("simple::Point").unwrap();
        assert!(has_use_ref("Point", &tree));
    }
    #[test]
    pub fn has_use_ref_negative() {
        let tree: UseTree = syn::parse_str("simple::Other").unwrap();
        assert!(!has_use_ref("Point", &tree));
    }
}
