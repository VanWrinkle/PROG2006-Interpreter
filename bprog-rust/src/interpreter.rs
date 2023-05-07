use std::collections::VecDeque;
use num::bigint::Sign;
use crate::op::Op;
use crate::parsed::Parsed;
use crate::stack::Stack;
use crate::types::{Params, Constraint, Type};
use crate::types::Signature;

pub fn run(stack: &mut Stack<Parsed>, input: &mut VecDeque<Parsed>) {
    while !input.is_empty() {
        if let Some(p) = input.pop_front() {
            match p {
                Parsed::Function(op) => {
                    exec_op(op, stack, input )
                },
                other => {
                    stack.push(other)
                }
            }
        }
    }
}



fn exec_op(op: Op, stack: &mut Stack<Parsed>, input: &mut VecDeque<Parsed>) {
    let signature = op.get_signature();
    match &signature.stack_args {
        Params::Nullary => {
            if signature.ret != Constraint::Void {
                stack.push(op.exec_nullary())
            }
        }
        Params::Unary(c) => {
            let arg = stack.pop().unwrap();
            if c.is_satisfied_by(&arg.get_type()) {
                let res = op.exec_unary(arg);
                //TODO: Error handling
                if signature.ret != Constraint::Void {
                    stack.push(res)
                }
            } else {
                print_mismatch_arg(op, signature.stack_args, Args::Unary(arg.get_type()))
            }
        },
        Params::Binary(c1, c2) => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            // Checks that the constraints of the function signature is satisfied.
            if c1.is_satisfied_by(&lhs.get_type()) &&
                c2.is_satisfied_by(&rhs.get_type()) {

                let res = op.exec_binary(&lhs, &rhs);
                if signature.ret.is_satisfied_by(&res.get_type()) {
                    stack.push(res);
                } else {
                    println!("Shit the bed! {}", res);
                };
            } else {
                print_mismatch_arg(op, signature.stack_args, Args::Binary(
                    lhs.get_type(),
                    rhs.get_type())
                )
            }
        },
        Params::Temary(c1, c2, c3) => {

        }
    }
}

fn print_mismatch_arg(op: Op, exp: Params, got: Args) {
    match (exp, got) {
        (Params::Unary(expected), Args::Unary(actual)) => {
            println!("err: argument of type \x1b[33m{}\x1b[0m does not satisfy constraint in \
            the function \x1b[36m{}\x1b[0m, with signature (\x1b[31m{}\x1b[0m -> {}).", actual, op, expected, op.get_signature().ret)
        },
        (Params::Binary(exp1, exp2), Args::Binary(act1, act2)) => {
            println!("bug: {} {}", act1, act2);
            let lhs = !exp1.is_satisfied_by(&act1);
            let rhs = !exp2.is_satisfied_by(&act2);
            let mut do_grammar = "does";
            print!("err: ");
            if lhs {
                print!("first argument of type \x1b[33m{}\x1b[0m ", act1);
            }
            if lhs && rhs {
                do_grammar = "do";
                print!("and ")
            }
            if rhs {
                print!("second argument of type \x1b[33m{}\x1b[0m ", act2);
            }
            print!("{} not match constraints in the function \x1b[36m{}\x1b[0m, with signature (", do_grammar, op);
            if lhs {
                print!("\x1b[31m{}\x1b[0m, ", exp1)
            } else {
                print!("{}, ", exp1)
            }
            if rhs {
                print!("\x1b[31m{}\x1b[0m ", exp2)
            } else {
                print!("{} ", exp1)
            }
            println!("-> {})", op.get_signature().ret);
        }
        _ => {}
    }
}

enum Args {
    Unary(Type),
    Binary(Type, Type),
    Temary(Type, Type, Type)
}










