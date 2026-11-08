use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CalcError {
    Syntax,
    DivisionByZero,
    Domain,
    TooLong,
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::i18n::tr(match self {
            Self::Syntax => "Check the expression and parentheses",
            Self::DivisionByZero => "Cannot divide by zero",
            Self::Domain => "Result is outside the real number range",
            Self::TooLong => "Expression is too long or deeply nested",
        }))
    }
}

pub(crate) fn evaluate(input: &str, degrees: bool, answer: f64) -> Result<f64, CalcError> {
    if input.len() > 1024 {
        return Err(CalcError::TooLong);
    }
    let normalized = input
        .replace('×', "*")
        .replace('÷', "/")
        .replace('−', "-")
        .replace('π', "pi");
    let mut parser = Parser {
        input: normalized.as_bytes(),
        pos: 0,
        depth: 0,
        degrees,
        answer,
    };
    let result = parser.expression(0)?;
    parser.spaces();
    if parser.pos != parser.input.len() {
        return Err(CalcError::Syntax);
    }
    finite(result)
}

fn finite(value: f64) -> Result<f64, CalcError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CalcError::Domain)
    }
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    depth: usize,
    degrees: bool,
    answer: f64,
}
impl Parser<'_> {
    fn spaces(&mut self) {
        while self
            .input
            .get(self.pos)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.pos += 1;
        }
    }
    fn take(&mut self, byte: u8) -> bool {
        self.spaces();
        if self.input.get(self.pos) == Some(&byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expression(&mut self, minimum: u8) -> Result<f64, CalcError> {
        self.depth += 1;
        if self.depth > 64 {
            return Err(CalcError::TooLong);
        }
        let mut lhs = if self.take(b'-') {
            -self.expression(5)?
        } else if self.take(b'+') {
            self.expression(5)?
        } else if self.take(b'(') {
            let value = self.expression(0)?;
            if !self.take(b')') {
                return Err(CalcError::Syntax);
            }
            value
        } else {
            self.atom()?
        };
        loop {
            self.spaces();
            let Some(&operator) = self.input.get(self.pos) else {
                break;
            };
            if operator == b'%' || operator == b'!' {
                if 9 < minimum {
                    break;
                }
                self.pos += 1;
                lhs = if operator == b'%' {
                    lhs / 100.0
                } else {
                    if !(0.0..=170.0).contains(&lhs) || lhs.fract() != 0.0 {
                        return Err(CalcError::Domain);
                    }
                    (1..=lhs as u32).fold(1.0, |n, i| n * f64::from(i))
                };
                continue;
            }
            let (left, right) = match operator {
                b'+' | b'-' => (1, 2),
                b'*' | b'/' => (3, 4),
                b'^' => (7, 6),
                _ => break,
            };
            if left < minimum {
                break;
            }
            self.pos += 1;
            let rhs = self.expression(right)?;
            lhs = finite(match operator {
                b'+' => lhs + rhs,
                b'-' => lhs - rhs,
                b'*' => lhs * rhs,
                b'/' => {
                    if rhs == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    lhs / rhs
                }
                b'^' => lhs.powf(rhs),
                _ => unreachable!(),
            })?;
        }
        self.depth -= 1;
        finite(lhs)
    }
    fn atom(&mut self) -> Result<f64, CalcError> {
        self.spaces();
        let start = self.pos;
        if self.input.get(start).is_some_and(u8::is_ascii_alphabetic) {
            while self
                .input
                .get(self.pos)
                .is_some_and(u8::is_ascii_alphabetic)
            {
                self.pos += 1;
            }
            let name = &self.input[start..self.pos];
            match name {
                b"pi" => return Ok(std::f64::consts::PI),
                b"e" => return Ok(std::f64::consts::E),
                b"ans" => return Ok(self.answer),
                _ => {}
            }
            if !self.take(b'(') {
                return Err(CalcError::Syntax);
            }
            let x = self.expression(0)?;
            if !self.take(b')') {
                return Err(CalcError::Syntax);
            }
            let angle = if self.degrees { x.to_radians() } else { x };
            finite(match name {
                b"sin" => angle.sin(),
                b"cos" => angle.cos(),
                b"tan" => {
                    if angle.cos().abs() < 1e-14 {
                        return Err(CalcError::Domain);
                    }
                    angle.tan()
                }
                b"asin" => {
                    let v = x.asin();
                    if self.degrees { v.to_degrees() } else { v }
                }
                b"acos" => {
                    let v = x.acos();
                    if self.degrees { v.to_degrees() } else { v }
                }
                b"atan" => {
                    let v = x.atan();
                    if self.degrees { v.to_degrees() } else { v }
                }
                b"sqrt" => x.sqrt(),
                b"ln" => x.ln(),
                b"log" => x.log10(),
                b"abs" => x.abs(),
                b"exp" => x.exp(),
                _ => return Err(CalcError::Syntax),
            })
        } else {
            while self
                .input
                .get(self.pos)
                .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
            {
                self.pos += 1;
            }
            if self.input.get(self.pos) == Some(&b'e') || self.input.get(self.pos) == Some(&b'E') {
                self.pos += 1;
                if matches!(self.input.get(self.pos), Some(b'+' | b'-')) {
                    self.pos += 1;
                }
                while self.input.get(self.pos).is_some_and(u8::is_ascii_digit) {
                    self.pos += 1;
                }
            }
            std::str::from_utf8(&self.input[start..self.pos])
                .ok()
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or(CalcError::Syntax)
                .and_then(finite)
        }
    }
}

pub(crate) fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".into();
    }
    if value.abs() >= 1e12 || value.abs() < 1e-9 {
        return format!("{value:.10e}");
    }
    let places = (11 - value.abs().log10().floor() as i32).clamp(0, 12) as usize;
    let output = format!("{value:.places$}");
    if output.contains('.') {
        output.trim_end_matches('0').trim_end_matches('.').into()
    } else {
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculator_expression_contract() {
        for (input, expected) in [
            ("2+3*4", 14.0),
            ("(2+3)*4", 20.0),
            ("2^3^2", 512.0),
            ("-2^2", -4.0),
            ("2^-2", 0.25),
            ("5!", 120.0),
            ("200*15%", 30.0),
            ("sin(30)", 0.5),
            ("sqrt(81)+log(100)", 11.0),
            ("1e-3*1000", 1.0),
            ("ans+2", 9.0),
        ] {
            assert!(
                (evaluate(input, true, 7.0).expect(input) - expected).abs() < 1e-10,
                "{input}"
            );
        }
        assert_eq!(
            format_number(evaluate("0.1+0.2", true, 0.0).expect("decimal")),
            "0.3"
        );
        assert!((evaluate("sin(pi/2)", false, 0.0).expect("radians") - 1.0).abs() < 1e-10);
    }
    #[test]
    fn calculator_rejects_invalid_and_unbounded_input() {
        assert_eq!(evaluate("1/0", true, 0.0), Err(CalcError::DivisionByZero));
        for expression in ["sqrt(-1)", "ln(0)", "171!", "2.5!", "tan(90)", "10^1000"] {
            assert_eq!(evaluate(expression, true, 0.0), Err(CalcError::Domain));
        }
        for expression in ["", "2+", "(2", "2)", "garbage(2)", "1..2", "2 3"] {
            assert_eq!(evaluate(expression, true, 0.0), Err(CalcError::Syntax));
        }
        assert_eq!(
            evaluate(&"(".repeat(70), true, 0.0),
            Err(CalcError::TooLong)
        );
        assert_eq!(
            evaluate(&"1".repeat(1025), true, 0.0),
            Err(CalcError::TooLong)
        );
    }
}
