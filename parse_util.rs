use syn::{parse_str, Item};

fn get_item_name(item: &Item) -> Option<String> {
    match item {
        Item::Struct(s) => Some(s.ident.to_string()),
        Item::Impl(i) => {
            if let syn::Type::Path(tp) = &*i.self_ty {
                if let Some(segment) = tp.path.segments.last() {
                    return Some(segment.ident.to_string());
                }
            }
            None
        }
        _ => None,
    }
}

fn main() {
    let s = "impl<F, ReqBody, Ret, ResBody, E> Service<Request<ReqBody>> for ServiceFn<F, ReqBody> {}";
    let item: Item = parse_str(s).unwrap();
    println!("{:?}", get_item_name(&item));
}
