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

/// Compilation output to be printed
pub enum CompilationOutput<'db> {
    Ast(Can<'db>),
    Ir(LoweredIr<'db>),
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
        } else if matches!(args.pretty_out, Some(PrettyOut::SIR)) {
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
    output: CompilationOutput<'db>,
) {
    match (mode, output) {
        (PpMode::PrettyAst, CompilationOutput::Ast(ast)) => {
            ast.print();
        }
        (PpMode::DebugAst, CompilationOutput::Ast(ast)) => {
            println!("{:#?}", ast);
        }
        (PpMode::DebugIr, CompilationOutput::Ir(ir)) => {
            println!("{:#?}", ir.can(db));
        }
        (PpMode::PrettyIr, CompilationOutput::Ir(ir)) => {
            let output = scrap_ir::print_can(db, ir.can(db));
            print!("{}", output);
        }
        (PpMode::PrettyAst | PpMode::DebugAst, CompilationOutput::Ir(_)) => {
            eprintln!("Error: AST printing mode but IR was provided");
        }
        (PpMode::PrettyIr | PpMode::DebugIr, CompilationOutput::Ast(_)) => {
            eprintln!("Error: IR printing mode but AST was provided");
        }
    }
}
