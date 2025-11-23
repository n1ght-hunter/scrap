//! Pretty-printing routines for various compiler representations.

use scrap_ast::Can;
use scrap_ast_lowering::LoweredIr;
use scrap_shared::pretty_print::PrettyPrint;

use crate::args::{PrettyOut, UnPrettyOut};

/// The type of pretty-printing to perform.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PpMode {
    /// Print the AST (Abstract Syntax Tree) in a human-readable format
    PrettyAst,
    /// Print the AST in debug format
    DebugAst,
    /// Print the IR (Intermediate Representation) in debug format
    DebugIr,
    /// Print the IR (Intermediate Representation) in a human-readable format
    PrettyIr,
}

impl PpMode {
    /// Returns true if this mode requires AST to be available
    pub fn needs_ast(&self) -> bool {
        matches!(self, PpMode::PrettyAst | PpMode::DebugAst)
    }

    /// Returns true if this mode requires IR to be available
    pub fn needs_ir(&self) -> bool {
        matches!(self, PpMode::DebugIr | PpMode::PrettyIr)
    }

    /// Determine the pretty-print mode from command-line arguments
    pub fn determine_pp_mode(args: &crate::args::Args) -> Option<PpMode> {
        if matches!(args.pretty_out, Some(PrettyOut::Ast)) {
            Some(PpMode::PrettyAst)
        } else if matches!(args.pretty_out, Some(PrettyOut::IR)) {
            Some(PpMode::PrettyIr)
        } else if matches!(args.unpretty_out, Some(UnPrettyOut::Ast)) {
            Some(PpMode::DebugAst)
        } else if matches!(args.unpretty_out, Some(UnPrettyOut::SIR)) {
            Some(PpMode::DebugIr)
        } else {
            None
        }
    }
}

/// Print the compilation result in the specified format
pub fn print<'db>(
    db: &'db dyn scrap_shared::Db,
    mode: PpMode,
    ast: &Can<'db>,
    ir: Option<LoweredIr<'db>>,
) {
    match mode {
        PpMode::PrettyAst => {
            ast.print();
        }
        PpMode::DebugAst => {
            println!("{:#?}", ast);
        }
        PpMode::DebugIr => {
            if let Some(ir) = ir {
                println!("{:#?}", ir.can(db));
            } else {
                eprintln!("Error: IR not available");
            }
        }
        PpMode::PrettyIr => {
            if let Some(ir) = ir {
                let output = scrap_ir::print_can(db, ir.can(db));
                print!("{}", output);
            } else {
                eprintln!("Error: IR not available");
            }
        }
    }
}
