
/////////////////////////// OP ////////////////////////////////////////////////////////////////////

use std::{fmt, io};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::io::{Write};
use std::str::FromStr;
use crate::interpreter::{Args, Binding};
use crate::numeric::Numeric;
use crate::parsed::Parsed;
use crate::parsing::{ parse_to_quotation};
use crate::stack_error::StackError;
use crate::types::{Params, Constraint, heterogeneous_binary, homogenous_binary, nullary, Signature, Type, unary};


#[derive(Clone, PartialEq)]
/// enumerator of operations, i.e. specific functions.
pub enum Op {
    Void,
    IOPrint,
    IORead,
    ParseInt,
    ParseFloat,
    ParseWords,
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    LT,
    GT,
    EQ,
    And,
    Or,
    Not,
    Head,
    Tail,
    Empty,
    Length,
    Cons,
    Append,
    Each,
    Map,
    Foldl,
    If,
    Loop,
    Times,
    Exec,
    Assign,
    AssignFunc,
    AsSymbol,
    EvalSymbol,
    Dup,
    Swap,
    Pop,
    Mod,
    Error
}


impl Op {

    pub fn exec_nullary(&self, mods: Args, _bindings: &mut HashMap<String, Binding>) -> Parsed {
        match self {
            Op::IORead => Self::exec_ioread(),
            Op::Void => Self::exec_void(),
            Op::AsSymbol => Self::exec_as_symbol(mods),
            Op::Loop => Self::exec_loop(mods),
            Op::Error => Self::exec_err(mods),
            _ => panic!("bug:  use of wrong exec_* function for function {}", self)
        }
    }

    fn exec_loop(c: Args) -> Parsed {
        match c {
            Args::Binary(mut quot1, mut quot2) => {
                quot1 = quot1.coerce(&Type::Quotation);
                quot2 = quot2.coerce(&Type::Quotation);
                let ret = format!(
                    " {:?} exec not if {{ {:?} exec loop {:?} {:?} }} {{ }} ",
                    quot1,  quot2, quot1, quot2);
                return parse_to_quotation(ret)
            }
            _ => panic!("invalid closure count sent to each function"),
        }
    }

    fn exec_as_symbol(c: Args) -> Parsed {
        if let Args::Unary(Parsed::Symbol(s)) = c {
            return Parsed::Symbol(s.clone());
        }
        panic!("bug: function ' (eval as symbol) fed non symbol as modifier. Check constraints.")
    }

    fn exec_eval(arg: Parsed, bindings: &mut HashMap<String, Binding>) -> Parsed {
        if let Parsed::Symbol(s) = arg {
            return if let Some(binding) = bindings.get(s.as_str()) {
                binding.value.clone()
            } else {
                Parsed::Symbol(s.clone())
            }
        }
        panic!("bug: function ' (eval as symbol) fed non symbol as modifier. Check constraints.")
    }


    pub fn exec_unary(&self, arg: Parsed, c: Args, bindings: &mut  HashMap<String, Binding>) -> Parsed {
        match self {
            Op::IOPrint => Self::exec_print(arg),
            Op::ParseInt => Self::exec_parse_int(arg),
            Op::ParseFloat => Self::exec_parse_float(arg),
            Op::ParseWords => Self::exec_words(arg),
            Op::Empty => Self::exec_empty(arg),
            Op::Length => Self::exec_length(arg),
            Op::Head => Self::exec_head(arg),
            Op::Tail => Self::exec_tail(arg),
            Op::Not => Self::exec_not(arg),
            Op::Pop => Self::exec_pop(arg),
            Op::Dup => Self::exec_dup(arg),
            Op::Exec => Self::exec_exec(arg),
            Op::If => Self::exec_if(arg, c),
            Op::Times => Self::exec_times(arg, c),
            Op::Map => Self::exec_map(arg, c),
            Op::Each => Self::exec_each(arg, c),
            Op::EvalSymbol => Self::exec_eval(arg, bindings),
            _ => panic!("bug:  use of wrong exec_* function for function {}", self)
        }
    }

    pub fn exec_binary(&self, lhs: &Parsed, rhs: &Parsed, c: Args, bindings: &mut HashMap<String, Binding>) -> Parsed {
        match self {
            Op::Mod => Self::exec_mod(lhs, rhs),
            Op::Add => Self::exec_add(lhs, rhs),
            Op::Sub => Self::exec_sub(lhs, rhs),
            Op::Mul => Self::exec_mul(lhs, rhs),
            Op::Div => Self::exec_div(lhs, rhs),
            Op::IntDiv => Self::exec_intdiv(lhs, rhs),
            Op::GT => Self::exec_gt(lhs, rhs),
            Op::LT => Self::exec_lt(lhs, rhs),
            Op::EQ => Self::exec_eq(lhs, rhs),
            Op::And => Self::exec_and(lhs, rhs),
            Op::Or => Self::exec_or(lhs, rhs),
            Op::Append => Self::exec_append(lhs, rhs),
            Op::Cons => Self::exec_cons(lhs, rhs),
            Op::Swap => Self::exec_swap(lhs, rhs),
            Op::Foldl => Self::exec_foldl(lhs, rhs, c),
            Op::Assign => Self::exec_assign(lhs, rhs, c, bindings, false),
            Op::AssignFunc => Self::exec_assign(lhs, rhs, c, bindings, true),
            _ => panic!("bug:  use of wrong exec_* function for function {}, or function not implemented.", self)
        }
    }


    fn exec_assign(lhs: &Parsed, rhs: &Parsed, _c: Args, bindings: &mut HashMap<String, Binding>, func: bool) -> Parsed {
        if func && !Constraint::Executable.is_satisfied_by(&rhs.get_type()) {
            panic!("bug: non executable value attempted bound to function. Check constraint system.")
        }
        match lhs {
            Parsed::Symbol(s) => {
                if let Some (val) = bindings.get(s.as_str()) {
                    if val.constant {
                        return Parsed::Error(StackError::Undefined);
                    }
                }
                let binding = Binding{
                    function: func,
                    constant: false,
                    value: rhs.clone(),
                };
                bindings.insert(s.clone(), binding);
                Parsed::Void

            },
            _ => panic!("bug: exec_assign given non-symbol for bind operation"),
        }
    }


    ////////////////////////////////////////////////////////////////////////////////////////////////
    ////                               BUILT IN FUNCTION DEFINITIONS                            ////
    ////////////////////////////////////////////////////////////////////////////////////////////////



    //// VOID FUNCTION DEFINITION ////

    fn exec_void() -> Parsed {
        Parsed::Void
    }



    //// IO FUNCTION DEFINITIONS ////

    pub fn exec_ioread() -> Parsed {
        print!("input : ");
        io::stdout().flush().unwrap();
        let mut string = String::new();
        if let Ok(_) = io::stdin().read_line(&mut string) {
            string.pop();
            Parsed::String(string)
        } else {
            panic!("bug: failed to read from stdin.")
        }
    }

    pub fn exec_print(arg: Parsed) -> Parsed {
        println!("output: {}", arg);
        Parsed::Void
    }



    //// PARSING FUNCTION DEFINITIONS ////

    pub fn exec_parse_int(arg: Parsed) -> Parsed {
        match arg {
            Parsed::String(s) => {
                return if let Ok(i) = s.parse::<i128>() {
                    Parsed::Num(Numeric::Integer(i))
                } else {
                    Parsed::Error(StackError::Overflow)
                }
            },
            _ => panic!("bug: argument type not implemented for parseInteger")
        }
    }

    pub fn exec_parse_float(arg: Parsed) -> Parsed {
        match arg {
            Parsed::String(s) => {
                return if let Ok(f) = s.parse::<f64>() {
                    Parsed::Num(Numeric::Float(f))
                } else {
                    Parsed::Error(StackError::Overflow)
                }
            },
            _ => panic!("bug: argument type not implemented for parseFloat")
        }
    }

    pub fn exec_words(arg: Parsed) -> Parsed {
        match arg {
            Parsed::String(s) => {
                Parsed::List(
                    s.split_whitespace()
                        .map(|s| Parsed::String(s.to_string()))
                        .collect::<Vec<Parsed>>(),
                )
            }
            _ => panic!("bug: argument type not implemented for words"),
        }
    }



    //// ARITHMETIC, ORDERING, EQ, BOOLEAN FUNCTION DEFINITIONS ////
    pub fn exec_mod(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        if let (Parsed::Num(Numeric::Integer(i)), Parsed::Num(Numeric::Integer(i2))) = (lhs, rhs) {
            return Parsed::Num(Numeric::Integer(i % i2));
        }
        panic!("bug: non integer passed to modulo operation");
    }

    pub fn exec_add(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        lhs + rhs
    }

    pub fn exec_sub(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        lhs - rhs
    }

    pub fn exec_mul(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        lhs * rhs
    }

    pub fn exec_div(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        &lhs.coerce(&Type::Float) / &rhs.coerce(&Type::Float)
    }

    pub fn exec_intdiv(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        (&lhs.coerce(&Type::Integer) / &rhs.coerce(&Type::Integer)).coerce(&Type::Integer)
    }

    pub fn exec_gt(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        Parsed::Bool(lhs > rhs)
    }

    pub fn exec_lt(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        Parsed::Bool(lhs < rhs)
    }

    pub fn exec_eq(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        Parsed::Bool(lhs == rhs)
    }

    pub fn exec_and(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        lhs & rhs
    }

    pub fn exec_or(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        lhs | rhs
    }

    pub fn exec_not(arg: Parsed) -> Parsed {
        -arg
    }

    //// CONTAINER FUNCTION DEFINITIONS ////

    pub fn exec_empty(arg: Parsed) -> Parsed {
        Parsed::Bool(arg.size() == Parsed::Num(Numeric::Integer(0)))
    }

    pub fn exec_length(arg: Parsed) -> Parsed {
        arg.size()
    }

    pub fn exec_head(arg: Parsed) -> Parsed {
        match arg {
            Parsed::List(v) => {
                v.get(0).unwrap_or_else(||&Parsed::Error(StackError::HeadEmpty)).clone()
            }
            _ => panic!("head not supported for {}", arg),
        }
    }

    pub fn exec_tail(arg: Parsed) -> Parsed {
        match arg {
            Parsed::List(v) => {
                if !v.is_empty() {
                    Parsed::List(v[1..].to_vec())
                } else {
                    panic!("exec_tail: TODO")
                }
            }
            _ => panic!("tail not support"),
        }
    }

    pub fn exec_append(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        lhs + rhs
    }

    pub fn exec_cons(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        &Parsed::List(vec![lhs.clone()]) + rhs
    }



   //// STACK FUNCTION DEFINITIONS ////

    /// Consumes a Parsed value and returns Void.
    pub fn exec_pop(_: Parsed) -> Parsed {
        Parsed::Void
    }

    /// Consumes a Parsed value and returns a Quotation that places
    /// two instances of the consumed value back onto the stack.
    pub fn exec_dup(arg: Parsed) -> Parsed {
        Parsed::Quotation(VecDeque::from(vec![arg.clone(), arg.clone()]))
    }

    /// Takes two Parsed values and returns a Quotation that puts them back onto
    /// the sack in reverse order.
    pub fn exec_swap(lhs: &Parsed, rhs: &Parsed) -> Parsed {
        Parsed::Quotation(VecDeque::from(vec![rhs.clone(), lhs.clone()]))
    }



    //// CONTROL FUNCTION DEFINITIONS ////

    pub fn exec_exec(arg: Parsed) -> Parsed {
        match arg.coerce(&Type::Quotation) {
            Parsed::Quotation(q) => Parsed::Quotation(q.clone()),
            // TODO: Define
            _ => Parsed::Error(StackError::Undefined),
        }
    }

    pub fn exec_if(arg: Parsed, c: Args) -> Parsed {
        match c {
            Args::Binary(then_quotation, else_quotation) => {
                if arg == Parsed::Bool(true) {
                    then_quotation.coerce(&Type::Quotation)
                } else {
                    else_quotation.coerce(&Type::Quotation)
                }
            }
            _ => panic!("Invalid Closure count sent to if function"),
        }
    }

    pub fn exec_times(arg: Parsed, c: Args) -> Parsed {
        match c {
            Args::Unary(quotation) => match arg {
                Parsed::Num(Numeric::Integer(i)) => {
                    let quotation = quotation.coerce(&Type::Quotation);
                    return parse_to_quotation(
                            format!(" {} 0 > if {{ {:?} exec {} 1 - times {:?} }} {{ }} ",
                                    i, quotation,  i, quotation)
                        );
                }
                //TODO: Stack error definition
                _ => Parsed::Error(StackError::Undefined),
            },
            _ => panic!("Invalid Closure count sent to times function"),
        }
    }



    //// HIGHER ORDER FUNCTION DEFINITIONS ////

    /// Defines the map function.
    ///
    ///
    /// {
    pub fn exec_map(arg: Parsed, c: Args) -> Parsed {
        match c {
            Args::Unary(mut quotation) => {
                quotation = quotation.coerce(&Type::Quotation);
                return parse_to_quotation(
                    format!(
                        "{:?} length 0 > if {{  {:?} head {:?} exec {:?} tail map {:?} cons }} {{ [ ] }} ",
                        arg, arg, quotation, arg, quotation
                    )
                );
            }
            _ => panic!("invalid closure count sent to map function"),
        }
    }

    pub fn exec_each(arg: Parsed, c: Args) -> Parsed {
        match c {
            Args::Unary(mut quotation) => {
                quotation = quotation.coerce(&Type::Quotation);
                return parse_to_quotation(
                    format!(
                        " {:?} length 0 > if {{ {:?} head {:?} exec {:?} tail each {:?} }} {{ }} ",
                        arg,  arg, quotation, arg, quotation)
                );
            }
            _ => panic!("invalid closure count sent to each function"),
        }
    }


    pub fn exec_foldl(lhs: &Parsed, rhs: &Parsed, c: Args) -> Parsed {
        match c {
            Args::Unary(quotation) => {
                let quotation = quotation.coerce(&Type::Quotation);
                return parse_to_quotation(
                    format!(
                        " {:?} length 0 > if {{ {:?} {:?} head {:?} exec {:?} tail swap foldl {:?} }} {{ {:?} }}  ",
                        lhs,  rhs, lhs, quotation, lhs, quotation, rhs)
                );
            },
            _ => panic!("invalid closure count sent to foldl function")
        }
    }

    /// Retrieves the Signature of a function, containing details about
    /// argument and return constraints.
    pub fn get_signature(&self) -> Signature {
        match self {
            Op::Void => Self::get_void_sig(),
            Op::IOPrint => Self::get_print_sig(),
            Op::IORead => Self::get_read_sig(),
            Op::ParseInt => Self::get_parse_int_sig(),
            Op::ParseFloat => Self::get_parse_float_sig(),
            Op::ParseWords => Self::get_words_sig(),
            Op::Mod => Self::get_mod_sig(),
            Op::Add | Op::Sub | Op::Mul | Op::Div => Self::get_arithmetic_sig(),
            Op::IntDiv => Self::get_arithmetic_sig(),
            Op::LT | Op::GT => Self::get_ord_sig(),
            Op::EQ => Self::get_eq_sig(),
            Op::And | Op::Or => Self::get_and_or_sig(),
            Op::Not => Self::get_not_sig(),
            Op::Head => Self::get_head_sig(),
            Op::Tail => Self::get_tail_sig(),
            Op::Empty => Self::get_empty_sig(),
            Op::Length => Self::get_length_sig(),
            Op::Cons => Self::get_cons_sig(),
            Op::Append => Self::get_append_sig(),
            Op::Each => Self::get_each_sig(),
            Op::Map => Self::get_map_sig(),
            Op::Foldl => Self::get_foldl_sig(),
            Op::If => Self::get_if_sig(),
            Op::Loop => Self::get_loop_sig(),
            Op::Times => Self::get_times_sig(),
            Op::Exec => Self::get_exec_sig(),
            Op::Assign => Self::get_assign_sig(),
            Op::AssignFunc => Self::get_assign_func_sig(),
            Op::AsSymbol => Self::get_as_symbol_sig(),
            Op::EvalSymbol => Self::get_eval_symbol_sig(),
            Op::Dup => Self::get_dup_sig(),
            Op::Swap => Self::get_swap_sig(),
            Op::Pop => Self::get_pop_sig(),
            Op::Error => Self::get_err_sig(),
        }
    }

    fn get_err_sig() -> Signature {
        let mut sig = nullary(Constraint::Error);
        sig.modifiers = Params::Unary(Constraint::String);
        sig
    }

    fn exec_err(mods: Args) -> Parsed {
        if let Args::Unary(Parsed::String(err)) = mods {
            return Parsed::Error(StackError::UserDefined(err.clone()))
        }
        panic!("bug: invalid modifier sent to exec_err.")
    }

    //////////////////////////////// SIGNATURE DEFINITIONS //////////////////////////////////////

    //// VOID /////

    fn get_void_sig() -> Signature {
        nullary(Constraint::Void)
    }

    //// IO /////

    fn get_print_sig() -> Signature {
        unary(Constraint::Display, Constraint::Void)
    }

    fn get_read_sig() -> Signature {
        nullary(Constraint::String)
    }

    //// PARSE ////

    fn get_parse_int_sig() -> Signature {
        unary(Constraint::String, Constraint::Integer)
    }

    fn get_parse_float_sig() -> Signature {
        unary(Constraint::String, Constraint::Float)
    }

    fn get_words_sig() -> Signature {
        unary(Constraint::String, Constraint::List)
    }

    //// ARITHMETIC, ORDERING, EQ, BOOLEAN ////
    fn get_mod_sig() -> Signature {
        homogenous_binary(Constraint::Integer, Constraint::Integer)
    }

    fn get_arithmetic_sig() -> Signature {
        homogenous_binary(Constraint::Num, Constraint::Num)
    }

    fn get_ord_sig() -> Signature {
        homogenous_binary(Constraint::Ord, Constraint::Bool)
    }

    fn get_eq_sig() -> Signature {
        homogenous_binary(Constraint::Eq, Constraint::Bool)
    }

    fn get_and_or_sig() -> Signature {
        homogenous_binary(Constraint::Boolean, Constraint::Bool)
    }

    fn get_not_sig() -> Signature {
        unary(Constraint::Num, Constraint::Num)
    }

    //// CONTAINER OPERATIONS ////

    fn get_head_sig() -> Signature {
        unary(Constraint::List, Constraint::Any)
    }

    fn get_tail_sig() -> Signature {
        unary(Constraint::List, Constraint::List)
    }

    fn get_empty_sig() -> Signature {
        unary(Constraint::List, Constraint::Bool)
    }

    fn get_length_sig() -> Signature {
        unary(Constraint::Sized, Constraint::Integer)
    }

    fn get_cons_sig() -> Signature {
        heterogeneous_binary(
            Constraint::Any,
            Constraint::List,
            Constraint::List
        )
    }

    fn get_append_sig() -> Signature {
        homogenous_binary(Constraint::Sized, Constraint::Sized)
    }

    //// HIGHER ORDER ////

    fn get_each_sig() -> Signature {
        let mut sig = unary(Constraint::List, Constraint::Any);
        sig.modifiers = Params::Unary(
            Constraint::Executable
        );
        sig
    }

    fn get_map_sig() -> Signature {
        let mut sig = unary(Constraint::List, Constraint::Quotation);
        sig.modifiers = Params::Unary(Constraint::Executable);
        sig
    }

    pub fn get_foldl_sig() -> Signature {
        let mut sig = heterogeneous_binary(
            Constraint::List,
            Constraint::Any,
            Constraint::Executable
        );
        sig.modifiers = Params::Unary(Constraint::Executable);
        sig
    }

    //// CONTROL ////

    pub fn get_if_sig() -> Signature {
        let mut sig = unary(Constraint::Boolean, Constraint::Any);
        sig.modifiers = Params::Binary(Constraint::Any, Constraint::Any);
        sig
    }

    pub fn get_loop_sig() -> Signature {
        let mut sig = nullary(Constraint::Any);
        sig.modifiers = Params::Binary(Constraint::Executable, Constraint::Executable);
        sig
    }

    pub fn get_times_sig() -> Signature {
        let mut sig = unary(Constraint::Integer, Constraint::Any);
        sig.modifiers = Params::Unary(Constraint::Any);
        sig
    }

    pub fn get_exec_sig() -> Signature {
        unary(Constraint::Executable, Constraint::Executable)
    }

    //// ASSIGNMENT ////

    pub fn get_assign_sig() -> Signature {
        heterogeneous_binary(
            Constraint::Symbol,
            Constraint::Any,
            Constraint::Void
        )
    }

    pub fn get_assign_func_sig() -> Signature {
        heterogeneous_binary (
            Constraint::Symbol,
            Constraint::Executable,
            Constraint::Void
        )
    }

    pub fn get_as_symbol_sig() -> Signature {
        let mut sig = nullary(Constraint::Symbol);
        sig.modifiers = Params::Unary(Constraint::Symbol);
        sig
    }

    pub fn get_eval_symbol_sig() -> Signature {
        unary(Constraint::Symbol, Constraint::Any)
    }

    //// STACK FUNCTIONS ////

    pub fn get_dup_sig() -> Signature {
        unary(Constraint::Any, Constraint::Any)
    }

    pub fn get_swap_sig() -> Signature {
        heterogeneous_binary(
            Constraint::Any,
            Constraint::Any,
            Constraint::Any
        )
    }

    pub fn get_pop_sig() -> Signature {
        unary(Constraint::Any, Constraint::Void)
    }
}

/// Display for Operations
impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Op::Void => write!(f, "()"),
            Op::IOPrint => write!(f, "print"),
            Op::IORead => write!(f, "read"),
            Op::ParseInt => write!(f, "parseInteger"),
            Op::ParseFloat => write!(f, "parseFloat"),
            Op::ParseWords => write!(f, "words"),
            Op::Mod => write!(f, "%"),
            Op::Add => write!(f, "+"),
            Op::Sub => write!(f, "-"),
            Op::Mul => write!(f, "*"),
            Op::Div => write!(f, "/"),
            Op::IntDiv => write!(f, "div"),
            Op::LT => write!(f, "<"),
            Op::GT => write!(f, ">"),
            Op::EQ => write!(f, "=="),
            Op::And => write!(f, "&&"),
            Op::Or => write!(f, "||"),
            Op::Not => write!(f, "not"),
            Op::Head => write!(f, "head"),
            Op::Tail => write!(f, "tail"),
            Op::Empty => write!(f, "empty"),
            Op::Length => write!(f, "length"),
            Op::Cons => write!(f, "cons"),
            Op::Append => write!(f, "append"),
            Op::Each => write!(f, "each"),
            Op::Map => write!(f, "map"),
            Op::Foldl => write!(f, "foldl"),
            Op::If => write!(f, "if"),
            Op::Loop => write!(f, "loop"),
            Op::Times => write!(f, "times"),
            Op::Exec => write!(f, "exec"),
            Op::Assign => write!(f, ":="),
            Op::AssignFunc => write!(f, "fun"),
            Op::AsSymbol => write!(f, "'"),
            Op::EvalSymbol => write!(f, "eval"),
            Op::Dup => write!(f, "dup"),
            Op::Swap => write!(f, "swap"),
            Op::Pop => write!(f, "pop"),
            Op::Error => write!(f, "err"),
        }
    }
}

/// implements FromStr for Op, allowing the use of .parse() to get Op directly
/// from a string.
impl FromStr for Op {
    type Err = String;  // TODO: StackError? Other?

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "print" => Ok(Op::IOPrint),
            "read" => Ok(Op::IORead),
            "parseInteger" => Ok(Op::ParseInt),
            "parseFloat" => Ok(Op::ParseFloat),
            "words" => Ok(Op::ParseWords),
            "%" => Ok(Op::Mod),
            "+" => Ok(Op::Add),
            "-" => Ok(Op::Sub),
            "*" => Ok(Op::Mul),
            "/" => Ok(Op::Div),
            "div" => Ok(Op::IntDiv),
            "<" => Ok(Op::LT),
            ">" => Ok(Op::GT),
            "==" => Ok(Op::EQ),
            "&&" => Ok(Op::And),
            "||" => Ok(Op::Or),
            "not" => Ok(Op::Not),
            "head" => Ok(Op::Head),
            "tail" => Ok(Op::Tail),
            "empty" => Ok(Op::Empty),
            "length" => Ok(Op::Length),
            "cons" => Ok(Op::Cons),
            "append" => Ok(Op::Append),
            "each" => Ok(Op::Each),
            "map" => Ok(Op::Map),
            "foldl" => Ok(Op::Foldl),
            "if" => Ok(Op::If),
            "loop" => Ok(Op::Loop),
            "times" => Ok(Op::Times),
            "exec" => Ok(Op::Exec),
            ":=" => Ok(Op::Assign),
            "fun" => Ok(Op::AssignFunc),
            "'" => Ok(Op::AsSymbol),
            "eval" => Ok(Op::EvalSymbol),
            "pop" => Ok(Op::Pop),
            "swap" => Ok(Op::Swap),
            "dup" => Ok(Op::Dup),
            "()" => Ok(Op::Void),
            "err" => Ok(Op::Error),
            _ => Err(format!("unknown operation: {}", s)),
        }
    }
}

