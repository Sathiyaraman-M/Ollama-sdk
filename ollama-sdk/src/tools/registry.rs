use std::collections::HashMap;
use std::sync::Arc;

use crate::tools::DynTool;
use crate::{Error, Result};

/// A registry for managing and accessing [`DynTool`] instances.
///
/// This struct allows for registering, unregistering, and retrieving tools
/// by their unique names. It ensures that tool names are unique.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: Arc<HashMap<String, DynTool>>,
}

impl ToolRegistry {
    /// Creates a new, empty `ToolRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new tool with the registry.
    ///
    /// # Arguments
    ///
    /// * `tool` - The [`DynTool`] instance to register.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Client`] if a tool with the same name is already registered.
    pub fn register_tool(&mut self, tool: DynTool) -> Result<()> {
        let tool_name = tool.name().to_string();
        let tools = Arc::make_mut(&mut self.tools);
        if tools.insert(tool_name.clone(), tool).is_some() {
            return Err(Error::Client(format!(
                "Tool with name '{}' already registered",
                tool_name
            )));
        }
        Ok(())
    }

    /// Unregisters a tool from the registry by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to unregister.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Client`] if no tool with the given name is found.
    pub fn unregister_tool(&mut self, name: &str) -> Result<()> {
        let tools = Arc::make_mut(&mut self.tools);
        if tools.remove(name).is_none() {
            return Err(Error::Client(format!(
                "Tool with name '{}' not found",
                name
            )));
        }
        Ok(())
    }

    /// Retrieves a registered tool by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a cloned [`DynTool`] if found, or `None` otherwise.
    pub fn get_tool(&self, name: &str) -> Option<DynTool> {
        self.tools.get(name).cloned()
    }
}
