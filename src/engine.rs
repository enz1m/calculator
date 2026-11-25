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

// Results within a few ulps of the operands' magnitude are binary rounding residue.
const RESIDUE: f64 = 4.0 * f64::EPSILON;

// Large angles keep their result: there the tolerance would swallow real values.
fn without_residue(value: f64, operand_scale: f64) -> f64 {
    let tolerance = operand_scale.abs() * RESIDUE;
    if tolerance < 1e-9 && value.abs() <= tolerance {
        0.0
    } else {
        value
    }
}

// A sum is only as precise as its larger operand, so digits below that are dropped.
fn settled_sum(value: f64, operand_scale: f64) -> f64 {
    let quantum = 10f64.powi((operand_scale.abs() * RESIDUE).log10().ceil() as i32);
    if !quantum.is_normal() || !value.is_finite() {
        return value;
    }
    (value / quantum).round() * quantum
}

fn finite(value: f64) -> Result<f64, CalcError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CalcError::Domain)
    }
}

// `percent` marks a value whose outermost operation is a postfix `%`.
struct Operand {
    value: f64,
    percent: bool,
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
        self.operand(minimum).map(|operand| operand.value)
    }
    fn operand(&mut self, minimum: u8) -> Result<Operand, CalcError> {
        self.depth += 1;
        if self.depth > 64 {
            return Err(CalcError::TooLong);
        }
        let (mut lhs, mut percent) = if self.take(b'-') {
            let operand = self.operand(5)?;
            (-operand.value, operand.percent)
        } else if self.take(b'+') {
            let operand = self.operand(5)?;
            (operand.value, operand.percent)
        } else if self.take(b'(') {
            let value = self.expression(0)?;
            if !self.take(b')') {
                return Err(CalcError::Syntax);
            }
            (value, false)
        } else {
            (self.atom()?, false)
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
                percent = operator == b'%';
                lhs = if percent {
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
            let rhs = self.operand(right)?;
            // A percentage added or subtracted is a share of the left operand.
            let rhs = match operator {
                b'+' | b'-' if rhs.percent => lhs * rhs.value,
                _ => rhs.value,
            };
            percent = false;
            lhs = finite(match operator {
                b'+' => settled_sum(lhs + rhs, lhs.abs().max(rhs.abs())),
                b'-' => settled_sum(lhs - rhs, lhs.abs().max(rhs.abs())),
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
        Ok(Operand {
            value: finite(lhs)?,
            percent,
        })
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
            let turn_degrees = x.rem_euclid(360.0);
            let quarter_turn = (self.degrees && turn_degrees % 90.0 == 0.0)
                .then_some(turn_degrees / 90.0)
                .map(|quarters| quarters as u8 % 4);
            let angle = if self.degrees {
                turn_degrees.to_radians()
            } else {
                x
            };
            finite(match (name, quarter_turn) {
                (b"sin", Some(quarters)) => [0.0, 1.0, 0.0, -1.0][usize::from(quarters)],
                (b"cos", Some(quarters)) => [1.0, 0.0, -1.0, 0.0][usize::from(quarters)],
                (b"tan", Some(0 | 2)) => 0.0,
                (b"tan", Some(_)) => return Err(CalcError::Domain),
                (b"sin", None) => without_residue(angle.sin(), angle),
                (b"cos", None) => without_residue(angle.cos(), angle),
                (b"tan", None) => {
                    if angle.cos().abs() < 1e-14 {
                        return Err(CalcError::Domain);
                    }
                    without_residue(angle.tan(), angle)
                }
                (b"asin", _) => {
                    let v = x.asin();
                    if self.degrees { v.to_degrees() } else { v }
                }
                (b"acos", _) => {
                    let v = x.acos();
                    if self.degrees { v.to_degrees() } else { v }
                }
                (b"atan", _) => {
                    let v = x.atan();
                    if self.degrees { v.to_degrees() } else { v }
                }
                (b"sqrt", _) => x.sqrt(),
                (b"ln", _) => x.ln(),
                (b"log", _) => x.log10(),
                (b"abs", _) => x.abs(),
                (b"exp", _) => x.exp(),
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
    let significant_places = 11 - value.abs().log10().floor() as i32;
    let places = significant_places.clamp(0, 20) as usize;
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
        for (input, degrees, shown) in [
            ("0.1+0.2", true, "0.3"),
            ("0.1+0.2-0.3", true, "0"),
            ("sin(180)", true, "0"),
            ("cos(90)", true, "0"),
            ("cos(-180)", true, "-1"),
            ("tan(540)", true, "0"),
            ("sin(pi)", false, "0"),
            ("1.23456789012e-7", true, "0.000000123456789012"),
            ("sin(-1e-20)", true, "0"),
            ("123456789+0.000001-123456789", true, "0.000001"),
            ("1e-300+1e-300", true, "2.0000000000e-300"),
        ] {
            assert_eq!(
                format_number(evaluate(input, degrees, 0.0).expect(input)),
                shown,
                "{input}"
            );
        }
        assert!((evaluate("sin(pi/2)", false, 0.0).expect("radians") - 1.0).abs() < 1e-10);
        assert!(evaluate("sin(1e20)", false, 0.0).expect("radians").abs() > 0.1);
    }
    #[test]
    fn percent_after_addition_or_subtraction_scales_the_left_operand() {
        for (input, shown) in [
            ("50+10%", "55"),
            ("50−10%", "45"),
            ("200-25%", "150"),
            ("100+10%+10%", "121"),
            ("100+10%-10%", "99"),
            ("50+-10%", "45"),
            ("50×10%", "5"),
            ("50÷10%", "500"),
            ("10%", "0.1"),
            ("50+2*10%", "50.2"),
            ("50+(10%)", "50.1"),
            ("50+300%!", "56"),
            ("2*(50+10%)", "110"),
            ("3+50+10%", "58.3"),
        ] {
            assert_eq!(
                format_number(evaluate(input, true, 0.0).expect(input)),
                shown,
                "{input}"
            );
        }
    }
    #[test]
    fn calculator_rejects_invalid_and_unbounded_input() {
        assert_eq!(evaluate("1/0", true, 0.0), Err(CalcError::DivisionByZero));
        for expression in [
            "sqrt(-1)", "ln(0)", "171!", "2.5!", "tan(90)", "tan(-90)", "10^1000",
        ] {
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
