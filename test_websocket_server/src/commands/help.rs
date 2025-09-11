//! Console command: help/?

use crate::commands::CommandRegistry; //=-- Access registry
use std::sync::{Arc, Mutex}; //=-- Cache guard
use std::time::{Instant, Duration}; //=-- Cache TTL

/// Register the help command with optional fancy styling and a cache TTL
pub fn register(reg: &mut CommandRegistry, fancy: bool, ttl: Duration) { //=--
    //=-- Seed cache with current help text; allow refresh via ctx.help_supplier when TTL expires
    let initial = reg.help_text_with_fancy(fancy);
    let cache: Arc<Mutex<(Instant, String)>> = Arc::new(Mutex::new((Instant::now(), initial)));
    let cache_cl = cache.clone();
    reg.register(&["help", "?"], "Show this help message", move |ctx, _| {
        let mut guard = cache_cl.lock().expect("help cache poisoned");
        let now = Instant::now();
        if now.duration_since(guard.0) >= ttl {
            //=-- TTL expired: regenerate help text using supplier from context
            let fresh = (ctx.help_supplier)(fancy);
            guard.0 = now;
            guard.1 = fresh;
        }
        tracing::info!("\n{}", guard.1);
    });
}
