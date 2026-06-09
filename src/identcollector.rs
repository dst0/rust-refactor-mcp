use std::collections::HashSet;
use syn::visit::Visit;
use syn::{File, Item, ItemFn, ItemUse, Type, UseTree};
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
        self.found.insert(attr.path().get_ident().map(|id| id.to_string()).unwrap_or_default());
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
