//! Command registry and handler for console stdin

//=-- Individual command modules live under src/commands/
pub mod send_heartbeat; //=-- heartbeat, hb
pub mod disconnect;     //=-- disconnect, dc
pub mod quit;           //=-- quit, exit, q
pub mod show_config;    //=-- config, cfg
pub mod help;       //=-- help, ?
pub mod dynamic_payload; //=-- commands.toml-backed payload commands

use std::sync::Arc; //=--
use tokio::sync::broadcast; //=--
use axum::extract::ws::Utf8Bytes; //=-- For broadcasting text frames
use crate::state::ControlCommand; //=--

/// Execution context passed to command handlers
pub struct CommandContext { //=--
    pub tx: broadcast::Sender<Utf8Bytes>, //=-- JSON broadcast channel
    pub ctrl_tx: broadcast::Sender<ControlCommand>, //=-- Control channel
    pub shutdown: tokio_util::sync::CancellationToken, //=-- Shutdown token
    pub help_supplier: Arc<dyn Fn(bool) -> String + Send + Sync + 'static>, //=-- Function to render help text on demand
}

type CommandHandler = Arc<dyn Fn(&CommandContext, &str) + Send + Sync + 'static>; //=--

struct Command { //=--
    keywords: Vec<String>, //=-- canonical first, aliases after
    description: String, //=-- human description
    handler: CommandHandler, //=-- behavior
}

/// Registry for console commands
pub struct CommandRegistry { //=--
    commands: Vec<Command>, //=-- ordered list
}

impl CommandRegistry { //=--
    pub fn new() -> Self { //=--
        Self { commands: Vec::new() }
    }

    /// Register a command with keywords and description
    pub fn register<H>(&mut self, keywords: &[&str], description: &str, handler: H) //=--
    where
        H: Fn(&CommandContext, &str) + Send + Sync + 'static,
    {
        let kws = keywords.iter().map(|s| s.to_ascii_lowercase()).collect::<Vec<_>>();
        self.commands.push(Command {
            keywords: kws,
            description: description.to_string(),
            handler: Arc::new(handler),
        });
    }

    /// Attempt to parse and execute a command. Returns true if consumed.
    pub fn parse_and_execute(&self, input: &str, ctx: &CommandContext) -> bool { //=--
        let needle = input.trim().to_ascii_lowercase();
        if needle.is_empty() { return false; }
        //=-- Search from the end so later-registered commands take precedence (useful for reloads)
        if let Some(cmd) = self.commands.iter().rev().find(|c| c.keywords.iter().any(|k| k == &needle)) {
            (cmd.handler)(ctx, input);
            true
        } else {
            false
        }
    }

    /// Return the list of distinct primary command names, with latest registration winning //=--
    pub fn primary_names_distinct(&self) -> Vec<String> { //=--
        use std::collections::HashSet; //=--
        let mut seen = HashSet::new();
        let mut out: Vec<String> = Vec::new();
        //=-- Walk from the end so later registrations win, then reverse to keep natural order
        for cmd in self.commands.iter().rev() {
            if let Some(first) = cmd.keywords.first() {
                if seen.insert(first) {
                    out.push(first.clone());
                }
            }
        }
        out.reverse();
        out
    }

    /// Build a help text with optional ANSI styling
    pub fn help_text_with_fancy(&self, fancy: bool) -> String { //=--
        //=-- Styles helper; when fancy=false, all styles become empty strings
        #[allow(dead_code)] //=-- Localized allow: some style fields may be unused depending on formatting
        struct Styles<'a> { 
            reset: &'a str,
            bold: &'a str,
            dim: &'a str,
            yellow: &'a str,
            cyan: &'a str,
            magenta: &'a str,
            border_h: &'a str,
            border_v: &'a str,
            border_tl: &'a str,
            border_tr: &'a str,
            border_bl: &'a str,
            border_br: &'a str,
            bullet: &'a str,
        }

        let s = if fancy {
            Styles {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                dim: "\x1b[2m",
                yellow: "\x1b[33m",
                cyan: "\x1b[36m",
                magenta: "\x1b[35m",
                border_h: "─",
                border_v: "│",
                border_tl: "┌",
                border_tr: "┐",
                border_bl: "└",
                border_br: "┘",
                bullet: "•",
            }
        } else {
            Styles {
                reset: "",
                bold: "",
                dim: "",
                yellow: "",
                cyan: "",
                magenta: "",
                border_h: "-",
                border_v: "|",
                border_tl: "+",
                border_tr: "+",
                border_bl: "+",
                border_br: "+",
                bullet: "-",
            }
        };

        //=-- Compose output
        let mut out = String::new();
        //=-- Header box
        let title = " Available commands ";
        let bar = s.border_h.repeat(32);
        out.push_str(&format!(
            "{top_left}{bar}{top_right}\n{side} {bold}{yellow}{title}{reset} {side}\n{bottom_left}{bar}{bottom_right}\n",
            top_left = s.border_tl,
            bar = bar,
            top_right = s.border_tr,
            side = s.border_v,
            bold = s.bold,
            yellow = s.yellow,
            title = title,
            reset = s.reset,
            bottom_left = s.border_bl,
            bottom_right = s.border_br,
        ));
        out.push_str(&format!("{dim}Type a command, or a one-line JSON payload, then press Enter{reset}\n\n",
            dim = s.dim, reset = s.reset));

        //=-- Commands list
        for cmd in &self.commands {
            if let Some((first, rest)) = cmd.keywords.split_first() {
                let alias_part = if rest.is_empty() {
                    String::new()
                } else {
                    let alias_list = rest.join(", ");
                    format!(" {dim}(aliases: {cyan}{alias_list}{reset}{dim}){reset}",
                        dim = s.dim, cyan = s.cyan, alias_list = alias_list, reset = s.reset)
                };
                out.push_str(&format!(
                    "  {bullet} {bold}{cyan}{first}{reset}{alias_part}\n     {dim}— {desc}{reset}\n",
                    bullet = s.bullet,
                    bold = s.bold,
                    cyan = s.cyan,
                    first = first,
                    reset = s.reset,
                    alias_part = alias_part,
                    dim = s.dim,
                    desc = cmd.description,
                ));
            }
        }

        out
    }
}
