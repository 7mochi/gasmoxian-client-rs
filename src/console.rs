//! Terminal logging and user input.
//!
//! Four severity levels: `debug` (only shown with `LOG=debug`),
//! `info`, `ok`, and `err`. Also provides banner printing and
//! an interactive username prompt.

use std::sync::atomic::{AtomicBool, Ordering};

use console::style;
use dialoguer::{Input, theme::ColorfulTheme};

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn init_debug() {
    let enabled = std::env::var("LOG")
        .map(|v| matches!(v.to_lowercase().as_str(), "debug" | "1" | "true"))
        .unwrap_or(false);
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

const BANNER: &str = r#"
   ____    _    ____  __  __  _____  _____    _    _   _ 
  / ___|  / \  / ___||  \/  |/ _ \ \/ /_ _|  / \  | \ | |
 | |  _  / _ \ \___ \| |\/| | | | \  / | |  / _ \ |  \| |
 | |_| |/ ___ \ ___) | |  | | |_| /  \ | | / ___ \| |\  |
  \____/_/   \_\____/|_|  |_|\___/_/\_\___/_/   \_\_| \_|
"#;

pub fn debug(msg: impl std::fmt::Display) {
    if DEBUG_ENABLED.load(Ordering::Relaxed) {
        println!("{} {}", style("[DEBUG]").dim(), style(msg).dim());
    }
}

pub fn info(msg: impl std::fmt::Display) {
    println!("  {}", style(msg).cyan());
}

pub fn ok(msg: impl std::fmt::Display) {
    println!("{} {}", style("✔").green(), msg);
}

pub fn err(msg: impl std::fmt::Display) {
    println!("{} {}", style("✗").red(), msg);
}

pub fn print_banner() {
    let _ = console::Term::stdout().clear_screen();
    println!("{}", style(BANNER).green());
    println!(
        "{}",
        style("Gasmoxian client (press CTRL + C to quit)").bold()
    );
    println!();
}

pub fn prompt_username() -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Your username")
        .interact_text()
        .unwrap_or_else(|_| {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            input.trim().to_string()
        })
}
