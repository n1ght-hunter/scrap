//! Unification algorithm for type inference.

use scrap_span::Span;

use crate::{
    constraints::Constraint,
    context::TypeContext,
    types::{InferTy, TyVid},
};

impl<'db> TypeContext<'db> {
    /// Unify two types, binding type variables as needed.
    /// Returns true if unification succeeded.
    pub fn unify(&mut self, t1: &InferTy<'db>, t2: &InferTy<'db>, span: Span<'db>) -> bool {
        let t1 = self.resolve(t1);
        let t2 = self.resolve(t2);

        match (&t1, &t2) {
            // Same type variable - trivially equal
            (InferTy::Var(v1), InferTy::Var(v2)) if v1 == v2 => true,

            // Bind type variable to concrete type
            (InferTy::Var(vid), other) | (other, InferTy::Var(vid)) => {
                // Occurs check: prevent infinite types like T = List<T>
                if self.occurs_check(*vid, other) {
                    self.emit_infinite_type(
                        &format!("?{}", vid.0),
                        &self.ty_to_string(other),
                        span,
                    );
                    return false;
                }
                self.bind(*vid, other.clone());
                true
            }

            // Primitive types must match exactly
            (InferTy::Void, InferTy::Void) => true,
            (InferTy::Bool, InferTy::Bool) => true,
            (InferTy::Str, InferTy::Str) => true,

            // Sized ints must match exactly
            (InferTy::Int(k1), InferTy::Int(k2)) if k1 == k2 => true,
            (InferTy::Uint(k1), InferTy::Uint(k2)) if k1 == k2 => true,

            // Sized floats must match exactly
            (InferTy::Float(k1), InferTy::Float(k2)) if k1 == k2 => true,

            // Never type unifies with anything (it's a bottom type)
            (InferTy::Never, _) | (_, InferTy::Never) => true,

            // ADT types must have same name
            (InferTy::Adt(n1), InferTy::Adt(n2)) => {
                if n1 == n2 {
                    true
                } else {
                    self.emit_type_mismatch(&self.ty_to_string(&t1), &self.ty_to_string(&t2), span);
                    false
                }
            }

            // Generic params must match
            (InferTy::Param(p1), InferTy::Param(p2)) => {
                if p1 == p2 {
                    true
                } else {
                    self.emit_type_mismatch(&self.ty_to_string(&t1), &self.ty_to_string(&t2), span);
                    false
                }
            }

            // Applied types: unify name and all arguments
            (InferTy::App(n1, args1), InferTy::App(n2, args2)) => {
                if n1 != n2 {
                    self.emit_type_mismatch(&self.ty_to_string(&t1), &self.ty_to_string(&t2), span);
                    return false;
                }
                if args1.len() != args2.len() {
                    self.emit_type_arity_mismatch(args1.len(), args2.len(), span);
                    return false;
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    if !self.unify(a1, a2, span) {
                        return false;
                    }
                }
                true
            }

            // Function types
            (InferTy::Fn(params1, ret1), InferTy::Fn(params2, ret2)) => {
                if params1.len() != params2.len() {
                    self.emit_type_mismatch(&self.ty_to_string(&t1), &self.ty_to_string(&t2), span);
                    return false;
                }
                // Unify parameter types
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    if !self.unify(p1, p2, span) {
                        return false;
                    }
                }
                // Unify return types
                self.unify(ret1, ret2, span)
            }

            // Reference types: inner must unify, mutability must match
            (InferTy::Ref(inner1, m1), InferTy::Ref(inner2, m2)) => {
                if m1 != m2 {
                    self.emit_type_mismatch(&self.ty_to_string(&t1), &self.ty_to_string(&t2), span);
                    return false;
                }
                self.unify(inner1, inner2, span)
            }

            // Pointer types: inner must unify
            (InferTy::Ptr(inner1), InferTy::Ptr(inner2)) => self.unify(inner1, inner2, span),

            // Tuple types
            (InferTy::Tuple(elems1), InferTy::Tuple(elems2)) => {
                if elems1.len() != elems2.len() {
                    self.emit_type_mismatch(&self.ty_to_string(&t1), &self.ty_to_string(&t2), span);
                    return false;
                }
                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    if !self.unify(e1, e2, span) {
                        return false;
                    }
                }
                true
            }

            // Error type unifies with anything (for error recovery)
            (InferTy::Error, _) | (_, InferTy::Error) => true,

            // Mismatch - types are incompatible
            _ => {
                self.emit_type_mismatch(&self.ty_to_string(&t1), &self.ty_to_string(&t2), span);
                false
            }
        }
    }

    /// Check if a type variable occurs in a type (for infinite type detection).
    /// Returns true if the type variable appears in the type.
    fn occurs_check(&self, vid: TyVid, ty: &InferTy<'db>) -> bool {
        match ty {
            InferTy::Var(v) => {
                if *v == vid {
                    return true;
                }
                // Follow the chain if this variable is bound
                if let Some(resolved) = self.probe(*v) {
                    self.occurs_check(vid, resolved)
                } else {
                    false
                }
            }
            InferTy::App(_, args) => args.iter().any(|a| self.occurs_check(vid, a)),
            InferTy::Fn(params, ret) => {
                params.iter().any(|p| self.occurs_check(vid, p)) || self.occurs_check(vid, ret)
            }
            InferTy::Tuple(elems) => elems.iter().any(|e| self.occurs_check(vid, e)),
            InferTy::Ref(inner, _) => self.occurs_check(vid, inner),
            InferTy::Ptr(inner) => self.occurs_check(vid, inner),
            // Primitive types and ADTs don't contain type variables
            InferTy::Void
            | InferTy::Bool
            | InferTy::Int(_)
            | InferTy::Uint(_)
            | InferTy::Float(_)
            | InferTy::Str
            | InferTy::Never
            | InferTy::Adt(_)
            | InferTy::Param(_)
            | InferTy::Error => false,
        }
    }

    /// Solve all collected constraints.
    /// Returns true if all constraints were satisfied.
    pub fn solve_constraints(&mut self) -> bool {
        let constraints = self.take_constraints();
        let mut success = true;

        for constraint in constraints {
            match constraint {
                Constraint::Eq(t1, t2, origin) => {
                    if !self.unify(&t1, &t2, origin.span) {
                        success = false;
                    }
                }
            }
        }

        success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrap_shared::types::IntTy;

    #[scrap_macros::salsa_test]
    fn test_unify_same_primitives(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");
        let span = Span::new(db, 0, 0);

        assert!(ctx.unify(&InferTy::Int(IntTy::I32), &InferTy::Int(IntTy::I32), span));
        assert!(ctx.unify(&InferTy::Bool, &InferTy::Bool, span));
        assert!(ctx.unify(&InferTy::Str, &InferTy::Str, span));
    }

    #[scrap_macros::salsa_test]
    fn test_unify_different_primitives(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");
        let span = Span::new(db, 0, 0);

        assert!(!ctx.unify(&InferTy::Int(IntTy::I32), &InferTy::Bool, span));
        assert!(db.dcx().has_errors());
    }

    #[scrap_macros::salsa_test]
    fn test_unify_type_var_with_concrete(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");
        let span = Span::new(db, 0, 0);

        let var = ctx.fresh_ty_var();
        assert!(ctx.unify(&var, &InferTy::Int(IntTy::I32), span));

        // After unification, resolving should give Int
        let resolved = ctx.resolve(&var);
        assert_eq!(resolved, InferTy::Int(IntTy::I32));
    }

    #[scrap_macros::salsa_test]
    fn test_unify_two_type_vars(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");
        let span = Span::new(db, 0, 0);

        let var1 = ctx.fresh_ty_var();
        let var2 = ctx.fresh_ty_var();

        // Unify two type variables
        assert!(ctx.unify(&var1, &var2, span));

        // Then unify one with a concrete type
        assert!(ctx.unify(&var1, &InferTy::Int(IntTy::I32), span));

        // Both should resolve to Int
        assert_eq!(ctx.resolve(&var1), InferTy::Int(IntTy::I32));
        assert_eq!(ctx.resolve(&var2), InferTy::Int(IntTy::I32));
    }

    #[scrap_macros::salsa_test]
    fn test_unify_never_with_anything(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");
        let span = Span::new(db, 0, 0);

        assert!(ctx.unify(&InferTy::Never, &InferTy::Int(IntTy::I32), span));
        assert!(ctx.unify(&InferTy::Bool, &InferTy::Never, span));
    }

    #[scrap_macros::salsa_test]
    fn test_unify_tuples(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");
        let span = Span::new(db, 0, 0);

        let tuple1 = InferTy::Tuple(vec![InferTy::Int(IntTy::I32), InferTy::Bool]);
        let tuple2 = InferTy::Tuple(vec![InferTy::Int(IntTy::I32), InferTy::Bool]);
        assert!(ctx.unify(&tuple1, &tuple2, span));

        // Different lengths should fail
        let tuple3 = InferTy::Tuple(vec![InferTy::Int(IntTy::I32)]);
        assert!(!ctx.unify(&tuple1, &tuple3, span));
    }

    #[scrap_macros::salsa_test]
    fn test_occurs_check(db: &dyn scrap_shared::Db) {
        let mut ctx = TypeContext::new(db, "", "test.sc");
        let span = Span::new(db, 0, 0);

        let var = ctx.fresh_ty_var();
        // Try to unify ?0 with Tuple(?0) - should fail with infinite type error
        let infinite = InferTy::Tuple(vec![var.clone()]);
        assert!(!ctx.unify(&var, &infinite, span));
        assert!(db.dcx().has_errors());
    }
}
