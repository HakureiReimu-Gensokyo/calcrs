#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use calcrs::eval::Context;
use calcrs::evaluate;
use colored::Colorize as _;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut rl = match DefaultEditor::new() {
        Ok(ed) => ed,
        Err(e) => {
            eprintln!("fatal: could not initialise readline: {e}");
            std::process::exit(1);
        }
    };

    let mut ctx = Context::new();

    println!("{}", format!("  calcrs {VERSION}").bold());
    println!(
        "{}",
        "  type 'help' for reference, 'exit' to quit\n".dimmed()
    );

    loop {
        match rl.readline("calc› ") {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(input);
                handle(input, &mut ctx);
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("{}", "\n  bye.".dimmed());
                break;
            }
            Err(e) => {
                eprintln!("{} {e}", "  readline:".red());
                break;
            }
        }
    }
}

fn handle(input: &str, ctx: &mut Context) {
    match input {
        "exit" | "quit" | "q" => {
            println!("{}", "  bye.".dimmed());
            std::process::exit(0);
        }
        "help" | "?" => print_help(),
        "history" | "hist" => print_history(ctx),
        "vars" => print_vars(ctx),
        "clear" => {
            ctx.history.clear();
            println!("{}", "  history cleared.".dimmed());
        }
        _ => match evaluate(input, ctx) {
            Ok(v) => {
                ctx.record(input, v);
                println!("  {}  {}\n", "=".dimmed(), fmt(v).yellow().bold());
            }
            Err(e) => {
                eprintln!("  {}  {}\n", "✗".red().bold(), e.to_string().red());
            }
        },
    }
}

/// Format an `f64` cleanly:
/// * Integer-valued floats show without a decimal point
/// * Others show up to 10 significant digits with trailing zeros stripped
/// * Fall back to scientific notation for very large exponents
fn fmt(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1.0e15 {
        return format!("{v:.0}");
    }
    let s = format!("{v:.10}");
    let t = s.trim_end_matches('0').trim_end_matches('.');
    if t.len() > 22 {
        format!("{v:.8e}")
    } else {
        t.to_owned()
    }
}

fn print_history(ctx: &Context) {
    if ctx.history.is_empty() {
        println!("{}", "  (empty)".dimmed());
        return;
    }
    for (i, (expr, val)) in ctx.history.iter().enumerate() {
        println!(
            "  {} {}  {}  {}",
            format!("[{:>2}]", i + 1).dimmed(),
            expr.cyan(),
            "=".dimmed(),
            fmt(*val).yellow(),
        );
    }
    println!();
}

fn print_vars(ctx: &Context) {
    let mut pairs: Vec<_> = ctx.vars().iter().collect();
    pairs.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in pairs {
        println!("  {} {} {}", k.cyan(), "=".dimmed(), fmt(*v).yellow());
    }
    println!();
}

fn print_help() {
    println!(
        r#"
  {title}

  {ops}
    +  -  *  /  %  ^  (**)        arithmetic & exponentiation (^ is right-assoc)
    name = <expr>                  variable assignment

  {consts}
    pi  π   e   phi  φ   sqrt2    mathematical constants
    ans                            last result
    inf                            +infinity

  {trig}
    sin  cos  tan                  radians in, radians out
    asin  acos  atan               inverse trig
    atan2(y, x)                    four-quadrant arctangent
    sinh  cosh  tanh               hyperbolic
    deg(x)  rad(x)                 convert radians↔degrees

  {explog}
    exp  exp2  ln  log  log2  log10  logb(x, base)

  {rootpow}
    sqrt  √   cbrt   pow(b, e)   hypot(a, b)

  {round}
    abs   floor   ceil   round   trunc   fract   sign

  {misc}
    min(a,b)  max(a,b)  clamp(x, lo, hi)  gcd(a,b)  lcm(a,b)

  {lits}
    1_000.5e-3    decimal / scientific notation (underscores ignored)
    0xFF          hexadecimal
    0b1010_0001   binary

  {cmds}
    history / hist   show last 50 calculations
    vars             show all variables
    clear            clear history
    exit / quit / q  quit
"#,
        title = "calcrs reference".bold().underline(),
        ops = "Operators".underline(),
        consts = "Constants".underline(),
        trig = "Trigonometric".underline(),
        explog = "Exponential / Logarithmic".underline(),
        rootpow = "Root / Power".underline(),
        round = "Rounding".underline(),
        misc = "Misc".underline(),
        lits = "Numeric literals".underline(),
        cmds = "Session commands".underline(),
    );
}
