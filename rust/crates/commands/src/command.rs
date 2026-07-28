use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub selection: Option<serde_json::Value>,
}

pub trait CommandTrait: Send {
    fn name(&self) -> &str;
    fn execute(&mut self) -> CommandResult;
    fn undo(&mut self);
    fn redo(&mut self) -> CommandResult;
    fn merge(&mut self, _other: &dyn CommandTrait) -> bool {
        false
    }
    fn box_clone(&self) -> Box<dyn CommandTrait>;
}

impl Clone for Box<dyn CommandTrait> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone)]
pub struct Command {
    inner: Box<dyn CommandTrait>,
}

impl Command {
    pub fn new(inner: Box<dyn CommandTrait>) -> Self {
        Self { inner }
    }

    pub fn name(&self) -> &str {
        self.inner.name()
    }

    pub fn execute(&mut self) -> CommandResult {
        self.inner.execute()
    }

    pub fn undo(&mut self) {
        self.inner.undo();
    }

    pub fn redo(&mut self) -> CommandResult {
        self.inner.redo()
    }

    pub fn merge(&mut self, other: &Command) -> bool {
        self.inner.merge(other.inner.as_ref())
    }
}
