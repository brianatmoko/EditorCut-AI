use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::pose::StickmanPose;
use crate::cinematic::{CinematicMovie, StageEntity, CinematicAct};
use crate::poses::get_pose;

// ═══════════════════════════════════════════════════════════════
// EXECUTION STATE TRACKING
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct EntityExecutionState {
    pub pos: (f32, f32, f32),
    pub target_pos: (f32, f32, f32),
    pub movement_start_pos: (f32, f32, f32),
    /// None until the first update tick; set to act_elapsed once movement begins
    pub movement_start_time: Option<f64>,
    pub movement_duration: f64,
    pub current_action: String,
    pub pose: StickmanPose,
    pub facing_left: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionSnapshot {
    pub entity_id: String,
    pub timestamp: f64,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub frame_number: u32,
    pub action: String,
    pub progress_pct: f32,
}

#[derive(Clone, Debug)]
pub struct ExecutionLog {
    pub level: String,  // "INFO", "DEBUG", "WARN", "ERROR"
    pub component: String,  // "EXECUTOR", "MOVEMENT", "VALIDATION"
    pub timestamp: f64,
    pub message: String,
    pub context: HashMap<String, String>,
}

pub struct SceneExecutor {
    pub current_movie: CinematicMovie,
    pub current_act_idx: usize,
    pub movie_start_time: f64,
    /// Per-act init flag — prevents double-initialize when update() called multiple
    /// times for the same act (e.g. after preview.rs calls initialize_act(0) explicitly,
    /// then executor.update(0.0) would re-trigger init without this guard).
    pub acts_initialized: Vec<bool>,
    
    pub entity_states: HashMap<String, EntityExecutionState>,
    pub position_history: Vec<PositionSnapshot>,
    pub logs: Vec<ExecutionLog>,
    
    // Configuration
    pub target_fps: u32,
    pub enable_position_logging: bool,
}

// ═══════════════════════════════════════════════════════════════
// INITIALIZATION & SETUP
// ═══════════════════════════════════════════════════════════════

impl SceneExecutor {
    pub fn new(movie: CinematicMovie) -> Self {
        let n_acts = movie.acts.len();
        let mut executor = Self {
            current_movie: movie,
            current_act_idx: 0,
            movie_start_time: 0.0,
            acts_initialized: vec![false; n_acts],
            entity_states: HashMap::new(),
            position_history: Vec::new(),
            logs: Vec::new(),
            target_fps: 60,
            enable_position_logging: true,
        };
        
        executor.log_info("EXECUTOR_INIT", vec![
            ("total_acts".to_string(), format!("{}", executor.current_movie.acts.len())),
            ("total_duration".to_string(), format!("{:.1}s", executor.current_movie.total_duration)),
            ("title".to_string(), executor.current_movie.title.clone()),
        ]);
        
        executor
    }
    
    pub fn initialize_act(&mut self, act_idx: usize) {
        if act_idx >= self.current_movie.acts.len() {
            self.log_error("INIT_ACT", vec![
                ("reason".to_string(), "act_index_out_of_bounds".to_string()),
                ("requested".to_string(), format!("{}", act_idx)),
                ("total_acts".to_string(), format!("{}", self.current_movie.acts.len())),
            ]);
            return;
        }
        
        // C5 guard: skip if this act was already initialized
        if self.acts_initialized.get(act_idx).copied().unwrap_or(false) {
            return;
        }
        self.acts_initialized[act_idx] = true;
        
        let act = self.current_movie.acts[act_idx].clone();
        
        self.log_info("ACT_INIT", vec![
            ("act_number".to_string(), format!("{}", act.act_number)),
            ("title".to_string(), act.title.clone()),
            ("duration".to_string(), format!("{:.1}s", act.duration)),
            ("num_entities".to_string(), format!("{}", act.entities.len())),
            ("intensity".to_string(), format!("{:.1}", act.intensity)),
            ("theme".to_string(), act.theme.clone()),
        ]);
        
        self.current_act_idx = act_idx;
        
        // Initialize entity states for this act
        for entity in act.entities {
            self.initialize_entity(&entity);
        }
    }
    
    fn initialize_entity(&mut self, entity: &StageEntity) {
        // Carry forward position from previous act if this entity was seen before.
        // The AI director generates entities per-act; we look up by character_skin_id
        // to maintain visual continuity (same character walks from where they ended last act).
        let prev_pos = self.entity_states.get(&entity.id).map(|s| s.pos);
        let start_pos = if let Some(pp) = prev_pos {
            // Use carried-forward X/Y, but keep this act's Z (depth may change for staging)
            (pp.0, pp.1, entity.pos_z)
        } else {
            // Try to find a previous entity with the same skin (cross-act continuity)
            let skin = &entity.character_skin_id;
            let carried = self.entity_states.iter()
                .find(|(_, st)| st.current_action != "idle" || true) // any entity
                .map(|(_, st)| st.pos);
            // Better: look for an entity with the same skin whose ID doesn't match the current act
            let by_skin = self.entity_states.iter()
                .find(|(k, st)| {
                    *k != &entity.id && self.entity_skin(k) == Some(skin.clone())
                })
                .map(|(_, st)| st.pos);
            if let Some(bp) = by_skin {
                (bp.0, bp.1, entity.pos_z)
            } else {
                (entity.pos_x, entity.pos_y, entity.pos_z)
            }
        };
        let target_pos = (
            entity.end_x.unwrap_or(start_pos.0),
            entity.end_y.unwrap_or(start_pos.1),
            entity.pos_z
        );
        
        // Estimate movement duration based on distance
        let distance = (((target_pos.0 - start_pos.0) as f64).powi(2) 
                      + ((target_pos.1 - start_pos.1) as f64).powi(2)).sqrt();
        
        // Base speed: ~0.5 units per second for walk, faster for run
        let base_speed = match entity.action.to_lowercase().as_str() {
            s if s.contains("run") || s.contains("sprint") => 1.2,
            s if s.contains("walk") || s.contains("slow") => 0.4,
            s if s.contains("panic") => 1.5,
            _ => 0.6,
        };
        
        // Clamp movement_duration to the act duration so entities arrive before the act ends.
        // If the natural movement time exceeds the act, the entity will appear to rush,
        // but at least it reaches its target rather than snapping mid-animation.
        let act_dur = self.current_movie.acts.get(self.current_act_idx)
            .map(|a| a.duration).unwrap_or(10.0);
        let movement_duration = if distance > 0.01 {
            ((distance / base_speed) as f64).min(act_dur * 0.9)
        } else {
            0.5  // Minimal movement for same-position actions
        };
        
        self.entity_states.insert(
            entity.id.clone(),
            EntityExecutionState {
                pos: start_pos,
                target_pos,
                movement_start_pos: start_pos,
                movement_start_time: None,
                movement_duration,
                current_action: entity.action.clone(),
                pose: StickmanPose::neutral(),
                facing_left: entity.facing_left,
            }
        );
        
        self.log_debug("ENTITY_INIT", vec![
            ("entity_id".to_string(), entity.id.clone()),
            ("name".to_string(), entity.name.clone()),
            ("start_pos".to_string(), format!("({:.2}, {:.2}, {:.2})", start_pos.0, start_pos.1, start_pos.2)),
            ("target_pos".to_string(), format!("({:.2}, {:.2}, {:.2})", target_pos.0, target_pos.1, target_pos.2)),
            ("movement_duration".to_string(), format!("{:.2}s", movement_duration)),
            ("action".to_string(), entity.action.clone()),
        ]);
    }
}

// ═══════════════════════════════════════════════════════════════
// EXECUTION & MOVEMENT
// ═══════════════════════════════════════════════════════════════

impl SceneExecutor {
    pub fn update(&mut self, elapsed: f64) {
        if self.current_act_idx >= self.current_movie.acts.len() {
            self.log_warn("UPDATE", vec![
                ("reason".to_string(), "scene_complete".to_string()),
                ("total_acts".to_string(), format!("{}", self.current_movie.acts.len())),
            ]);
            return;
        }
        
        let act = &self.current_movie.acts[self.current_act_idx].clone();
        let act_elapsed = elapsed - act.start_time;
        
        if act_elapsed < 0.0 {
            return;  // Not yet at this act
        }
        
        // Initialize the act if it has not been initialized yet (C2 fix: no float-equality sentinel)
        if !self.acts_initialized.get(self.current_act_idx).copied().unwrap_or(false) {
            self.initialize_act(self.current_act_idx);
            if self.movie_start_time == 0.0 {
                self.movie_start_time = elapsed;
            }
        }
        
        self.log_frame_header(elapsed, act_elapsed, &act);
        
        // Execute all entities in current act
        for entity in &act.entities {
            self.execute_entity(entity, act_elapsed);
        }
        
        // Check if act is complete
        if act_elapsed >= act.duration {
            self.log_info("ACT_COMPLETE", vec![
                ("act_number".to_string(), format!("{}", act.act_number)),
                ("title".to_string(), act.title.clone()),
                ("duration".to_string(), format!("{:.2}s", act.duration)),
            ]);
            
            self.current_act_idx += 1;
            if self.current_act_idx < self.current_movie.acts.len() {
                self.initialize_act(self.current_act_idx);
            }
        }
    }
    
    fn execute_entity(&mut self, entity: &StageEntity, act_elapsed: f64) {
        let entity_key = entity.id.clone();
        
        if !self.entity_states.contains_key(&entity_key) {
            self.initialize_entity(entity);
        }

        let (movement_start_time, movement_duration, current_action);
        {
            let state = match self.entity_states.get_mut(&entity_key) {
                Some(s) => s,
                None => return,
            };
            // Set movement_start_time on first tick (Option sentinel — C3 fix)
            if state.movement_start_time.is_none() {
                state.movement_start_time = Some(act_elapsed);
            }
            movement_start_time = state.movement_start_time.unwrap();
            movement_duration = state.movement_duration;
            current_action = state.current_action.clone();
        }

        let elapsed_in_movement = act_elapsed - movement_start_time;
        let progress = if movement_duration > 0.01 {
            (elapsed_in_movement / movement_duration).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let new_pos = self.interpolate_position(
            self.get_entity_movement_start(&entity_key),
            self.get_entity_state_target(&entity_key),
            progress,
        );

        if let Some(state) = self.entity_states.get_mut(&entity_key) {
            state.pos = new_pos;
            // C4: cinematic renderer only consumes pose.pos_x/pos_y (world coords).
            // Skip the expensive get_pose() allocation on every frame; just update coords.
            state.pose.pos_x = new_pos.0 as f64;
            state.pose.pos_y = new_pos.1 as f64;
            state.facing_left = entity.facing_left;
        }

        self.log_entity_movement(&entity.id, &entity.name, new_pos, act_elapsed, progress as f32);

        if self.enable_position_logging {
            self.record_position_snapshot(
                entity.id.clone(),
                new_pos,
                act_elapsed,
                progress as f32,
                &current_action,
            );
        }
    }
    
    fn get_entity_state_pos(&self, key: &str) -> (f32, f32, f32) {
        self.entity_states.get(key).map(|s| s.pos).unwrap_or((0.0, 0.0, 0.0))
    }
    
    fn get_entity_movement_start(&self, key: &str) -> (f32, f32, f32) {
        self.entity_states.get(key).map(|s| s.movement_start_pos).unwrap_or((0.0, 0.0, 0.0))
    }
    
    fn get_entity_state_target(&self, key: &str) -> (f32, f32, f32) {
        self.entity_states.get(key).map(|s| s.target_pos).unwrap_or((0.0, 0.0, 0.0))
    }
    
    fn interpolate_position(&self, start: (f32, f32, f32), target: (f32, f32, f32), t: f64) -> (f32, f32, f32) {
        // Smooth easing function
        let t = self.ease_in_out_cubic(t);
        
        (
            (start.0 as f64 + (target.0 as f64 - start.0 as f64) * t) as f32,
            (start.1 as f64 + (target.1 as f64 - start.1 as f64) * t) as f32,
            start.2,  // Z stays constant
        )
    }
    
    fn ease_in_out_cubic(&self, t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// LOGGING & DIAGNOSTICS
// ═══════════════════════════════════════════════════════════════

impl SceneExecutor {
    fn log_frame_header(&self, _elapsed: f64, _act_elapsed: f64, _act: &CinematicAct) {
        // Throttled: only log every ~60 frames to avoid stderr spam
        if (self.position_history.len() as u64) % 60 != 0 {
            return;
        }
        eprintln!("[SceneExecutor] Act {} | t={:.1}s | ent={}",
            _act.act_number, _act_elapsed, _act.entities.len());
    }
    
    fn log_entity_movement(&self, _entity_id: &str, _entity_name: &str, 
                          _pos: (f32, f32, f32), _time: f64, _progress: f32) {
        // Disabled per-frame entity logging (causes massive stderr spam at 60fps)
    }
    
    fn record_position_snapshot(&mut self, entity_id: String, pos: (f32, f32, f32), 
                               elapsed: f64, progress: f32, action: &str) {
        self.position_history.push(PositionSnapshot {
            entity_id,
            timestamp: elapsed,
            pos_x: pos.0,
            pos_y: pos.1,
            pos_z: pos.2,
            frame_number: (elapsed * 60.0) as u32,
            action: action.to_string(),
            progress_pct: progress * 100.0,
        });
    }
    
    fn log_info(&mut self, component: &str, context: Vec<(String, String)>) {
        let mut ctx = HashMap::new();
        for (k, v) in context {
            ctx.insert(k, v);
        }
        
        let msg = ctx.iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(" | ");
        
        eprintln!("[INFO] [{}] {}", component, msg);
        
        self.logs.push(ExecutionLog {
            level: "INFO".to_string(),
            component: component.to_string(),
            timestamp: 0.0,
            message: msg,
            context: ctx,
        });
    }
    
    fn log_debug(&mut self, component: &str, context: Vec<(String, String)>) {
        let mut ctx = HashMap::new();
        for (k, v) in context {
            ctx.insert(k, v);
        }
        
        let msg = ctx.iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(" | ");
        
        eprintln!("[DEBUG] [{}] {}", component, msg);
    }
    
    fn log_warn(&mut self, component: &str, context: Vec<(String, String)>) {
        let mut ctx = HashMap::new();
        for (k, v) in context {
            ctx.insert(k, v);
        }
        
        let msg = ctx.iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(" | ");
        
        eprintln!("⚠️  [WARN] [{}] {}", component, msg);
    }
    
    fn log_error(&mut self, component: &str, context: Vec<(String, String)>) {
        let mut ctx = HashMap::new();
        for (k, v) in context {
            ctx.insert(k, v);
        }
        
        let msg = ctx.iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(" | ");
        
        eprintln!("❌ [ERROR] [{}] {}", component, msg);
    }
    
    pub fn get_position_history(&self) -> &Vec<PositionSnapshot> {
        &self.position_history
    }
    
    pub fn get_entity_state(&self, entity_id: &str) -> Option<&EntityExecutionState> {
        self.entity_states.get(entity_id)
    }
    
    pub fn get_current_act(&self) -> Option<&CinematicAct> {
        self.current_movie.acts.get(self.current_act_idx)
    }
    
    pub fn is_complete(&self) -> bool {
        self.current_act_idx >= self.current_movie.acts.len()
    }

    /// Look up the character_skin_id for a stored entity state key by scanning acts.
    fn entity_skin(&self, key: &str) -> Option<String> {
        for act in &self.current_movie.acts {
            if let Some(e) = act.entities.iter().find(|e| e.id == key) {
                return Some(e.character_skin_id.clone());
            }
        }
        None
    }
}

// ═══════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════

impl SceneExecutor {
    /// Get current pose for an entity
    pub fn get_entity_pose(&self, entity_id: &str) -> Option<StickmanPose> {
        self.entity_states.get(entity_id).map(|s| s.pose.clone())
    }
    
    /// Export position history for analysis
    pub fn export_position_log(&self) -> String {
        let mut output = String::new();
        output.push_str("ENTITY_ID,TIMESTAMP,POS_X,POS_Y,POS_Z,FRAME,ACTION,PROGRESS\n");
        
        for snapshot in &self.position_history {
            output.push_str(&format!(
                "{},{:.3},{:.4},{:.4},{:.4},{},{},{:.1}\n",
                snapshot.entity_id,
                snapshot.timestamp,
                snapshot.pos_x,
                snapshot.pos_y,
                snapshot.pos_z,
                snapshot.frame_number,
                snapshot.action,
                snapshot.progress_pct
            ));
        }
        
        output
    }
}
