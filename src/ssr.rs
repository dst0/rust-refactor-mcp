use quote::quote;
use syn::{
    visit_mut::{self, VisitMut},
    Expr, File, Type,
};

pub struct SSRVisitor {
    pub pattern: String,
    pub replacement: String,
    pub changed: bool,
}

impl VisitMut for SSRVisitor {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        let expr_str = quote!(#i).to_string().replace(" ", "");
        let pattern_str = self.pattern.replace(" ", "");

        if expr_str == pattern_str {
            if let Ok(replacement) = syn::parse_str::<Expr>(&self.replacement) {
                *i = replacement;
                self.changed = true;
            }
        }
        visit_mut::visit_expr_mut(self, i);
    }

    fn visit_type_mut(&mut self, i: &mut Type) {
        let type_str = quote!(#i).to_string().replace(" ", "");
        let pattern_str = self.pattern.replace(" ", "");

        if type_str == pattern_str {
            if let Ok(replacement) = syn::parse_str::<Type>(&self.replacement) {
                *i = replacement;
                self.changed = true;
            }
        }
        visit_mut::visit_type_mut(self, i);
    }
}

pub fn ssr(file_path: &str, pattern: &str, replacement: &str) -> Result<bool, String> {
    let source = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut parsed: File = syn::parse_file(&source).map_err(|e| e.to_string())?;
    let mut visitor = SSRVisitor {
        pattern: pattern.to_string(),
        replacement: replacement.to_string(),
        changed: false,
    };
    visitor.visit_file_mut(&mut parsed);
    if visitor.changed {
        let new_content = prettyplease::unparse(&parsed);
        std::fs::write(file_path, new_content).map_err(|e| e.to_string())?;
        let _ = std::process::Command::new("rustfmt")
            .args(["--edition", "2024", file_path])
            .status();
        Ok(true)
    } else {
        Ok(false)
    }
}
