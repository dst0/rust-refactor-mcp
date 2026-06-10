use syn::visit::Visit;
use syn::{ExprField, ExprStruct, PatStruct, Macro};

pub struct FieldUsageVisitor<'a> {
    pub target_fields: Vec<&'a str>,
    pub used_fields: Vec<String>,
}

impl<'a> FieldUsageVisitor<'a> {
    pub fn new(target_fields: Vec<&'a str>) -> Self {
        Self {
            target_fields,
            used_fields: Vec::new(),
        }
    }
}

impl<'ast, 'a> Visit<'ast> for FieldUsageVisitor<'a> {
    fn visit_expr_field(&mut self, node: &'ast ExprField) {
        if let syn::Member::Named(ident) = &node.member {
            let field_name = ident.to_string();
            if self.target_fields.contains(&field_name.as_str()) && !self.used_fields.contains(&field_name) {
                self.used_fields.push(field_name);
            }
        }
        syn::visit::visit_expr_field(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast ExprStruct) {
        for field in &node.fields {
            if let syn::Member::Named(ident) = &field.member {
                let field_name = ident.to_string();
                if self.target_fields.contains(&field_name.as_str()) && !self.used_fields.contains(&field_name) {
                    self.used_fields.push(field_name);
                }
            }
        }
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_pat_struct(&mut self, node: &'ast PatStruct) {
        for field in &node.fields {
            if let syn::Member::Named(ident) = &field.member {
                let field_name = ident.to_string();
                if self.target_fields.contains(&field_name.as_str()) && !self.used_fields.contains(&field_name) {
                    self.used_fields.push(field_name);
                }
            }
        }
        syn::visit::visit_pat_struct(self, node);
    }
}

pub struct MacroUsageVisitor<'a> {
    pub target_macros: Vec<&'a str>,
    pub used_macros: Vec<String>,
}

impl<'a> MacroUsageVisitor<'a> {
    pub fn new(target_macros: Vec<&'a str>) -> Self {
        Self {
            target_macros,
            used_macros: Vec::new(),
        }
    }
}

impl<'ast, 'a> Visit<'ast> for MacroUsageVisitor<'a> {
    fn visit_macro(&mut self, node: &'ast Macro) {
        if let Some(ident) = node.path.get_ident() {
            let mac_name = ident.to_string();
            if self.target_macros.contains(&mac_name.as_str()) && !self.used_macros.contains(&mac_name) {
                self.used_macros.push(mac_name);
            }
        }
        syn::visit::visit_macro(self, node);
    }
}
