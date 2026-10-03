// Masque la console en mode release sous Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod eval;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;

const CSS: &str = include_str!("style.css");

fn main() {
    let window = WindowBuilder::new()
        .with_title("Calculatrice")
        .with_inner_size(LogicalSize::new(340.0, 520.0))
        .with_min_inner_size(LogicalSize::new(280.0, 440.0));

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window).with_menu(None))
        .launch(App);
}

/// Touches : (libellé affiché, classe CSS).
const KEYS: [(&str, &str); 20] = [
    ("C", "fn"), ("(", "fn"), (")", "fn"), ("÷", "op"),
    ("7", ""), ("8", ""), ("9", ""), ("×", "op"),
    ("4", ""), ("5", ""), ("6", ""), ("−", "op"),
    ("1", ""), ("2", ""), ("3", ""), ("+", "op"),
    ("0", ""), (",", ""), ("⌫", "fn"), ("=", "eq"),
];

#[component]
fn App() -> Element {
    let mut expr = use_signal(String::new);
    let mut history = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let mut press = move |key: &str| {
        error.set(None);
        match key {
            "C" => {
                expr.set(String::new());
                history.set(String::new());
            }
            "⌫" => {
                expr.write().pop();
            }
            "=" => {
                let current = expr();
                if current.is_empty() {
                    return;
                }
                match eval::evaluate(&current) {
                    Ok(v) => {
                        history.set(format!("{current} ="));
                        expr.set(eval::format_number(v));
                    }
                    Err(e) => error.set(Some(e)),
                }
            }
            k => expr.write().push_str(k),
        }
    };

    // Aperçu du résultat pendant la saisie.
    let preview = eval::evaluate(&expr())
        .ok()
        .filter(|_| !expr().is_empty())
        .map(eval::format_number)
        .unwrap_or_default();

    rsx! {
        style { {CSS} }
        div {
            class: "calc",
            tabindex: 0,
            autofocus: true,
            onkeydown: move |e: KeyboardEvent| {
                let key: Option<String> = match e.key() {
                    Key::Enter => Some("=".into()),
                    Key::Backspace => Some("⌫".into()),
                    Key::Escape | Key::Delete => Some("C".into()),
                    Key::Character(c) => match c.as_str() {
                        "*" | "x" => Some("×".into()),
                        "/" => Some("÷".into()),
                        "-" => Some("−".into()),
                        "." | "," => Some(",".into()),
                        s if s.len() == 1 && "0123456789+()%=".contains(s) => Some(c),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(k) = key {
                    e.prevent_default();
                    press(&k);
                }
            },
            div { class: "screen",
                div { class: "history", "{history}" }
                div { class: "expr", if expr().is_empty() { "0" } else { "{expr}" } }
                div { class: if error().is_some() { "preview err" } else { "preview" },
                    if let Some(msg) = error() { "{msg}" } else { "{preview}" }
                }
            }
            div { class: "keys",
                for (label, class) in KEYS {
                    button {
                        key: "{label}",
                        class: "key {class}",
                        onclick: move |_| press(label),
                        "{label}"
                    }
                }
            }
        }
    }
}
