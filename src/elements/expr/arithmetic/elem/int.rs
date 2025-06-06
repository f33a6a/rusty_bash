//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::exec::ExecError;
use crate::utils::exit;
use super::ArithElem;
use super::variable;

pub fn unary_calc(op: &str, num: i128, stack: &mut Vec<ArithElem>) -> Result<(), ExecError> {
    match op {
        "+"  => stack.push( ArithElem::Integer(num) ),
        "-"  => stack.push( ArithElem::Integer(-num) ),
        "!"  => stack.push( ArithElem::Integer(if num == 0 { 1 } else { 0 }) ),
        "~"  => stack.push( ArithElem::Integer( !num ) ),
        _ => exit::internal("unknown unary operator"),
    }
    Ok(())
}

pub fn bin_calc(op: &str, left: i128, right: i128, stack: &mut Vec<ArithElem>) -> Result<(), ExecError> {
    let bool_to_01 = |b| { if b { 1 } else { 0 } };

    let ans = match op {
        "+"  => left + right,
        "-"  => left - right,
        "*"  => left * right,
        "&"  => left & right,
        "^"  => left ^ right,
        "|"  => left | right,
        "&&"  => bool_to_01( left != 0 && right != 0 ),
        "||"  => bool_to_01( left != 0 || right != 0 ),
        "<<"  => if right < 0 {0} else {left << right},
        ">>"  => if right < 0 {0} else {left >> right},
        "<="  => bool_to_01( left <= right ),
        ">="  => bool_to_01( left >= right ),
        "<"  => bool_to_01( left < right ),
        ">"  => bool_to_01( left > right ),
        "=="  => bool_to_01( left == right ),
        "!="  => bool_to_01( left != right ),
        "%" | "/" => {
            if right == 0 {
                let expr = format!("{} / {}", &left, &right);
                return Err(ExecError::DivZero(expr, right.to_string()));
            }
            match op {
                "%" => left % right,
                _   => left / right,
            }
        },
        "**" => {
            if right >= 0 {
                let r = right.try_into().unwrap();
                left.pow(r)
            }else{
                return Err(ExecError::Exponent(right));
            }
        },
        _    => exit::internal("unknown binary operator"),
    };

    stack.push(ArithElem::Integer(ans));
    Ok(())
}

pub fn substitute(op: &str, name: &String, index: &String,
    cur: i128, right: i128, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    let new_value = match op {
        "+=" => cur + right,
        "-=" => cur - right,
        "*=" => cur * right,
        "&="  => cur & right,
        "^="  => cur ^ right,
        "|="  => cur | right,
        "<<="  => if right < 0 {0} else {cur << right},
        ">>="  => if right < 0 {0} else {cur >> right},
        "/=" | "%=" => {
            if right == 0 {
                let expr = format!("{} {} {}", &cur, &op, &right);
                return Err(ExecError::DivZero(expr, right.to_string()));
            }
            match op == "%=" {
                true  => cur % right,
                false => cur / right,
            }
        },
        _   => return Err(ExecError::OperandExpected(op.to_string())),
    };

    core.db.set_param2(&name, index, &new_value.to_string(), None)?;
    Ok(ArithElem::Integer(new_value))
}

fn parse_with_base(base: i128, s: &mut String) -> Result<i128, ExecError> {
    if s.is_empty() {
        return Err(ExecError::InvalidArithmeticOperator(s.clone(), s.clone()));
    }

    let mut ans = 0;
    for ch in s.chars() {
        ans *= base;
        let num = if ch >= '0' && ch <= '9' {
            ch as i128 - '0' as i128
        }else if ch >= 'a' && ch <= 'z' {
            ch as i128 - 'a' as i128 + 10
        }else if ch >= 'A' && ch <= 'Z' {
            match base <= 36 {
                true  => ch as i128 - 'A' as i128 + 10,
                false => ch as i128 - 'A' as i128 + 36,
            }
        }else if ch == '@' {
            62
        }else if ch == '_' {
            63
        }else{
            return Err(ExecError::InvalidArithmeticOperator(s.clone(), ch.to_string()));
        };

        match num < base {
            true  => ans += num,
            false => return Err(ExecError::InvalidBase(base.to_string())),
        }
    }

    Ok(ans)
}

fn get_base(s: &mut String) -> Result<i128, ExecError> {
    if s.starts_with("0x") || s.starts_with("0X") {
        s.remove(0);
        s.remove(0);
        return Ok(16);
    }

    if s.starts_with("0") && s.len() > 1 {
        s.remove(0);
        return Ok(8);
    }

    if let Some(n) = s.find("#") {
        let base_str = s[..n].to_string();
        *s = s[(n+1)..].to_string();
        return match base_str.parse::<i128>() {
            Ok(n) => {
                match n <= 64 {
                    true  => Ok(n),
                    false => Err(ExecError::InvalidBase(base_str)),
                }
            },
            _     => Err(ExecError::InvalidBase(base_str)),
        };
    }

    Ok(10)
}

pub fn parse(s: &str) -> Result<i128, ExecError> {
    if s.find('\'').is_some() 
    || s.find('.').is_some() {
        return Err(ExecError::Other("invalid number".to_string()));
    }
    if s.is_empty() {
        return Ok(0);
    }

    let mut sw = s.to_string();
    let sign = variable::get_sign(&mut sw);
    let base = get_base(&mut sw)?;

    match ( parse_with_base(base, &mut sw), sign.as_str() ) {
        (Ok(n), "-") => Ok(-n), 
        (Ok(n), _)   => Ok(n), 
        (Err(e), _)  => Err(ExecError::Other(format!("{:?}", e))),
    }
}

