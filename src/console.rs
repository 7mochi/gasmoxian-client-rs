use std::sync::Mutex;

use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use indicatif::{ProgressBar, ProgressStyle};

static SPINNER: Mutex<Option<ProgressBar>> = Mutex::new(None);

const BANNER: &str = r#"
   ____    _    ____  __  __  _____  _____    _    _   _ 
  / ___|  / \  / ___||  \/  |/ _ \ \/ /_ _|  / \  | \ | |
 | |  _  / _ \ \___ \| |\/| | | | \  / | |  / _ \ |  \| |
 | |_| |/ ___ \ ___) | |  | | |_| /  \ | | / ___ \| |\  |
  \____/_/   \_\____/|_|  |_|\___/_/\_\___/_/   \_\_| \_|
"#;

pub fn print_banner() {
    let _ = console::Term::stdout().clear_screen();
    println!("{}", style(BANNER).green());
    println!("{}", style("Gasmoxian Client (press CTRL + C to quit)").bold());
    println!();
}

pub fn new_spinner(msg: &str) {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb.set_message(msg.to_string());
    *SPINNER.lock().unwrap() = Some(pb);
}

pub fn spinner_ok(msg: &str) {
    if let Some(pb) = SPINNER.lock().unwrap().take() {
        pb.finish_with_message(format!("{} {}", style("✔").green(), msg));
    }
}

pub fn spinner_err(msg: &str) {
    if let Some(pb) = SPINNER.lock().unwrap().take() {
        pb.finish_with_message(format!("{} {}", style("✗").red(), msg));
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
