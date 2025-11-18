/// A trait for pretty-printing objects.
/// This is used for outputing human-readable representations of AST nodes and other structures.
pub trait PrettyPrint {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result;

    fn pretty_to_string(&self) -> String {
        let mut s = String::new();
        self.pretty_print(&mut s).unwrap();
        s
    }

    fn print(&self) {
        println!("{}", self.pretty_to_string());
    }
}
