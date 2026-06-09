use syn::Item;
pub fn make_item_pub(item: &mut Item) {
    let pub_vis = syn::Visibility::Public(syn::token::Pub::default());
    match item {
        Item::Fn(f) => f.vis = pub_vis,
        Item::Struct(s) => {
            s.vis = pub_vis.clone();
            match &mut s.fields {
                syn::Fields::Named(fields) => {
                    for field in &mut fields.named {
                        field.vis = pub_vis.clone();
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for field in &mut fields.unnamed {
                        field.vis = pub_vis.clone();
                    }
                }
                syn::Fields::Unit => {}
            }
        }
        Item::Enum(e) => {
            e.vis = pub_vis.clone();
            for variant in &mut e.variants {
                match &mut variant.fields {
                    syn::Fields::Named(fields) => {
                        for field in &mut fields.named {
                            field.vis = pub_vis.clone();
                        }
                    }
                    syn::Fields::Unnamed(fields) => {
                        for field in &mut fields.unnamed {
                            field.vis = pub_vis.clone();
                        }
                    }
                    syn::Fields::Unit => {}
                }
            }
        }
        Item::Trait(t) => t.vis = pub_vis,
        Item::Type(t) => t.vis = pub_vis,
        Item::Const(c) => c.vis = pub_vis,
        Item::Static(s) => s.vis = pub_vis,
        Item::Mod(m) => m.vis = pub_vis,
        _ => {}
    }
}
