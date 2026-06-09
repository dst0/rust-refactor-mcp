pub use crate::foo::Foo;
pub fn baz(f: &Foo) {
    println!("{}", f.value);
}
