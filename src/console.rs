use std::sync::Mutex;

use console::style;
use dialoguer::{Input, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};

static SPINNER: Mutex<Option<ProgressBar>> = Mutex::new(None);

const BANNER: &str = r#"
   ____    _    ____  __  __  _____  _____    _    _   _ 
  / ___|  / \  / ___||  \/  |/ _ \ \/ /_ _|  / \  | \ | |
 | |  _  / _ \ \___ \| |\/| | | | \  / | |  / _ \ |  \| |
 | |_| |/ ___ \ ___) | |  | | |_| /  \ | | / ___ \| |\  |
  \____/_/   \_\____/|_|  |_|\___/_/\_\___/_/   \_\_| \_|
"#;

/// Clears the screen and prints the Gasmoxian banner.
pub fn print_banner() {
    let _ = console::Term::stdout().clear_screen();
    println!("{}", style(BANNER).green());
    println!(
        "{}",
        style("Gasmoxian Client (press CTRL + C to quit)").bold()
    );
    println!();
}

/// Creates a new spinner with the given message.
/// The spinner runs in-place until `spinner_ok()` or `spinner_err()` is called.
pub fn new_spinner(msg: &str) {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb.set_message(msg.to_string());
    *SPINNER.lock().expect("SPINNER lock poisoned") = Some(pb);
}

/// Finishes the current spinner with a success (green ✔) message.
pub fn spinner_ok(msg: &str) {
    if let Some(pb) = SPINNER.lock().expect("SPINNER lock poisoned").take() {
        pb.finish_with_message(format!("{} {}", style("✔").green(), msg));
    }
}

/// Finishes the current spinner with a failure (red ✗) message.
pub fn spinner_err(msg: &str) {
    if let Some(pb) = SPINNER.lock().expect("SPINNER lock poisoned").take() {
        pb.finish_with_message(format!("{} {}", style("✗").red(), msg));
    }
}

/// Prints an informational message in cyan.
pub fn info(msg: impl std::fmt::Display) {
    println!("  {}", style(msg).cyan());
}

/// Prints a success message prefixed with a green check mark.
pub fn ok(msg: impl std::fmt::Display) {
    println!("{} {}", style("✔").green(), msg);
}

/// Prints an error message prefixed with a red cross mark.
pub fn err(msg: impl std::fmt::Display) {
    println!("{} {}", style("✗").red(), msg);
}

/// Prompts the user for a username using dialoguer, falling back to stdin.
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

/// Prompts the user for a private server IP and port via dialoguer.
/// Returns `None` if the user cancels.
///
/// # Panics
/// Panics if dialoguer cannot read from stdin (terminal not available).
pub fn prompt_private_server() -> Option<(String, u16)> {
    let ip: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Server IP")
        .default("127.0.0.1".into())
        .interact_text()
        .ok()?;

    let port: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Server Port (0-65535)")
        .validate_with(|input: &String| -> Result<(), &str> {
            input
                .parse::<u16>()
                .map(|_| ())
                .map_err(|_| "Invalid port, must be 0-65535")
        })
        .interact_text()
        .ok()?;

    port.parse::<u16>().ok().map(|p| (ip, p))
}
