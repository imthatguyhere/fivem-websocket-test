//! Console command: help/?

use crate::commands::CommandRegistry; //=-- Access registry
use std::sync::{Arc, Mutex}; //=-- Cache guard
use std::time::{Instant, Duration}; //=-- Cache TTL

/// Register the help command with optional fancy styling and a cache TTL
pub fn register(reg: &mut CommandRegistry, fancy: bool, ttl: Duration) { //=--
    //=-- Pre-render help once and place behind a simple TTL cache
    let initial = reg.help_text_with_fancy(fancy);
    let cache = Arc::new(Mutex::new((Instant::now().checked_sub(ttl).unwrap_or_else(Instant::now), initial)));
    let cache_cl = cache.clone();
    reg.register(&["help", "?"], "Show this help message", move |_ctx, _| {
        let mut guard = cache_cl.lock().expect("help cache poisoned");
        let (ref mut last, ref text) = *guard;
        let now = Instant::now();
        if now.duration_since(*last) >= ttl {
            //=-- Commands are static; refresh timestamp only to honor TTL semantics
            *last = now;
        }
        tracing::info!("\n{}", text);
    });
}
