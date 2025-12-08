//! Type inference mechanism for qcc.
use crate::ast::{Expr, FunctionAST, LiteralAST, Qast, QccCell, VarAST};
use crate::error::{QccError, QccErrorKind, Result};
use crate::types::Type;
use crate::utils::{mangle, mangle_module};
use std::borrow::{Borrow, BorrowMut};

/// A generic symbol table implementation.
struct SymbolTable<T> {
    table: std::collections::HashSet<T>,
}

impl<T> SymbolTable<T>
where
    T: std::cmp::Eq + std::hash::Hash,
{
    fn new() -> Self {
        Self {
            table: std::collections::HashSet::new(),
        }
    }

    fn push(&mut self, value: T) {
        self.table.insert(value);
    }

    fn extend(&mut self, values: Vec<T>) {
        self.table.extend(values);
    }

    fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.table.iter()
    }
}

impl<T> std::fmt::Display for SymbolTable<T>
where
    T: std::fmt::Display + std::cmp::Eq + std::hash::Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in self.iter() {
            write!(f, "{} ", entry)?;
        }
        writeln!(f, "")
    }
}

/// Sanity type checker for entire Qast.
pub(crate) fn checker(ast: &Qast) -> Result<()> {
    for module in ast {
        for function in &*module {
            for expr in &*function {
                check_expr(expr);
            }
        }
    }

    Ok(())
}

/// Checks type of an expression and returns it, an unknown type or a mismatch
/// results in an error being returned.
fn check_expr(expr: &QccCell<Expr>) -> Result<Type> {
    match *expr.as_ref().borrow() {
        Expr::Var(ref v) => {
            if !v.is_typed() {
                return Err(QccErrorKind::UnknownType)?;
            } else {
                return Ok(v.get_type());
            }
        }
        Expr::BinaryExpr(ref lhs, _, ref rhs) => {
            let lhs_type = check_expr(lhs)?;
            let rhs_type = check_expr(rhs)?;

            if lhs_type != rhs_type {
                return Err(QccErrorKind::TypeMismatch)?;
            }

            Ok(lhs_type)
        }
        Expr::Tensor(ref tensor) => {
            let mut tensor_type = Type::Bottom;
            let mut previous_type = tensor_type;

            for value in tensor {
                let _type = value.as_ref().borrow().get_type();
                if previous_type != _type {
                    return Err(QccErrorKind::TypeMismatch)?;
                }
            }

            Ok(tensor_type)
        }
        Expr::FnCall(ref f, ref args) => {
            for arg in args {
                check_expr(arg)?;
            }

            if *f.get_output_type() == Type::Bottom {
                return Err(QccErrorKind::UnknownType)?;
            }

            Ok(*f.get_output_type())
        }
        Expr::Let(ref var, ref val) => {
            if !var.is_typed() {
                return Err(QccErrorKind::UnknownType)?;
            }
            let val_type = check_expr(val)?;

            if var.get_type() != val_type {
                return Err(QccErrorKind::TypeMismatch)?;
            }

            Ok(val_type)
        }
        Expr::Conditional(ref conditional, ref truth_block, ref false_block) => {
            for expr in truth_block {
                check_expr(expr);
            }

            for expr in false_block {
                check_expr(expr);
            }

            let last_truth = truth_block.last();
            let last_false = false_block.last();

            if last_false.is_none() && last_truth.is_none() {
                return Ok(Type::Bottom);
            } else if last_false.is_none() ^ last_truth.is_none() {
                let last_expr;
                if last_false.is_none() {
                    last_expr = last_truth;
                } else {
                    last_expr = last_false;
                }

                return Ok(last_expr.unwrap().as_ref().borrow().get_type());
            } else {
                let truth_block_type = last_truth.unwrap().as_ref().borrow().get_type();
                let false_block_type = last_false.unwrap().as_ref().borrow().get_type();

                if truth_block_type != false_block_type {
                    return Err(QccErrorKind::TypeMismatch)?;
                }

                return Ok(truth_block_type);
            }
        }
        Expr::Literal(ref lit) => match *lit.as_ref().borrow() {
            LiteralAST::Lit_Digit(ref digit) => Ok(Type::F64),
            LiteralAST::Lit_Str(ref s) => Ok(Type::Bottom),
            LiteralAST::Lit_Qbit(_) => Ok(Type::Qbit),
        },
    }
}

/// Type inference method.
pub fn infer(ast: &mut Qast) -> Result<()> {
    let mut seen_errors = false;
    let mut function_table: SymbolTable<VarAST> = SymbolTable::new();

    // Merge all modules in one giant monolith module. Easier to do DCE and type
    // inference.
    let mut ast = ast.merge();

    for mut module in &mut ast {
        let module_name = module.get_name();
        // functions but only collect their names and return types.
        for function in &*module {
            function_table.push(VarAST::new_with_type(
                function.get_name().clone(),
                function.get_loc().clone(),
                function.get_output_type().clone(),
            ));
            // A copy of function prepended with its module name is also added.
            // If the function is used inside the module, then we check against
            // the value pushed above, and it is called from other module, then
            // we check against the value pushed below.
            function_table.push(VarAST::new_with_type(
                module_name.clone() + "$" + function.get_name(),
                function.get_loc().clone(),
                function.get_output_type().clone(),
            ));
        }

        for mut function in &mut *module {
            // parameter symbols
            let mut parameter_table: SymbolTable<VarAST> = SymbolTable::new();
            for param in function.iter_params() {
                parameter_table.push(param.clone());
            }

            // local variables
            let mut local_var_table: SymbolTable<VarAST> = SymbolTable::new();
            for instruction in &*function {
                // only add let-lhs and only if they are type checked
                match *instruction.as_ref().borrow() {
                    Expr::Let(ref def, _) => {
                        // don't type check lhs-rhs, otherwise along with a
                        // mismatch error, an unknown type error would also be
                        // raised if local st doesn't find typed lhs.
                        let checked: Result<Type> = Ok(def.get_type());
                        if checked.is_ok_and(|ty| ty != Type::Bottom) {
                            local_var_table.push(def.clone());
                        }
                    }
                    _ => {}
                }
            }

            // infer local var types
            for instruction in &mut *function {
                let instruction_type = infer_expr(instruction);

                if instruction_type.is_some_and(|ty| ty != Type::Bottom) {
                    match *instruction.as_ref().borrow() {
                        Expr::Let(ref var, _) => {
                            if var.is_typed() {
                                local_var_table.push(var.clone());
                            }
                        }
                        _ => {}
                    }
                }

                if instruction_type.is_none() || instruction_type == Some(Type::Bottom) {
                    // we couldn't infer all types for expression
                    // see if either symbol table contains any information
                    match infer_from_table(
                        instruction,
                        &parameter_table,
                        &local_var_table,
                        &function_table,
                    ) {
                        None => {
                            // This infers type for let expressions based on the
                            // symbol table but doesn't update the table
                            // entries. For e.g.,
                            // ```quale
                            //   let a: f64 = 42;
                            //   let b = a;  // this is inferred as f64 type,
                            //               // but symbol table
                            //               // doesn't contain it after
                            //               // inferring
                            //   let c = b;  // hence, this would fail to be
                            //               // inferred
                            // ```
                            // So we have to update symbol tables accordingly.
                            match *instruction.as_ref().borrow() {
                                Expr::Let(ref var, _) => {
                                    if var.is_typed() {
                                        local_var_table.push(var.clone());
                                    }
                                }
                                _ => {}
                            }
                        }
                        Some(untyped) => {
                            seen_errors = true;
                            match untyped {
                                Ok(expr) => {
                                    // unknown type of expression err
                                    let err: QccError = QccErrorKind::UnknownType.into();
                                    let expr = expr.as_ref().borrow();
                                    err.report(
                                        format!("for `{}` {}", expr, expr.get_location()).as_str(),
                                    );
                                }
                                Err(err) => {
                                    // err is returned
                                    let row = instruction.as_ref().borrow().get_location().row();
                                    err.report(&format!(
                                        "on\n\t{}\t{}",
                                        row,
                                        instruction.as_ref().borrow()
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            // type check between function return type and the last returned
            // expression
            let fn_return_type = *function.get_output_type();
            let fn_name = function.borrow().get_name().clone();

            let last_instruction = function.last_mut();
            if last_instruction.is_some() {
                let last = last_instruction.unwrap();

                // get last expression's type
                let last_instruction_type = infer_expr(last);

                if fn_return_type == Type::Bottom
                    && last_instruction_type.is_some()
                    && last_instruction_type != Some(Type::Bottom)
                {
                    function.set_output_type(last_instruction_type.unwrap());
                } else {
                    if last_instruction_type != Some(fn_return_type) {
                        seen_errors = true;
                        let err: QccError = QccErrorKind::TypeMismatch.into();
                        let last_expr = last.as_ref().borrow();
                        if last_instruction_type.is_none() {
                            err.report(&format!(
                                "between\n\t`{}` ({}) and `{}` ({}) {}",
                                last_expr,
                                Type::Bottom,
                                fn_name,
                                fn_return_type,
                                last.as_ref().borrow().get_location()
                            ));
                        } else {
                            err.report(&format!(
                                "between\n\t`{}` ({}) and `{}` ({}) {}",
                                last_expr,
                                last_instruction_type.unwrap(),
                                fn_name,
                                fn_return_type,
                                last.as_ref().borrow().get_location()
                            ));
                        }
                    }
                }
            }
        }
    }

    if seen_errors {
        return Err(QccErrorKind::TypeError)?;
    } else {
        Ok(())
    }
}

/// Infer type for expression returning the type. If inference isn't feasible
/// return None.
fn infer_expr(expr: &QccCell<Expr>) -> Option<Type> {
    match *expr.as_ref().borrow_mut() {
        Expr::Var(ref var) => {
            // return Some(*var.get_type());
            if var.get_type() == Type::Bottom {
                return None;
            } else {
                return Some(var.get_type());
            }
        }

        Expr::BinaryExpr(ref lhs, ref op, ref rhs) => {
            let lhs_type = infer_expr(&lhs)?;
            let rhs_type = infer_expr(&rhs)?;

            // A qubit can be operated by a float. So a binary expression like:
            //   2 * 0q(1, 0)
            // where a qubit is multiplied by 2, is valid. The resulting type
            // will be of a qubit.
            if (lhs_type == Type::F64 && rhs_type == Type::Qbit)
                || (lhs_type == Type::Qbit && rhs_type == Type::F64)
            {
                return Some(Type::bigtype(lhs_type, rhs_type));
            }

            if lhs_type != rhs_type {
                return None;
            }
            return Some(lhs_type);
        }

        Expr::Tensor(ref tensor) => {
            for value in tensor {
                let val_type = infer_expr(value);
                if val_type.is_none() {
                    return None;
                }
            }

            let mut tensor_type = Type::Bottom;
            if tensor.len() > 0 {
                tensor_type = tensor[0].as_ref().borrow().get_type();
            }

            for value in tensor {
                let val_type = infer_expr(value)?;
                if val_type != tensor_type {
                    return None;
                }
            }

            return Some(tensor_type);
        }

        Expr::FnCall(ref mut f, ref args) => {
            if *f.get_output_type() == Type::Bottom && args.len() != 0 {
                // we can only infer input types by matching against args
                for arg in args {
                    let arg_type = infer_expr(&arg)?;
                    f.insert_input_type(arg_type);
                }
                // TODO: we cannot infer function return type and it may return
                // a Bottom type.
                return Some(*f.get_output_type());
            } else {
                return Some(*f.get_output_type());
            }
        }

        Expr::Let(ref mut var, ref val) => {
            // val is an expression and it must have the same type as var
            if var.get_type() == Type::Bottom {
                // we need to type check from expression first
                let rhs_type = infer_expr(&val)?;
                var.set_type(rhs_type);
                return Some(rhs_type);
            } else {
                let lhs_type = var.get_type();
                let rhs_type = infer_expr(&val)?;
                if lhs_type != rhs_type {
                    return None;
                }
                return Some(lhs_type);
            }
        }

        Expr::Conditional(ref conditional, ref truth_block, ref false_block) => {
            let mut truth_block_type = Some(Type::Bottom);
            for expr in truth_block {
                truth_block_type = infer_expr(expr);
            }

            let mut false_block_type = Some(Type::Bottom);
            for expr in false_block {
                false_block_type = infer_expr(expr);
            }

            // Ensure both last expressions in truth_block and false_block are
            // of same type.
            if truth_block_type == false_block_type {
                return truth_block_type;
            } else {
                return Some(Type::Bottom);
            }
        }

        Expr::Literal(ref lit) => {
            return match *lit.as_ref().borrow() {
                LiteralAST::Lit_Digit(_) => Some(Type::F64),
                LiteralAST::Lit_Str(_) => Some(Type::Bottom),
                LiteralAST::Lit_Qbit(_) => Some(Type::Qbit),
            };
        }
    }
    Some(Type::Bottom)
}

/// Given an expression gather all variable references which have already been
/// typed and return them.
fn gather_already_typed(expr: &QccCell<Expr>) -> Vec<VarAST> {
    let mut symbol_table = vec![];
    match *expr.as_ref().borrow() {
        Expr::Var(ref var) => {
            if var.is_typed() {
                symbol_table.push(var.clone());
            }
        }
        Expr::BinaryExpr(ref lhs, ref op, ref rhs) => {
            let lhs_symbols = gather_already_typed(&lhs);
            symbol_table.extend(lhs_symbols);

            let rhs_symbols = gather_already_typed(&rhs);
            symbol_table.extend(rhs_symbols);
        }
        Expr::FnCall(ref f, ref args) => {
            for arg in args {
                let arg_table = gather_already_typed(&arg);
                symbol_table.extend(arg_table);
            }
        }
        Expr::Let(ref var, ref val) => {
            if var.is_typed() {
                symbol_table.push(var.clone());
            }
            let val_table = gather_already_typed(&val);
            symbol_table.extend(val_table);
        }
        _ => {}
    }
    symbol_table
}

/// Infer types for each part of expression from symbol tables. If some
/// expression cannot be typed, because no information was found in symbol
/// tables, then return that expression. Otherwise if complete expression is
/// typed then return None. If any mismatch is seen, return appropriate error.
fn infer_from_table(
    expr: &QccCell<Expr>,
    param_st: &SymbolTable<VarAST>,
    local_st: &SymbolTable<VarAST>,
    function_st: &SymbolTable<VarAST>,
) -> Option<Result<QccCell<Expr>>> {
    match *expr.as_ref().borrow_mut() {
        Expr::Var(ref mut var) => {
            let mut param_type = Type::Bottom;
            let mut local_type = Type::Bottom;
            for param in param_st.iter() {
                if param.name() == var.name() && param.is_typed()
                /*trivial*/
                {
                    param_type = param.get_type();
                }
            }
            for local in local_st.iter() {
                if local.name() == var.name() && local.is_typed() {
                    local_type = local.get_type();
                }
            }
            if param_type == local_type && param_type == Type::Bottom {
                // couldn't find any type information
                // return Some(var);
                return Some(Ok(Expr::Var(VarAST::new_with_type(
                    var.name().clone(),
                    var.location().clone(),
                    var.get_type(),
                ))
                .into()));
            }
            if param_type != Type::Bottom {
                var.set_type(param_type);
            } else if local_type != Type::Bottom {
                var.set_type(local_type);
            }
            None
        }

        Expr::BinaryExpr(ref lhs, ref op, ref rhs) => {
            let lhs_info = infer_from_table(lhs, param_st, local_st, function_st);
            if lhs_info.is_some() {
                return lhs_info;
            }
            let rhs_info = infer_from_table(rhs, param_st, local_st, function_st);
            if rhs_info.is_some() {
                return rhs_info;
            }
            None
        }

        Expr::Tensor(ref tensor) => {
            for value in tensor {
                let val_info = infer_from_table(value, param_st, local_st, function_st);
                if val_info.is_some() {
                    return val_info;
                }
            }
            None
        }

        Expr::FnCall(ref mut f, ref args) => {
            for arg in args {
                let info = infer_from_table(arg, param_st, local_st, function_st);
                if info.is_some() {
                    return info;
                }
            }

            for func in function_st.iter() {
                if func.name() == f.get_name() && func.is_typed() {
                    f.set_output_type(func.get_type());
                    return None;
                }
            }

            // unable to infer return type for function, returning it
            Some(Ok(Expr::FnCall(
                FunctionAST::new(
                    f.get_name().clone(),
                    f.get_loc().clone(),
                    Default::default(),
                    Default::default(),
                    *f.get_output_type(),
                    Default::default(),
                    Default::default(),
                ),
                vec![],
            )
            .into()))
        }
        Expr::Let(ref mut var, ref val) => {
            let rhs_info = infer_from_table(val, param_st, local_st, function_st);

            if rhs_info.is_some() {
                return rhs_info;
            }

            let var_type = var.get_type();
            let val_type = val.as_ref().borrow().get_type();

            if !var.is_typed() {
                var.set_type(val.as_ref().borrow().get_type());
                None
            } else if (var_type == Type::Qbit || var_type == Type::Bit)
                && (val_type == Type::Qbit || val_type == Type::Bit)
                && (var_type != val_type)
            {
                // If one of lhs or rhs is qbit while the other is bit, then we
                // will put a measure operator before it is assigned during
                // codegen.
                //
                //  let x: bit = y;     # y := qbit
                //
                // This holds according to our subtyping rules. Codegen will
                // lower this to:
                //
                //  let x0: bit = measure(y);
                //  let x: bit  = x0;
                //
                // Similarily,
                //
                //  let x: qbit = y;    # y := bit
                //
                // This is also fine. When codegen lowers this code, it
                // automatically puts required stub to create a logical qubit.
                None
            } else if var_type == val_type {
                None
            } else if var_type != val_type {
                // if one is qbit and other is bit, pass
                Some(Err(QccErrorKind::TypeMismatch.into()))
            } else {
                Some(Ok(Expr::Var(VarAST::new(
                    var.name().clone(),
                    var.location().clone(),
                ))
                .into()))
            }
        }
        Expr::Conditional(ref mut conditional, ref mut truth_block, ref mut false_block) => {
            for expr in truth_block {
                let info = infer_from_table(expr, param_st, local_st, function_st);

                if info.is_some() {
                    return info;
                }
            }

            for expr in false_block {
                let info = infer_from_table(expr, param_st, local_st, function_st);

                if info.is_some() {
                    return info;
                }
            }

            None
        }
        Expr::Literal(ref mut l) => {
            // A literal is usually typed but if it isn't then it should follow
            // based on what the context says.
            match *l.as_ref().borrow() {
                LiteralAST::Lit_Qbit(ref q) => None,
                // digits are trivially typed
                LiteralAST::Lit_Digit(ref d) => None,
                LiteralAST::Lit_Str(ref s) => todo!("{:?} perhaps a string", s),
            }
        }
    }
}

/// Given an expression return a vector of all variable references irrespective
/// of whether they are typed or not.
fn gather_all_vars(expr: &mut Box<Expr>) -> Vec<&mut VarAST> {
    vec![]
}
