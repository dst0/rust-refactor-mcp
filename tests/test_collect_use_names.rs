#[test]
fn test_collect() {
    let s = "pub use crate::body::incoming::sender::Sender;";
    let item: syn::Item = syn::parse_str(s).unwrap();
    if let syn::Item::Use(u) = item {
        let names = rust_refactor_mcp::extract::collect_use_names(&u.tree);
        println!("Names: {:?}", names);
        assert_eq!(names, vec!["Sender"]);
    }
}
