use std::collections::HashSet;
use syn::visit::Visit;
use syn::Type;
pub struct IdentCollector {
    pub found: HashSet<String>,
}
impl<'a> Visit<'a> for IdentCollector {
    fn visit_ident(&mut self, id: &'a syn::Ident) {
        self.found.insert(id.to_string());
        syn::visit::visit_ident(self, id);
    }
    fn visit_expr_method_call(&mut self, i: &'a syn::ExprMethodCall) {
        self.found.insert(i.method.to_string());
        syn::visit::visit_expr_method_call(self, i);
    }
    fn visit_attribute(&mut self, attr: &'a syn::Attribute) {
        self.found.insert(
            attr.path()
                .get_ident()
                .map(|id| id.to_string())
                .unwrap_or_default(),
        );
        match &attr.meta {
            syn::Meta::Path(_) => {}
            syn::Meta::List(list) => {
                self.scan_token_stream(list.tokens.clone());
            }
            syn::Meta::NameValue(nv) => {
                self.visit_expr(&nv.value);
            }
        }
        syn::visit::visit_attribute(self, attr);
    }
    fn visit_type(&mut self, ty: &'a Type) {
        syn::visit::visit_type(self, ty);
    }
    fn visit_expr(&mut self, expr: &'a syn::Expr) {
        syn::visit::visit_expr(self, expr);
    }
    fn visit_expr_macro(&mut self, mac: &'a syn::ExprMacro) {
        syn::visit::visit_macro(self, &mac.mac);
        self.scan_token_stream(mac.mac.tokens.clone());
    }
    fn visit_stmt_macro(&mut self, mac: &'a syn::StmtMacro) {
        syn::visit::visit_macro(self, &mac.mac);
        self.scan_token_stream(mac.mac.tokens.clone());
    }
    fn visit_item_macro(&mut self, mac: &'a syn::ItemMacro) {
        syn::visit::visit_macro(self, &mac.mac);
        self.scan_token_stream(mac.mac.tokens.clone());
    }
}
impl IdentCollector {
    fn scan_token_stream(&mut self, tokens: proc_macro2::TokenStream) {
        for tok in tokens {
            match tok {
                proc_macro2::TokenTree::Ident(id) => {
                    self.found.insert(id.to_string());
                }
                proc_macro2::TokenTree::Group(g) => {
                    self.scan_token_stream(g.stream());
                }
                _ => {}
            }
        }
    }
}

/// Collects first-segment identifiers from multi-segment paths (e.g. `dispatch` from `dispatch::Sender`).
/// These are bare module names that may need explicit `use` imports when moved to a different scope.
pub struct BarePathCollector {
    pub bare_modules: HashSet<String>,
}
impl<'a> Visit<'a> for BarePathCollector {
    fn visit_path(&mut self, path: &'a syn::Path) {
        if path.segments.len() >= 2 {
            let first = path.segments[0].ident.to_string();
            // Only collect lowercase (module-like) names, skip type names and language keywords
            if first
                .chars()
                .next()
                .map(|c| c.is_lowercase())
                .unwrap_or(false)
                && !matches!(
                    first.as_str(),
                    "crate" | "self" | "super" | "std" | "core" | "alloc" | "r#"
                )
            {
                self.bare_modules.insert(first);
            }
        }
        syn::visit::visit_path(self, path);
    }
}
