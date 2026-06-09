use syn::Type;
pub fn format_ty_name(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => {
            tp.path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_else(|| {
                    tp.path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default()
                })
        }
        _ => format!("{:?}", ty),
    }
}
