//! Évaluateur d'expressions arithmétiques (descente récursive).
//!
//! Grammaire :
//!   expr   := term (('+' | '-') term)*
//!   term   := factor (('*' | '/') factor)*
//!   factor := ('+' | '-') factor | primary '%'?
//!   primary:= nombre | '(' expr ')'

pub fn evaluate(input: &str) -> Result<f64, String> {
    let chars: Vec<char> = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '×' => '*',
            '÷' => '/',
            '−' => '-',
            ',' => '.',
            other => other,
        })
        .collect();
    if chars.is_empty() {
        return Ok(0.0);
    }
    let mut p = Parser { chars, pos: 0 };
    let value = p.expr()?;
    if p.pos < p.chars.len() {
        return Err(format!("Caractère inattendu « {} »", p.chars[p.pos]));
    }
    if !value.is_finite() {
        return Err("Résultat indéfini".into());
    }
    Ok(value)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn expr(&mut self) -> Result<f64, String> {
        let mut acc = self.term()?;
        while let Some(op @ ('+' | '-')) = self.peek() {
            self.pos += 1;
            let rhs = self.term()?;
            acc = if op == '+' { acc + rhs } else { acc - rhs };
        }
        Ok(acc)
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut acc = self.factor()?;
        while let Some(op @ ('*' | '/')) = self.peek() {
            self.pos += 1;
            let rhs = self.factor()?;
            if op == '*' {
                acc *= rhs;
            } else {
                if rhs == 0.0 {
                    return Err("Division par zéro".into());
                }
                acc /= rhs;
            }
        }
        Ok(acc)
    }

    fn factor(&mut self) -> Result<f64, String> {
        match self.peek() {
            Some('-') => {
                self.pos += 1;
                Ok(-self.factor()?)
            }
            Some('+') => {
                self.pos += 1;
                self.factor()
            }
            _ => {
                let mut v = self.primary()?;
                while self.peek() == Some('%') {
                    self.pos += 1;
                    v /= 100.0;
                }
                Ok(v)
            }
        }
    }

    fn primary(&mut self) -> Result<f64, String> {
        match self.peek() {
            Some('(') => {
                self.pos += 1;
                let v = self.expr()?;
                if self.peek() != Some(')') {
                    return Err("Parenthèse fermante manquante".into());
                }
                self.pos += 1;
                Ok(v)
            }
            Some(c) if c.is_ascii_digit() || c == '.' => {
                let start = self.pos;
                while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '.') {
                    self.pos += 1;
                }
                let s: String = self.chars[start..self.pos].iter().collect();
                s.parse::<f64>()
                    .map_err(|_| format!("Nombre invalide « {s} »"))
            }
            Some(c) => Err(format!("Caractère inattendu « {c} »")),
            None => Err("Expression incomplète".into()),
        }
    }
}

/// Formate un résultat sans zéros inutiles ni bruit flottant.
pub fn format_number(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    if v.abs() >= 1e15 || v.abs() < 1e-9 {
        return format!("{v:e}");
    }
    let s = format!("{:.10}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priorites() {
        assert_eq!(evaluate("2+3*4").unwrap(), 14.0);
        assert_eq!(evaluate("(2+3)×4").unwrap(), 20.0);
        assert_eq!(evaluate("10÷4").unwrap(), 2.5);
    }

    #[test]
    fn unaires_et_pourcent() {
        assert_eq!(evaluate("-3+5").unwrap(), 2.0);
        assert_eq!(evaluate("2*-3").unwrap(), -6.0);
        assert_eq!(evaluate("50%").unwrap(), 0.5);
        assert_eq!(evaluate("1,5+1").unwrap(), 2.5);
    }

    #[test]
    fn erreurs() {
        assert!(evaluate("1/0").is_err());
        assert!(evaluate("(1+2").is_err());
        assert!(evaluate("3+").is_err());
        assert!(evaluate("1..2").is_err());
    }

    #[test]
    fn formatage() {
        assert_eq!(format_number(0.1 + 0.2), "0,3");
        assert_eq!(format_number(14.0), "14");
        assert_eq!(format_number(-2.5), "-2,5");
    }
}
