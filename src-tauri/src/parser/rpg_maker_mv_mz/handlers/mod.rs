//! Command handlers for RPG Maker MV/MZ
//!
//! Each handler is responsible for extracting and injecting translations
//! for specific event command types.

pub mod dialogue;
pub mod choices;
pub mod comment;
pub mod plugin;

use crate::parser::types::{
    EventCode, ExtractionContext, ExtractionOptions, ExtractionResult, InjectionOptions,
    InjectionResult, TranslationPath,
};
use super::command::EventCommand;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for handling specific event commands
pub trait CommandHandler: Send + Sync {
    /// Get the event codes this handler can process
    fn handles(&self) -> Vec<EventCode>;

    /// Extract translation units from command(s)
    ///
    /// # Arguments
    /// * `commands` - Full list of commands in the page
    /// * `index` - Current index in the command list
    /// * `path_prefix` - Path prefix for the current location
    /// * `context` - Extraction context (speaker, preceding lines, etc.)
    /// * `options` - Extraction options
    ///
    /// # Returns
    /// ExtractionResult containing extracted units and consumed count
    fn extract(
        &self,
        commands: &[EventCommand],
        index: usize,
        path_prefix: &TranslationPath,
        context: &mut ExtractionContext,
        options: &ExtractionOptions,
    ) -> ExtractionResult;

    /// Inject translations back into commands
    ///
    /// # Arguments
    /// * `commands` - Mutable list of commands to modify
    /// * `index` - Current index in the command list
    /// * `translations` - Map of unit IDs to translated text
    /// * `path_prefix` - Path prefix for the current location
    /// * `context` - Extraction context
    /// * `options` - Injection options
    ///
    /// # Returns
    /// Number of commands after modification (may differ from consumed)
    fn inject(
        &self,
        commands: &mut Vec<EventCommand>,
        index: usize,
        translations: &HashMap<String, String>,
        path_prefix: &TranslationPath,
        context: &ExtractionContext,
        options: &InjectionOptions,
    ) -> InjectionResult;
}

/// Registry of command handlers
pub struct HandlerRegistry {
    handlers: HashMap<EventCode, Arc<dyn CommandHandler>>,
}

impl HandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Create a registry with all default handlers
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        
        // Register dialogue handlers (101, 401)
        registry.register_handler(Arc::new(dialogue::ShowTextHandler));
        registry.register_handler(Arc::new(dialogue::DialogueHandler));
        
        // Register choice handlers (102, 402)
        registry.register_handler(Arc::new(choices::ChoicesHandler));
        registry.register_handler(Arc::new(choices::ChoiceBranchHandler));
        
        // Register comment handler (408)
        registry.register_handler(Arc::new(comment::CommentHandler));
        
        // Register plugin handler (357)
        registry.register_handler(Arc::new(plugin::PluginCommandHandler::new()));
        
        registry
    }

    /// Register a handler for all its supported codes
    pub fn register_handler(&mut self, handler: Arc<dyn CommandHandler>) {
        let codes = handler.handles();
        for code in codes {
            self.handlers.insert(code, Arc::clone(&handler));
        }
    }

    /// Get a handler for a specific event code
    pub fn get(&self, code: EventCode) -> Option<&dyn CommandHandler> {
        self.handlers.get(&code).map(|h| h.as_ref())
    }

    /// Check if there's a handler for a specific code
    pub fn has_handler(&self, code: EventCode) -> bool {
        self.handlers.contains_key(&code)
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Helper function to generate a unit ID
pub fn generate_unit_id(
    path: &TranslationPath,
    index: usize,
    suffix: &str,
) -> String {
    let base = path.append_index(index);
    base.to_unit_id(suffix)
}

pub use dialogue::{DialogueHandler, ShowTextHandler};
pub use choices::{ChoicesHandler, ChoiceBranchHandler};
pub use comment::CommentHandler;
pub use plugin::PluginCommandHandler;
