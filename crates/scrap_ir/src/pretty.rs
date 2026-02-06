//! Pretty-printing for IR structures

use crate::ir::*;
use std::fmt::Write;

pub struct IrPrinter<'a, 'db> {
    db: &'db dyn scrap_shared::Db,
    output: String,
    indent: usize,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, 'db> IrPrinter<'a, 'db> {
    pub fn new(db: &'db dyn scrap_shared::Db) -> Self {
        Self {
            db,
            output: String::new(),
            indent: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn print_can(&mut self, can: Can<'db>) -> String {
        for module in can.modules(self.db) {
            self.print_module(*module);
            writeln!(self.output).unwrap();
        }
        self.output.clone()
    }

    fn print_module(&mut self, module: Module<'db>) {
        writeln!(
            self.output,
            "module {} {{",
            module.id(self.db).path(self.db)
        )
        .unwrap();
        self.indent += 1;

        for item in module.items(self.db) {
            match item {
                Items::Function(func) => self.print_function(*func),
                Items::Struct(s) => self.print_struct(*s),
                Items::Enum(e) => self.print_enum(*e),
            }
            writeln!(self.output).unwrap();
        }

        self.indent -= 1;
        writeln!(self.output, "}}").unwrap();
    }

    fn print_function(&mut self, func: Function<'db>) {
        let sig = func.signature(self.db);
        self.write_indent();
        write!(self.output, "fn {}", sig.name(self.db).text(self.db)).unwrap();

        // Parameters (referenced by local ID: _1, _2, ...)
        write!(self.output, "(").unwrap();
        let params = sig.params(self.db);
        for (i, ty) in params.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ").unwrap();
            }
            write!(self.output, "_{}: ", i + 1).unwrap();
            self.print_type(ty);
        }
        write!(self.output, ")").unwrap();

        // Return type
        let ret_ty = sig.return_ty(self.db);
        write!(self.output, " -> ").unwrap();
        self.print_type(&ret_ty);

        writeln!(self.output, " {{").unwrap();
        self.indent += 1;

        self.print_body(func.body(self.db));

        self.indent -= 1;
        self.write_indent();
        writeln!(self.output, "}}").unwrap();
    }

    fn print_struct(&mut self, s: Struct<'db>) {
        self.write_indent();
        writeln!(self.output, "struct {} {{", s.name(self.db).text(self.db)).unwrap();
        self.indent += 1;

        for (name, ty) in s.fields(self.db) {
            self.write_indent();
            write!(self.output, "{}: ", name.text(self.db)).unwrap();
            self.print_type(ty);
            writeln!(self.output, ",").unwrap();
        }

        self.indent -= 1;
        self.write_indent();
        writeln!(self.output, "}}").unwrap();
    }

    fn print_enum(&mut self, e: Enum<'db>) {
        self.write_indent();
        writeln!(self.output, "enum {} {{", e.name(self.db).text(self.db)).unwrap();
        self.indent += 1;

        for variant in e.variants(self.db) {
            self.write_indent();
            match variant {
                EnumVariant::Unit(name) => {
                    writeln!(self.output, "{},", name.text(self.db)).unwrap();
                }
                EnumVariant::Tuple(name, types) => {
                    write!(self.output, "{}(", name.text(self.db)).unwrap();
                    for (i, ty) in types.iter().enumerate() {
                        if i > 0 {
                            write!(self.output, ", ").unwrap();
                        }
                        self.print_type(ty);
                    }
                    writeln!(self.output, "),").unwrap();
                }
                EnumVariant::Struct(name, fields) => {
                    writeln!(self.output, "{} {{", name.text(self.db)).unwrap();
                    self.indent += 1;
                    for (field_name, ty) in fields {
                        self.write_indent();
                        write!(self.output, "{}: ", field_name.text(self.db)).unwrap();
                        self.print_type(ty);
                        writeln!(self.output, ",").unwrap();
                    }
                    self.indent -= 1;
                    self.write_indent();
                    writeln!(self.output, "}},").unwrap();
                }
            }
        }

        self.indent -= 1;
        self.write_indent();
        writeln!(self.output, "}}").unwrap();
    }

    fn print_body(&mut self, body: Body<'db>) {
        let locals = body.local_decls(self.db);
        let param_count = body.param_count(self.db);

        // Print debug info for named locals (maps names to local IDs)
        for (i, local) in locals.iter().enumerate() {
            if let Some(name) = local.name(self.db) {
                self.write_indent();
                writeln!(self.output, "debug {} => _{};", name.text(self.db), i).unwrap();
            }
        }

        // Print let declarations for return place and non-param locals (skip params)
        for (i, local) in locals.iter().enumerate() {
            // Skip params (_1 through _param_count) — they're in the signature
            if i >= 1 && i <= param_count {
                continue;
            }
            self.write_indent();
            write!(self.output, "let _{}: ", i).unwrap();
            self.print_type(&local.ty(self.db));
            writeln!(self.output, ";").unwrap();
        }

        if !locals.is_empty() {
            writeln!(self.output).unwrap();
        }

        // Print basic blocks
        let blocks = body.blocks(self.db);
        for (i, block) in blocks.iter().enumerate() {
            self.write_indent();
            writeln!(self.output, "bb{}: {{", i).unwrap();
            self.indent += 1;

            for stmt in block.statements(self.db) {
                self.print_statement(*stmt);
            }

            self.print_terminator(&block.terminator(self.db));

            self.indent -= 1;
            self.write_indent();
            writeln!(self.output, "}}").unwrap();
        }
    }

    fn print_statement(&mut self, stmt: Statement<'db>) {
        self.write_indent();
        match stmt.kind(self.db) {
            StatementKind::Assign(place, rvalue) => {
                self.print_place(&place);
                write!(self.output, " = ").unwrap();
                self.print_rvalue(&rvalue);
                writeln!(self.output, ";").unwrap();
            }
        }
    }

    fn print_terminator(&mut self, terminator: &Terminator<'db>) {
        self.write_indent();
        match terminator {
            Terminator::Goto { target } => {
                writeln!(self.output, "goto -> bb{};", target.0).unwrap();
            }
            Terminator::SwitchInt { discr, targets } => {
                write!(self.output, "switchInt(").unwrap();
                self.print_operand(discr);
                write!(self.output, ") -> [").unwrap();
                for (i, target) in targets.iter().enumerate() {
                    if i > 0 {
                        write!(self.output, ", ").unwrap();
                    }
                    write!(self.output, "bb{}", target.0).unwrap();
                }
                writeln!(self.output, "];").unwrap();
            }
            Terminator::Return => {
                writeln!(self.output, "return;").unwrap();
            }
            Terminator::Call {
                func,
                args,
                destination,
                target,
            } => {
                self.print_place(destination);
                write!(self.output, " = ").unwrap();
                self.print_operand(func);
                write!(self.output, "(").unwrap();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(self.output, ", ").unwrap();
                    }
                    self.print_operand(arg);
                }
                writeln!(self.output, ") -> bb{};", target.0).unwrap();
            }
            Terminator::Unreachable => {
                writeln!(self.output, "unreachable;").unwrap();
            }
        }
    }

    fn print_rvalue(&mut self, rvalue: &Rvalue<'db>) {
        match rvalue {
            Rvalue::Use(op) => self.print_operand(op),
            Rvalue::BinaryOp(op, lhs, rhs) => {
                self.print_operand(lhs);
                write!(self.output, " {} ", self.binop_str(*op)).unwrap();
                self.print_operand(rhs);
            }
            Rvalue::UnaryOp(op, operand) => {
                write!(self.output, "{}", self.unop_str(*op)).unwrap();
                self.print_operand(operand);
            }
            Rvalue::Constant(c) => self.print_constant(c),
            Rvalue::Aggregate(kind, operands) => {
                match kind {
                    AggregateKind::Struct(type_id) => {
                        write!(self.output, "{}{{", type_id.name(self.db)).unwrap();
                    }
                    AggregateKind::EnumVariant(type_id, variant_idx) => {
                        write!(
                            self.output,
                            "{}::variant_{}(",
                            type_id.name(self.db),
                            variant_idx
                        )
                        .unwrap();
                    }
                }
                for (i, op) in operands.iter().enumerate() {
                    if i > 0 {
                        write!(self.output, ", ").unwrap();
                    }
                    self.print_operand(op);
                }
                match kind {
                    AggregateKind::Struct(_) => write!(self.output, "}}").unwrap(),
                    AggregateKind::EnumVariant(_, _) => write!(self.output, ")").unwrap(),
                }
            }
            Rvalue::Array(operands) => {
                write!(self.output, "[").unwrap();
                for (i, op) in operands.iter().enumerate() {
                    if i > 0 {
                        write!(self.output, ", ").unwrap();
                    }
                    self.print_operand(op);
                }
                write!(self.output, "]").unwrap();
            }
        }
    }

    fn print_operand(&mut self, operand: &Operand<'db>) {
        match operand {
            Operand::Place(place) => self.print_place(place),
            Operand::Constant(c) => self.print_constant(c),
            Operand::FunctionRef(func_id) => {
                write!(self.output, "{}", func_id.text(self.db)).unwrap();
            }
        }
    }

    fn print_place(&mut self, place: &Place<'db>) {
        match place {
            Place::Local(local_id) => {
                write!(self.output, "_{}", local_id.0).unwrap();
            }
            Place::Field(base, field_idx) => {
                self.print_place(base);
                write!(self.output, ".{}", field_idx).unwrap();
            }
            Place::__Phantom(_) => unreachable!(),
        }
    }

    fn print_constant(&mut self, constant: &Constant<'db>) {
        match constant {
            Constant::Int(val) => {
                let ty = val.ty();
                match val {
                    scrap_shared::types::IntVal::Isize(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::IntVal::I8(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::IntVal::I16(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::IntVal::I32(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::IntVal::I64(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::IntVal::I128(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                }.unwrap();
            }
            Constant::Uint(val) => {
                let ty = val.ty();
                match val {
                    scrap_shared::types::UintVal::Usize(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::UintVal::U8(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::UintVal::U16(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::UintVal::U32(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::UintVal::U64(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::UintVal::U128(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                }.unwrap();
            }
            Constant::Float(val) => {
                let ty = val.ty();
                match val {
                    scrap_shared::types::FloatVal::F32(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                    scrap_shared::types::FloatVal::F64(v) => write!(self.output, "{}_{}", v, ty.name_str()),
                }.unwrap();
            }
            Constant::Void => write!(self.output, "void").unwrap(),
            Constant::Bool(b) => write!(self.output, "{}", b).unwrap(),
            Constant::String(s) => write!(self.output, "\"{}\"", s.text(self.db)).unwrap(),
        }
    }

    fn print_type(&mut self, ty: &Ty<'db>) {
        match ty {
            Ty::Void => write!(self.output, "void").unwrap(),
            Ty::Bool => write!(self.output, "bool").unwrap(),
            Ty::Int(k) => write!(self.output, "{}", k.name_str()).unwrap(),
            Ty::Uint(k) => write!(self.output, "{}", k.name_str()).unwrap(),
            Ty::Float(k) => write!(self.output, "{}", k.name_str()).unwrap(),
            Ty::Str => write!(self.output, "str").unwrap(),
            Ty::Adt(type_id) => write!(self.output, "{}", type_id.name(self.db)).unwrap(),
            Ty::Never => write!(self.output, "!").unwrap(),
        }
    }

    fn binop_str(&self, op: BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::BitXor => "^",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Ne => "!=",
            BinOp::Ge => ">=",
            BinOp::Gt => ">",
        }
    }

    fn unop_str(&self, op: UnOp) -> &'static str {
        match op {
            UnOp::Neg => "-",
            UnOp::Not => "!",
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            write!(self.output, "    ").unwrap();
        }
    }
}

/// Pretty-print a Can to a string
pub fn print_can<'db>(db: &'db dyn scrap_shared::Db, can: Can<'db>) -> String {
    let mut printer = IrPrinter::new(db);
    printer.print_can(can)
}
