use std::collections::HashMap;
use std::sync::Arc;

use crate::tools::DynTool;
use crate::{Error, Result};

#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: Arc<HashMap<String, DynTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn get_tool(&self, name: &str) -> Option<DynTool> {
        self.tools.get(name).cloned()
    }
}
