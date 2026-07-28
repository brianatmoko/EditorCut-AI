use crate::Command;

const MAX_UNDO_DEPTH: usize = 100;

#[derive(Default)]
pub struct CommandStack {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
    saved_at_index: usize,
    current_index: usize,
}

impl CommandStack {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::with_capacity(MAX_UNDO_DEPTH),
            redo_stack: Vec::new(),
            saved_at_index: 0,
            current_index: 0,
        }
    }

    pub fn execute(&mut self, mut command: Command) {
        command.execute();
        self.undo_stack.truncate(self.current_index);
        if self.undo_stack.len() >= MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
            self.saved_at_index = self.saved_at_index.saturating_sub(1);
        } else {
            self.current_index += 1;
        }
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if self.current_index == 0 {
            return false;
        }
        self.current_index -= 1;
        if let Some(command) = self.undo_stack.get_mut(self.current_index) {
            command.undo();
            self.redo_stack.push(command.clone());
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(mut command) = self.redo_stack.pop() {
            command.redo();
            self.undo_stack.push(command);
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn mark_saved(&mut self) {
        self.saved_at_index = self.current_index;
    }

    pub fn is_saved(&self) -> bool {
        self.saved_at_index == self.current_index
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_index = 0;
        self.saved_at_index = 0;
    }
}
