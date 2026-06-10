#[test]
fn test_parse_impl() {
    let s = "impl<F, ReqBody, Ret, ResBody, E> Service<Request<ReqBody>> for ServiceFn<F, ReqBody> {}";
    let item: syn::Item = syn::parse_str(s).unwrap();
    if let syn::Item::Impl(i) = item {
        if let syn::Type::Path(tp) = &*i.self_ty {
            if let Some(segment) = tp.path.segments.last() {
                println!("MATCHED: {}", segment.ident.to_string());
                assert_eq!(segment.ident.to_string(), "ServiceFn");
                return;
            }
        }
    }
    panic!("Did not match");
}
