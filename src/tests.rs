#[cfg(test)]
mod tests {
    use crate::eval::Context;
    use crate::evaluate;

    fn eval(s: &str) -> f64 {
        evaluate(s, &mut Context::new()).expect(s)
    }

    fn eval_err(s: &str) -> String {
        evaluate(s, &mut Context::new()).unwrap_err().to_string()
    }

    // ── basic arithmetic ───────────────────────────────────────────────────
    #[test]
    fn addition() {
        assert_eq!(eval("1 + 2"), 3.0);
    }

    #[test]
    fn subtraction() {
        assert_eq!(eval("10 - 4"), 6.0);
    }

    #[test]
    fn multiplication() {
        assert_eq!(eval("3 * 7"), 21.0);
    }

    #[test]
    fn division() {
        assert_eq!(eval("10 / 4"), 2.5);
    }

    #[test]
    fn remainder() {
        assert_eq!(eval("10 % 3"), 1.0);
    }

    #[test]
    fn power_left_assoc() {
        // Right-associative: 2^3^2 = 2^9 = 512
        assert_eq!(eval("2^3^2"), 512.0);
    }

    #[test]
    fn double_star_power() {
        assert_eq!(eval("2**10"), 1024.0);
    }

    #[test]
    fn unary_minus() {
        assert_eq!(eval("-5"), -5.0);
    }

    #[test]
    fn unary_minus_in_expr() {
        assert_eq!(eval("3 + -2"), 1.0);
    }

    #[test]
    fn precedence_mul_over_add() {
        assert_eq!(eval("2 + 3 * 4"), 14.0);
    }

    #[test]
    fn parentheses() {
        assert_eq!(eval("(2 + 3) * 4"), 20.0);
    }

    #[test]
    fn nested_parens() {
        assert_eq!(eval("((3+4)*(2-1))"), 7.0);
    }

    // ── numeric literals ───────────────────────────────────────────────────
    #[test]
    fn hex_literal() {
        assert_eq!(eval("0xFF"), 255.0);
    }

    #[test]
    fn binary_literal() {
        assert_eq!(eval("0b1010"), 10.0);
    }

    #[test]
    fn scientific_notation() {
        assert_eq!(eval("1.5e2"), 150.0);
    }

    #[test]
    fn negative_exponent() {
        assert!((eval("1e-3") - 0.001).abs() < 1e-15);
    }

    #[test]
    fn underscore_separator() {
        assert_eq!(eval("1_000_000"), 1_000_000.0);
    }

    // ── constants ──────────────────────────────────────────────────────────
    #[test]
    fn constant_pi() {
        assert!((eval("pi") - std::f64::consts::PI).abs() < 1e-15);
    }

    #[test]
    fn constant_e() {
        assert!((eval("e") - std::f64::consts::E).abs() < 1e-15);
    }

    #[test]
    fn constant_phi() {
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        assert!((eval("phi") - phi).abs() < 1e-15);
    }

    // ── variables ──────────────────────────────────────────────────────────
    #[test]
    fn variable_assign_and_read() {
        let mut ctx = Context::new();
        evaluate("x = 7", &mut ctx).unwrap();
        assert_eq!(evaluate("x * 6", &mut ctx).unwrap(), 42.0);
    }

    #[test]
    fn ans_tracks_last_result() {
        let mut ctx = Context::new();
        ctx.record("dummy", 99.0);
        assert_eq!(evaluate("ans + 1", &mut ctx).unwrap(), 100.0);
    }

    // ── trig functions ─────────────────────────────────────────────────────
    #[test]
    fn sin_half_pi() {
        assert!((eval("sin(pi/2)") - 1.0).abs() < 1e-15);
    }

    #[test]
    fn cos_pi() {
        assert!((eval("cos(pi)") - (-1.0)).abs() < 1e-15);
    }

    #[test]
    fn atan2_basic() {
        assert!((eval("atan2(1, 1)") - std::f64::consts::FRAC_PI_4).abs() < 1e-14);
    }

    // ── logarithms ─────────────────────────────────────────────────────────
    #[test]
    fn log10_100() {
        assert!((eval("log10(100)") - 2.0).abs() < 1e-15);
    }

    #[test]
    fn log2_8() {
        assert!((eval("log2(8)") - 3.0).abs() < 1e-15);
    }

    #[test]
    fn ln_e() {
        assert!((eval("ln(e)") - 1.0).abs() < 1e-15);
    }

    // ── rounding ───────────────────────────────────────────────────────────
    #[test]
    fn floor_neg() {
        assert_eq!(eval("floor(-1.3)"), -2.0);
    }

    #[test]
    fn ceil_neg() {
        assert_eq!(eval("ceil(-1.7)"), -1.0);
    }

    #[test]
    fn round_half() {
        assert_eq!(eval("round(2.5)"), 3.0);
    }

    // ── root / power ───────────────────────────────────────────────────────
    #[test]
    fn sqrt_4() {
        assert_eq!(eval("sqrt(4)"), 2.0);
    }

    #[test]
    fn cbrt_27() {
        assert!((eval("cbrt(27)") - 3.0).abs() < 1e-14);
    }

    // ── domain errors ──────────────────────────────────────────────────────
    #[test]
    fn div_zero() {
        assert!(eval_err("1 / 0").contains("division by zero"));
    }

    #[test]
    fn rem_zero() {
        assert!(eval_err("5 % 0").contains("division by zero"));
    }

    #[test]
    fn sqrt_negative() {
        assert!(eval_err("sqrt(-1)").contains("domain error"));
    }

    #[test]
    fn log_zero() {
        assert!(eval_err("log(0)").contains("domain error"));
    }

    #[test]
    fn log_negative() {
        assert!(eval_err("log(-1)").contains("domain error"));
    }

    #[test]
    fn asin_out_of_range() {
        assert!(eval_err("asin(2)").contains("domain error"));
    }

    #[test]
    fn undefined_variable() {
        assert!(eval_err("z + 1").contains("undefined variable"));
    }

    #[test]
    fn undefined_function() {
        assert!(eval_err("foo(1)").contains("undefined function"));
    }

    #[test]
    fn wrong_arg_count() {
        assert!(eval_err("sin(1, 2)").contains("expects"));
    }

    // ── history capacity ───────────────────────────────────────────────────
    #[test]
    fn history_cap() {
        use crate::eval::HISTORY_CAPACITY;
        let mut ctx = Context::new();
        for i in 0..HISTORY_CAPACITY + 10 {
            ctx.record(&format!("{i}"), i as f64);
        }
        assert_eq!(ctx.history.len(), HISTORY_CAPACITY);
    }

    // ── complex expressions ────────────────────────────────────────────────
    #[test]
    fn combined_expression() {
        let r = eval("sin(pi/4) + cos(pi/3) * log10(100)");
        let expected = (std::f64::consts::FRAC_PI_4).sin() + 0.5 * 2.0;
        assert!((r - expected).abs() < 1e-12);
    }

    #[test]
    fn nested_functions() {
        assert!((eval("sqrt(abs(-16))") - 4.0).abs() < 1e-15);
    }

    #[test]
    fn gcd_basic() {
        assert_eq!(eval("gcd(12, 8)"), 4.0);
    }

    #[test]
    fn lcm_basic() {
        assert_eq!(eval("lcm(4, 6)"), 12.0);
    }

    #[test]
    fn clamp_basic() {
        assert_eq!(eval("clamp(10, 0, 5)"), 5.0);
        assert_eq!(eval("clamp(-1, 0, 5)"), 0.0);
        assert_eq!(eval("clamp(3, 0, 5)"), 3.0);
    }
}
