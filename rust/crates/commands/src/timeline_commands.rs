use crate::{CommandResult, CommandTrait};
use timeline::TimelineElement;
use time::MediaTime;

/// Insert an element into a specific track.
pub struct InsertElementCommand {
    scene_id: String,
    track_id: String,
    element: TimelineElement,
    was_inserted: bool,
}

impl InsertElementCommand {
    pub fn new(scene_id: String, track_id: String, element: TimelineElement) -> Self {
        Self {
            scene_id,
            track_id,
            element,
            was_inserted: false,
        }
    }
}

impl CommandTrait for InsertElementCommand {
    fn name(&self) -> &str {
        "Insert Element"
    }

    fn execute(&mut self) -> CommandResult {
        // This is used by the command stack wrapper
        CommandResult::default()
    }

    fn undo(&mut self) {
        self.was_inserted = false;
    }

    fn redo(&mut self) -> CommandResult {
        self.was_inserted = true;
        CommandResult::default()
    }

    fn box_clone(&self) -> Box<dyn CommandTrait> {
        Box::new(Self {
            scene_id: self.scene_id.clone(),
            track_id: self.track_id.clone(),
            element: self.element.clone(),
            was_inserted: self.was_inserted,
        })
    }
}

/// Delete elements by ID.
pub struct DeleteElementsCommand {
    scene_id: String,
    element_ids: Vec<String>,
    removed_elements: Vec<TimelineElement>,
}

impl DeleteElementsCommand {
    pub fn new(scene_id: String, element_ids: Vec<String>) -> Self {
        Self {
            scene_id,
            element_ids,
            removed_elements: Vec::new(),
        }
    }
}

impl CommandTrait for DeleteElementsCommand {
    fn name(&self) -> &str {
        "Delete Elements"
    }

    fn execute(&mut self) -> CommandResult {
        CommandResult::default()
    }

    fn undo(&mut self) {
        self.removed_elements.clear();
    }

    fn redo(&mut self) -> CommandResult {
        CommandResult::default()
    }

    fn box_clone(&self) -> Box<dyn CommandTrait> {
        Box::new(Self {
            scene_id: self.scene_id.clone(),
            element_ids: self.element_ids.clone(),
            removed_elements: self.removed_elements.clone(),
        })
    }
}

/// Move elements to new positions.
pub struct MoveElementCommand {
    scene_id: String,
    element_id: String,
    old_start_time: MediaTime,
    new_start_time: MediaTime,
    old_track_id: String,
    new_track_id: Option<String>,
}

impl MoveElementCommand {
    pub fn new(
        scene_id: String,
        element_id: String,
        old_start_time: MediaTime,
        new_start_time: MediaTime,
        old_track_id: String,
        new_track_id: Option<String>,
    ) -> Self {
        Self {
            scene_id,
            element_id,
            old_start_time,
            new_start_time,
            old_track_id,
            new_track_id,
        }
    }
}

impl CommandTrait for MoveElementCommand {
    fn name(&self) -> &str {
        "Move Element"
    }

    fn execute(&mut self) -> CommandResult {
        CommandResult::default()
    }

    fn undo(&mut self) {}

    fn redo(&mut self) -> CommandResult {
        CommandResult::default()
    }

    fn box_clone(&self) -> Box<dyn CommandTrait> {
        Box::new(Self {
            scene_id: self.scene_id.clone(),
            element_id: self.element_id.clone(),
            old_start_time: self.old_start_time,
            new_start_time: self.new_start_time,
            old_track_id: self.old_track_id.clone(),
            new_track_id: self.new_track_id.clone(),
        })
    }
}

/// Split an element into two at a given time.
pub struct SplitElementCommand {
    scene_id: String,
    track_id: String,
    element_id: String,
    split_time: MediaTime,
    new_element: Option<TimelineElement>,
}

impl SplitElementCommand {
    pub fn new(scene_id: String, track_id: String, element_id: String, split_time: MediaTime) -> Self {
        Self {
            scene_id,
            track_id,
            element_id,
            split_time,
            new_element: None,
        }
    }
}

impl CommandTrait for SplitElementCommand {
    fn name(&self) -> &str {
        "Split Element"
    }

    fn execute(&mut self) -> CommandResult {
        CommandResult::default()
    }

    fn undo(&mut self) {
        self.new_element = None;
    }

    fn redo(&mut self) -> CommandResult {
        CommandResult::default()
    }

    fn box_clone(&self) -> Box<dyn CommandTrait> {
        Box::new(Self {
            scene_id: self.scene_id.clone(),
            track_id: self.track_id.clone(),
            element_id: self.element_id.clone(),
            split_time: self.split_time,
            new_element: self.new_element.clone(),
        })
    }
}
