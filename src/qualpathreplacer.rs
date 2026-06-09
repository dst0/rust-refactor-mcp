use syn::visit_mut::{self, VisitMut};
pub struct QualPathReplacer {
    pub old_mod: String,
    pub entity_name: String,
    pub new_mod: String,
    pub changed: bool,
}
impl VisitMut for QualPathReplacer {
    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.segments.len() == 2 && i.segments[0].ident == self.old_mod
            && i.segments[1].ident == self.entity_name
        {
            i.segments[0].ident = syn::Ident::new(
                &self.new_mod,
                i.segments[0].ident.span(),
            );
            self.changed = true;
        }
        visit_mut::visit_path_mut(self, i);
    }
}
