//! Dynamic payload commands loaded from commands.toml

use crate::commands::CommandRegistry;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::collections::HashSet; //=-- Duplicate detection

#[derive(Debug, Deserialize)]
struct CommandsFile {
    #[serde(default)]
    payload_commands: Vec<PayloadCommand>, //=-- Array of payload commands
}

#[derive(Debug, Deserialize, Clone)]
struct PayloadCommand {
    name: String, //=-- Primary command name
    #[serde(default)]
    aliases: Vec<String>, //=-- Optional aliases
    #[serde(default)]
    #[serde(rename = "type")]
    type_: Option<String>, //=-- Shorthand payload: JSON "type" (mapped from TOML key "type")
    #[serde(default)]
    data: Option<serde_json::Value>, //=-- Shorthand payload: JSON "data"
    #[serde(default)]
    payload: Option<serde_json::Value>, //=-- Full payload override (object/value)
    #[serde(default)]
    description: Option<String>, //=-- Optional custom description
}

/// Load commands from a TOML file path and register them with the command registry.
pub fn register_from_file(reg: &mut CommandRegistry, path: &str) {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("No commands file at '{}': {}", path, e); //=-- Not fatal
            return;
        }
    };

    let parsed: CommandsFile = match toml::from_str(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::warn!("⚠️ Failed to parse commands file '{}': {}", path, e);
            return;
        }
    };

    //=-- Existing primary names from the registry to warn on overrides
    let existing: HashSet<String> = reg.primary_names_distinct().into_iter().collect();

    //=-- Track duplicate keywords (names and aliases) within the file and list of dynamic names for logging/command
    let mut seen_keywords: HashSet<String> = HashSet::new();
    let mut dynamic_names: Vec<String> = Vec::new();

    for cmd in parsed.payload_commands.into_iter() {
        let mut keys: Vec<String> = Vec::with_capacity(1 + cmd.aliases.len());
        keys.push(cmd.name.clone());
        keys.extend(cmd.aliases.clone());
        let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

        //=-- Check duplicate keywords within the file (names and aliases must be unique)
        for keyword in &keys {
            if !seen_keywords.insert(keyword.clone()) {
                tracing::warn!("⚠️ Duplicate command keyword in commands.toml: '{}' (later one wins)", keyword);
            }
        }

        //=-- Warn if overriding an existing primary command name in the registry
        if existing.contains(&cmd.name) {
            tracing::warn!("⚠️ Dynamic command '{}' overrides an existing command", cmd.name);
        }

        let desc = cmd.description.clone().unwrap_or_else(|| {
            if let Some(t) = &cmd.type_ { format!("Send predefined payload: {}", t) } else { "Send predefined payload".to_string() }
        });

        //=-- Build the payload closure now; owned clone moved into closure
        let builder = move || -> String {
            if let Some(pl) = &cmd.payload {
                //=-- If given, use literal payload value
                pl.to_string()
            } else {
                let t = cmd.type_.unwrap_or_else(|| "custom".to_string());
                let d = cmd.data.unwrap_or(serde_json::Value::Null);
                json!({ "type": t, "data": d }).to_string()
            }
        };

        reg.register(&key_refs, &desc, move |ctx, _| {
            let payload = builder();
            let _ = ctx.tx.send(payload.clone().into());
            tracing::info!("📤 Broadcast payload (commands.toml): {}", payload);
        });

        dynamic_names.push(cmd.name);
    }

    //=-- Log what we loaded (boot or reload)
    if dynamic_names.is_empty() {
        tracing::info!("📦 commands.toml: no payload commands loaded");
    } else {
        tracing::info!("📦 commands.toml: loaded payload commands: {}", dynamic_names.join(", "));
    }

    //=-- Add a helper command to list dynamic command names
    let names_for_cmd = dynamic_names.clone();
    reg.register(&["commands", "cmds"], "List dynamically loaded commands", move |_ctx, _| {
        if names_for_cmd.is_empty() {
            tracing::info!("(commands.toml) No dynamic commands loaded");
        } else {
            tracing::info!("(commands.toml) Dynamic commands: {}", names_for_cmd.join(", "));
        }
    });
}
