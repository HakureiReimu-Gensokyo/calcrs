use std::collections::{HashMap, VecDeque};

use crate::error::{CalcError, Result};
use crate::parser::{BinOp, Expr};

/// Maximum number of history entries retained per session.
pub const HISTORY_CAPACITY: usize = 50;

/// Session state: variable bindings and calculation history.
pub struct Context {
    vars: HashMap<String, f64>,
    /// Ring-buffer of `(source_text, result)` pairs, newest at the back.
    pub history: VecDeque<(String, f64)>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Initialise a fresh context with all built-in constants pre-loaded.
    #[must_use]
    pub fn new() -> Self {
        let phi: f64 = (1.0 + 5.0_f64.sqrt()) / 2.0;

        let vars = HashMap::from([
            ("pi".into(), std::f64::consts::PI),
            ("π".into(), std::f64::consts::PI),
            ("e".into(), std::f64::consts::E),
            ("phi".into(), phi),
            ("φ".into(), phi),
            ("sqrt2".into(), std::f64::consts::SQRT_2),
            ("inf".into(), f64::INFINITY),
            ("ans".into(), 0.0),
        ]);

        Self {
            vars,
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
        }
    }

    /// Evaluate `expr` and return a finite, non-NaN `f64`.
    ///
    /// # Errors
    ///
    /// | Variant | Cause |
    /// |---------|-------|
    /// | [`CalcError::DivisionByZero`] | divisor is ±0 |
    /// | [`CalcError::Domain`]         | argument outside function's domain |
    /// | [`CalcError::Overflow`]       | result is ±∞ |
    /// | [`CalcError::UndefinedVariable`] | unknown identifier |
    /// | [`CalcError::UndefinedFunction`] | unknown function name |
    /// | [`CalcError::ArgCount`]       | wrong number of arguments |
    pub fn eval(&mut self, expr: &Expr) -> Result<f64> {
        let v = self.eval_node(expr)?;
        if v.is_nan() {
            return Err(CalcError::Domain {
                name: "result",
                msg: "evaluated to NaN".into(),
            });
        }
        if v.is_infinite() {
            return Err(CalcError::Overflow);
        }
        Ok(v)
    }

    /// Immutable view of current variable bindings (includes constants).
    #[must_use]
    pub fn vars(&self) -> &HashMap<String, f64> {
        &self.vars
    }

    /// Record a successful computation in history and update `ans`.
    pub fn record(&mut self, source: &str, value: f64) {
        if self.history.len() >= HISTORY_CAPACITY {
            self.history.pop_front();
        }
        self.history.push_back((source.to_owned(), value));
        self.vars.insert("ans".to_owned(), value);
    }

    // ── private recursive evaluator ────────────────────────────────────────

    fn eval_node(&mut self, expr: &Expr) -> Result<f64> {
        match expr {
            Expr::Number(n) => Ok(*n),

            Expr::Var(name) => self
                .vars
                .get(name.as_str())
                .copied()
                .ok_or_else(|| CalcError::UndefinedVariable(name.clone())),

            Expr::Assign(name, rhs) => {
                let v = self.eval_node(rhs)?;
                self.vars.insert(name.clone(), v);
                Ok(v)
            }

            Expr::Neg(inner) => Ok(-self.eval_node(inner)?),

            Expr::BinOp { lhs, op, rhs } => {
                let l = self.eval_node(lhs)?;
                let r = self.eval_node(rhs)?;
                apply_binop(*op, l, r)
            }

            Expr::Call { name, args } => self.eval_call(name, args),
        }
    }

    /// Evaluate exactly `n` arguments for a function named `fname`.
    fn eval_args(&mut self, fname: &'static str, args: &[Expr], n: usize) -> Result<Vec<f64>> {
        if args.len() != n {
            return Err(CalcError::ArgCount {
                name: fname,
                expected: n,
                got: args.len(),
            });
        }
        let mut out = Vec::with_capacity(n);
        for a in args {
            out.push(self.eval_node(a)?);
        }
        Ok(out)
    }

    #[inline]
    fn arg1(&mut self, fname: &'static str, args: &[Expr]) -> Result<f64> {
        Ok(self.eval_args(fname, args, 1)?[0])
    }

    #[inline]
    fn arg2(&mut self, fname: &'static str, args: &[Expr]) -> Result<(f64, f64)> {
        let v = self.eval_args(fname, args, 2)?;
        Ok((v[0], v[1]))
    }

    #[inline]
    fn arg3(&mut self, fname: &'static str, args: &[Expr]) -> Result<(f64, f64, f64)> {
        let v = self.eval_args(fname, args, 3)?;
        Ok((v[0], v[1], v[2]))
    }

    #[allow(clippy::too_many_lines)]
    fn eval_call(&mut self, name: &str, args: &[Expr]) -> Result<f64> {
        match name {
            // ── trigonometric ──────────────────────────────────────────────
            "sin" => Ok(self.arg1("sin", args)?.sin()),
            "cos" => Ok(self.arg1("cos", args)?.cos()),
            "tan" => Ok(self.arg1("tan", args)?.tan()),
            "sinh" => Ok(self.arg1("sinh", args)?.sinh()),
            "cosh" => Ok(self.arg1("cosh", args)?.cosh()),
            "tanh" => Ok(self.arg1("tanh", args)?.tanh()),

            "asin" => {
                let x = self.arg1("asin", args)?;
                if !(-1.0..=1.0).contains(&x) {
                    return Err(CalcError::Domain {
                        name: "asin",
                        msg: format!("argument {x} outside [-1, 1]"),
                    });
                }
                Ok(x.asin())
            }
            "acos" => {
                let x = self.arg1("acos", args)?;
                if !(-1.0..=1.0).contains(&x) {
                    return Err(CalcError::Domain {
                        name: "acos",
                        msg: format!("argument {x} outside [-1, 1]"),
                    });
                }
                Ok(x.acos())
            }
            "atan" => Ok(self.arg1("atan", args)?.atan()),
            "atan2" => {
                let (y, x) = self.arg2("atan2", args)?;
                Ok(y.atan2(x))
            }

            // ── angle conversion ───────────────────────────────────────────
            "deg" => Ok(self.arg1("deg", args)?.to_degrees()),
            "rad" => Ok(self.arg1("rad", args)?.to_radians()),

            // ── exponential / logarithmic ──────────────────────────────────
            "exp" => Ok(self.arg1("exp", args)?.exp()),
            "exp2" => Ok(self.arg1("exp2", args)?.exp2()),

            "ln" | "log" => {
                let x = self.arg1(if name == "ln" { "ln" } else { "log" }, args)?;
                require_positive(name, x)?;
                Ok(x.ln())
            }
            "log2" => {
                let x = self.arg1("log2", args)?;
                require_positive("log2", x)?;
                Ok(x.log2())
            }
            "log10" => {
                let x = self.arg1("log10", args)?;
                require_positive("log10", x)?;
                Ok(x.log10())
            }
            // logb(x, base) — arbitrary-base logarithm
            "logb" => {
                let (x, base) = self.arg2("logb", args)?;
                require_positive("logb: x", x)?;
                require_positive("logb: base", base)?;
                Ok(x.log(base))
            }

            // ── root / power ───────────────────────────────────────────────
            "sqrt" | "√" => {
                let x = self.arg1("sqrt", args)?;
                require_non_negative("sqrt", x)?;
                Ok(x.sqrt())
            }
            "cbrt" => Ok(self.arg1("cbrt", args)?.cbrt()),
            "pow" => {
                let (b, e) = self.arg2("pow", args)?;
                Ok(b.powf(e))
            }
            "hypot" => {
                let (a, b) = self.arg2("hypot", args)?;
                Ok(a.hypot(b))
            }

            // ── rounding ───────────────────────────────────────────────────
            "abs" => Ok(self.arg1("abs", args)?.abs()),
            "floor" => Ok(self.arg1("floor", args)?.floor()),
            "ceil" => Ok(self.arg1("ceil", args)?.ceil()),
            "round" => Ok(self.arg1("round", args)?.round()),
            "trunc" => Ok(self.arg1("trunc", args)?.trunc()),
            "fract" => Ok(self.arg1("fract", args)?.fract()),
            "sign" | "signum" => Ok(self.arg1("sign", args)?.signum()),

            // ── min / max / clamp ──────────────────────────────────────────
            "min" => {
                let (a, b) = self.arg2("min", args)?;
                Ok(a.min(b))
            }
            "max" => {
                let (a, b) = self.arg2("max", args)?;
                Ok(a.max(b))
            }
            "clamp" => {
                let (x, lo, hi) = self.arg3("clamp", args)?;
                Ok(x.clamp(lo, hi))
            }

            // ── integer arithmetic helpers ─────────────────────────────────
            "gcd" => {
                let (a, b) = self.arg2("gcd", args)?;
                Ok(euclidean_gcd(a.abs(), b.abs()))
            }
            "lcm" => {
                let (a, b) = self.arg2("lcm", args)?;
                let g = euclidean_gcd(a.abs(), b.abs());
                #[allow(clippy::float_cmp)]
                if g == 0.0 {
                    Ok(0.0)
                } else {
                    Ok((a.abs() / g) * b.abs())
                }
            }

            _ => Err(CalcError::UndefinedFunction(name.to_owned())),
        }
    }
}

// ── free-standing helpers ──────────────────────────────────────────────────

#[allow(clippy::float_cmp)]
#[inline]
fn apply_binop(op: BinOp, l: f64, r: f64) -> Result<f64> {
    match op {
        BinOp::Add => Ok(l + r),
        BinOp::Sub => Ok(l - r),
        BinOp::Mul => Ok(l * r),
        BinOp::Div => {
            if r == 0.0 {
                Err(CalcError::DivisionByZero)
            } else {
                Ok(l / r)
            }
        }
        BinOp::Rem => {
            if r == 0.0 {
                Err(CalcError::DivisionByZero)
            } else {
                Ok(l % r)
            }
        }
        BinOp::Pow => Ok(l.powf(r)),
    }
}

#[inline]
fn require_positive(name: &str, x: f64) -> Result<()> {
    if x <= 0.0 {
        Err(CalcError::Domain {
            name: "require_positive",
            msg: format!("{name}({x}): argument must be > 0"),
        })
    } else {
        Ok(())
    }
}

#[inline]
fn require_non_negative(name: &str, x: f64) -> Result<()> {
    if x < 0.0 {
        Err(CalcError::Domain {
            name: "require_non_negative",
            msg: format!("{name}({x}): argument must be ≥ 0"),
        })
    } else {
        Ok(())
    }
}

/// Euclidean GCD on the f64 integer subset.
#[allow(clippy::float_cmp)]
#[inline]
fn euclidean_gcd(mut a: f64, mut b: f64) -> f64 {
    while b != 0.0 {
        (a, b) = (b, a % b);
    }
    a
}
