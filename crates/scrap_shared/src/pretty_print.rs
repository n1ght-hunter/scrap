/// A trait for pretty-printing objects.
/// This is used for outputting human-readable representations of AST nodes and other structures.
pub trait PrettyPrint {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        self.pretty_print_indent(f, 0)
    }

    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result;

    fn pretty_to_string(&self) -> String {
        let mut s = String::new();
        self.pretty_print(&mut s).unwrap();
        s
    }

    fn print(&self) {
        println!("{}", self.pretty_to_string());
    }

    /// Helper method to write indentation
    fn write_indent(f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        for _ in 0..indent {
            write!(f, "  ")?;
        }
        Ok(())
    }
}
