# 🤖 AI ORCHESTRATION ARCHITECTURE - Detailed Technical Specification

**Problem Statement:**
Current system has hardcoded animation loops with NO AI intelligence. Character movements are just repeated poses. Need intelligent AI-driven scene generation with proper logging, token management, and incremental completion.

---

## 📊 SYSTEM OVERVIEW

```
┌─────────────────────────────────────────────────────────────┐
│                    USER INPUT (Prompt)                      │
│         "Polisi mengejar pencuri di kota"                   │
└────────────┬────────────────────────────────────────────────┘
             │
      ┌──────▼──────┐
      │ AI DIRECTOR │  (Gemini 2.5 Flash)
      │   ENGINE    │  • Scene planning
      │             │  • Character choreography
      └──────┬──────┘  • Narrative structure
             │
      ┌──────▼────────────────────┐
      │ SCRIPT EXECUTOR           │  (Rust)
      │ • Token-aware generation  │  • Incremental completion
      │ • Comprehensive logging   │  • Position tracking
      │ • Error recovery          │
      └──────┬────────────────────┘
             │
      ┌──────▼──────────────────────┐
      │ ANIMATION RENDERER          │  (GPUI/Web)
      │ • Character positioning     │  • Movement execution
      │ • Smooth interpolation      │  • Visual feedback
      │ • Real-time playback        │
      └─────────────────────────────┘
```

---

## 🎬 DETAILED FLOW

### STAGE 1: AI SCENE GENERATION

**AI Directive (Gemini Call):**

```python
SYSTEM_PROMPT = """
You are an intelligent film director AI. Your job:
1. Parse the user's prompt for story intent
2. Generate COMPLETE scene choreography
3. Specify EXACT character positions and movements
4. Include timing, intensity, and emotional beats

Output: Structured JSON with complete scene breakdown.
Be specific about positions (x,y,z coordinates).
"""

USER_PROMPT = """
Generate a film scene: "Polisi mengejar pencuri di kota"

Return JSON with acts structure:
{
  "title": "...",
  "total_duration": 120,
  "acts": [
    {
      "act_number": 1,
      "title": "...",
      "duration": 30,
      "entities": [
        {
          "id": "thief",
          "name": "Pencuri",
          "pos_x": 0.8,  // -1.0 to 1.0 stage coords
          "pos_y": 0.0,
          "pos_z": 1.0,  // depth/layer
          "action": "slow_walk",
          "end_x": 0.5,  // where they move to
          "end_y": 0.0,
          "movement_time": 15,  // seconds to reach end position
          "facing_left": true
        },
        ...
      ]
    }
  ]
}
"""
```

**Token Management Strategy:**

```
Input Budget (user prompt):     ~500 tokens
Scene context/theory:           ~2000 tokens
Character definitions:          ~1000 tokens
Available for generation:       ~4000 tokens (from 8192 max)

Generation Strategy:
- First call: Generate acts 1-2 (most important narrative)
- If incomplete: Continuation call for acts 3+
- If tokens exceeded: Break into scenes
```

### STAGE 2: INCREMENTAL COMPLETION

**Problem:** AI might not generate complete scene on first try (token limits).

**Solution:**

```rust
struct SceneGenerationState {
    // Incremental generation tracking
    completed_acts: usize,
    total_acts_needed: usize,
    position_snapshot: Vec<CharacterPos>,
    last_ai_response: CinematicMovie,
    token_used: usize,
    generation_attempts: usize,
    
    // Logging
    logs: Vec<GenerationLog>,
}

enum GenerationPhase {
    Setup,           // Initial AI call for scene
    Incomplete,      // Need to continue
    Continuing,      // Second AI call to complete
    PositionCheck,   // Verify all characters positioned
    Ready,           // Scene ready to render
}

// Pseudo-code for incremental generation
fn generate_scene_incremental(prompt: &str) -> CinematicMovie {
    let mut state = SceneGenerationState::new();
    
    // Phase 1: Initial generation
    log("PHASE_1_START", &prompt);
    let initial_movie = call_gemini_director(&prompt);
    state.last_ai_response = initial_movie.clone();
    state.token_used = initial_movie.estimate_tokens();
    
    log("PHASE_1_DONE", &format!("Generated {} acts, tokens: {}", 
        initial_movie.acts.len(), state.token_used));
    
    // Phase 2: Check completeness
    if is_scene_complete(&initial_movie) {
        log("SCENE_COMPLETE", "No further generation needed");
        return initial_movie;
    }
    
    // Phase 3: Generate continuation
    log("PHASE_2_START", "Generating missing acts...");
    let continuation_prompt = format!(
        "Continue the scene. Previous acts:\n{}\n\nGenerate acts {} and beyond.",
        summarize_acts(&initial_movie),
        initial_movie.acts.len() + 1
    );
    
    let continuation = call_gemini_director_continuation(&continuation_prompt);
    state.last_ai_response = merge_movies(initial_movie, continuation);
    
    log("PHASE_2_DONE", &format!("Scene now has {} acts",
        state.last_ai_response.acts.len()));
    
    // Phase 4: Validate all characters have end positions
    validate_positions(&state.last_ai_response);
    
    state.last_ai_response
}
```

### STAGE 3: CHARACTER EXECUTION ENGINE

**Critical: Actual Movement Implementation**

```rust
struct CharacterExecutor {
    // Current execution state
    current_act: usize,
    current_entity_id: String,
    
    // Movement tracking
    start_pos: (f32, f32, f32),
    target_pos: (f32, f32, f32),
    movement_progress: f32,  // 0.0 to 1.0
    
    // Logging
    position_log: Vec<PositionSnapshot>,
}

impl CharacterExecutor {
    fn execute_act(&mut self, act: &CinematicAct, elapsed: f64) {
        for entity in &act.entities {
            self.execute_entity(entity, elapsed);
        }
    }
    
    fn execute_entity(&mut self, entity: &StageEntity, elapsed: f64) {
        let start_pos = (entity.pos_x, entity.pos_y, entity.pos_z);
        let target_pos = (
            entity.end_x.unwrap_or(entity.pos_x),
            entity.end_y.unwrap_or(entity.pos_y),
            entity.pos_z
        );
        
        // Calculate movement timeline
        let total_duration = self.calculate_movement_duration(entity);
        let progress = (elapsed / total_duration).clamp(0.0, 1.0);
        
        // Smooth interpolation
        let current_pos = self.interpolate_position(
            start_pos,
            target_pos,
            progress
        );
        
        // Log position for debugging
        self.log_position(&entity.id, current_pos, elapsed);
        
        // Update character pose with action
        let pose = get_pose_from_action(&entity.action, progress);
        
        // CRITICAL: Apply position smoothing from animator
        apply_smooth_movement(&entity.id, current_pos, pose);
    }
    
    fn log_position(&mut self, entity_id: &str, pos: (f32, f32, f32), time: f64) {
        self.position_log.push(PositionSnapshot {
            entity_id: entity_id.to_string(),
            timestamp: time,
            pos_x: pos.0,
            pos_y: pos.1,
            pos_z: pos.2,
            frame_number: (time * 60.0) as u32,  // Assume 60 FPS
        });
        
        eprintln!("[EXEC] t={:.2}s entity={} pos=({:.2}, {:.2}, {:.2})",
            time, entity_id, pos.0, pos.1, pos.2);
    }
}
```

---

## 📝 COMPREHENSIVE LOGGING SYSTEM

**Log Levels:**

```rust
enum LogLevel {
    TRACE,    // Frame-by-frame position
    DEBUG,    // Detailed execution
    INFO,     // Key milestones
    WARN,     // Potential issues
    ERROR,    // Failures
}

struct LogEntry {
    timestamp: f64,
    level: LogLevel,
    component: String,    // "AI_DIRECTOR", "EXECUTOR", "RENDERER"
    message: String,
    context: Map<String, String>,
}
```

**Key Log Points:**

```
[AI_DIRECTOR] PHASE_1_START
  prompt: "Polisi mengejar pencuri..."
  max_tokens: 8192
  temperature: 1.35

[AI_DIRECTOR] PHASE_1_GEMINI_CALL
  timestamp: 2026-07-26T03:59:34.483
  system_prompt_tokens: ~2000
  user_prompt_tokens: ~800
  
[AI_DIRECTOR] PHASE_1_RESPONSE
  total_tokens_used: 4521
  acts_generated: 2
  entities_total: 8
  estimated_duration: 120.0s

[EXECUTOR] ACT_1_START
  act_title: "Babak I: Pendeteksian Pencuri"
  start_time: 0.0s
  duration: 30.0s
  entities: 3

[EXECUTOR] ENTITY_MOVEMENT_START
  entity_id: "thief"
  entity_name: "Pencuri"
  start_pos: (0.8, 0.0, 1.0)
  target_pos: (0.5, 0.0, 1.0)
  movement_time: 15.0s
  action: "slow_walk"
  
[EXECUTOR] ENTITY_POSITION_UPDATE
  entity_id: "thief"
  time: 7.5s (50% through)
  current_pos: (0.65, 0.0, 1.0)
  expected_pos: (0.65, 0.0, 1.0)
  action_frame: "walk_frame_2"

[EXECUTOR] ENTITY_POSITION_REACHED
  entity_id: "thief"
  final_pos: (0.5, 0.0, 1.0)
  actual_final_pos: (0.50, 0.0, 1.0)
  deviation: 0.0
  time_elapsed: 15.0s

[EXECUTOR] ACT_1_COMPLETE
  total_time: 30.0s
  all_entities_positioned: true
  movement_deviations: []

[EXECUTOR] PHASE_2_START
  reason: "Scene incomplete, continuing..."
  last_act: 2
  next_acts_needed: 2
```

---

## 🔄 AI TOKEN FLOW DIAGRAM

```
Scene Prompt Input
       │
       ├─ Token Count Estimation
       │  • User prompt: ~500
       │  • System instruction: ~2000
       │  • Character context: ~1000
       │  ├─ Available for generation: ~4500
       │  └─ Safety buffer: -500
       │     → Budget: ~4000 tokens
       │
       └─ AI Generation Attempt 1
          │
          ├─ Generated ~3500 tokens
          │  Acts 1-2 complete
          │  Acts 3-4 incomplete
          │
          └─ Decision: Scene Incomplete?
             │
             ├─ YES → Attempt 2 (Continuation)
             │  │
             │  ├─ Previous summary: ~800 tokens
             │  │  (summarize existing acts concisely)
             │  │
             │  ├─ Continuation prompt: ~200 tokens
             │  │  "Generate acts 3-4..."
             │  │
             │  └─ Generate remaining: ~2000 tokens
             │     Acts 3-4 complete
             │     Total movie now complete
             │
             └─ NO → Scene Complete
                └─ Validate & Return

Total Token Usage (worst case): ~6500 / 8192 (79%)
```

---

## 🎯 SPECIFIC IMPLEMENTATION TASKS

### Task 1: AI Director Enhancement (Python)

**File:** `apps/desktop/gemini_director.py`

```python
class AIDirector:
    def __init__(self, max_tokens=8192):
        self.max_tokens = max_tokens
        self.logs = []
        
    def generate_scene(self, prompt):
        # Phase 1: Initial generation
        self.log("PHASE_1_START", {"prompt": prompt[:100]})
        
        initial_response = self._call_gemini_with_logging(
            prompt=self._build_initial_prompt(prompt),
            max_tokens=4500
        )
        
        movie = self._parse_response(initial_response)
        self.log("PHASE_1_DONE", {
            "acts": len(movie['acts']),
            "entities": sum(len(a['entities']) for a in movie['acts']),
            "tokens_used": initial_response['tokens']
        })
        
        # Phase 2: Check completeness
        if not self._is_scene_complete(movie):
            self.log("SCENE_INCOMPLETE", {
                "completed_acts": len(movie['acts']),
                "attempting_continuation": True
            })
            
            continuation = self._call_gemini_continuation(movie)
            movie = self._merge_movies(movie, continuation)
            
            self.log("PHASE_2_DONE", {
                "final_acts": len(movie['acts']),
                "total_tokens": initial_response['tokens'] + continuation['tokens']
            })
        
        # Phase 3: Validate
        self._validate_all_entities_positioned(movie)
        
        return movie
    
    def _call_gemini_with_logging(self, prompt, max_tokens):
        """Call Gemini with comprehensive logging"""
        self.log("GEMINI_CALL", {
            "max_tokens": max_tokens,
            "prompt_preview": prompt[:100]
        })
        
        response = gemini.generate(
            prompt=prompt,
            system_prompt=DIRECTOR_IDENTITY,
            max_tokens=max_tokens,
            temperature=1.35
        )
        
        self.log("GEMINI_RESPONSE", {
            "input_tokens": response['usage']['input_tokens'],
            "output_tokens": response['usage']['output_tokens'],
            "total_tokens": response['usage']['total_tokens']
        })
        
        return response
```

### Task 2: Executor Enhancement (Rust)

**File:** `rust/crates/animation/src/executor.rs` (NEW)

```rust
pub struct SceneExecutor {
    current_movie: CinematicMovie,
    current_act_idx: usize,
    act_start_time: f64,
    
    entity_states: HashMap<String, EntityExecutionState>,
    logs: Vec<ExecutionLog>,
}

pub struct EntityExecutionState {
    pos: (f32, f32, f32),
    target_pos: (f32, f32, f32),
    movement_start_time: f64,
    movement_duration: f64,
    current_action: String,
    pose: StickmanPose,
}

impl SceneExecutor {
    pub fn new(movie: CinematicMovie) -> Self {
        Self {
            current_movie: movie,
            current_act_idx: 0,
            act_start_time: 0.0,
            entity_states: HashMap::new(),
            logs: Vec::new(),
        }
    }
    
    pub fn update(&mut self, elapsed: f64) {
        let current_act = &self.current_movie.acts[self.current_act_idx];
        
        self.log_frame(elapsed);
        
        for entity in &current_act.entities {
            self.execute_entity(entity, elapsed);
        }
        
        // Check if act complete
        if elapsed > current_act.start_time + current_act.duration {
            self.log("ACT_COMPLETE", current_act);
            self.current_act_idx += 1;
        }
    }
    
    fn execute_entity(&mut self, entity: &StageEntity, elapsed: f64) {
        let entity_key = &entity.id;
        
        let state = self.entity_states.entry(entity_key.clone())
            .or_insert_with(|| EntityExecutionState {
                pos: (entity.pos_x, entity.pos_y, entity.pos_z),
                target_pos: (
                    entity.end_x.unwrap_or(entity.pos_x),
                    entity.end_y.unwrap_or(entity.pos_y),
                    entity.pos_z
                ),
                movement_start_time: elapsed,
                movement_duration: self.estimate_movement_duration(entity),
                current_action: entity.action.clone(),
                pose: StickmanPose::neutral(),
            });
        
        // Calculate movement progress
        let elapsed_in_movement = elapsed - state.movement_start_time;
        let progress = (elapsed_in_movement / state.movement_duration).clamp(0.0, 1.0);
        
        // Interpolate position
        let new_pos = self.interpolate(state.pos, state.target_pos, progress);
        state.pos = new_pos;
        
        // Log position update
        self.log_entity_position(entity_key, new_pos, elapsed, progress);
        
        // Update pose
        state.pose = get_pose_from_action(&state.current_action, progress as f64);
    }
    
    fn log_frame(&mut self, elapsed: f64) {
        eprintln!("[SCENE_EXEC] ──────────────────────────────────");
        eprintln!("[SCENE_EXEC] Frame Time: {:.2}s", elapsed);
        eprintln!("[SCENE_EXEC] Current Act: {}/{}", 
            self.current_act_idx + 1, 
            self.current_movie.acts.len());
    }
    
    fn log_entity_position(&mut self, entity_id: &str, pos: (f32, f32, f32), 
                          elapsed: f64, progress: f32) {
        eprintln!("[EXEC_ENTITY] id={} | pos=({:.3}, {:.3}, {:.3}) | time={:.2}s | progress={:.1}%",
            entity_id, pos.0, pos.1, pos.2, elapsed, progress * 100.0);
        
        self.logs.push(ExecutionLog {
            timestamp: elapsed,
            entity_id: entity_id.to_string(),
            pos_x: pos.0,
            pos_y: pos.1,
            pos_z: pos.2,
            progress_pct: (progress * 100.0) as u8,
        });
    }
}
```

---

## 💾 DATABASE SCHEMA FOR LOGGING

```sql
CREATE TABLE ai_generation_logs (
    id INTEGER PRIMARY KEY,
    session_id TEXT,
    prompt TEXT,
    phase TEXT,  -- 'initial', 'continuation', 'validation'
    timestamp REAL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    total_tokens INTEGER,
    acts_generated INTEGER,
    is_complete BOOLEAN,
    error_message TEXT
);

CREATE TABLE scene_execution_logs (
    id INTEGER PRIMARY KEY,
    session_id TEXT,
    scene_id TEXT,
    timestamp REAL,
    entity_id TEXT,
    pos_x REAL,
    pos_y REAL,
    pos_z REAL,
    action TEXT,
    progress_pct INTEGER,
    frame_number INTEGER
);
```

---

## 🎬 EXECUTION FLOW EXAMPLE

```
User Input: "Polisi mengejar pencuri di kota"
           ↓
[AI_DIRECTOR] Starting scene generation
[AI_DIRECTOR] PHASE_1: Initial AI call
  • Prompt: "Generate a police chase scene..."
  • Max tokens: 4500
  • Temperature: 1.35
           ↓
[GEMINI] Processing...
  • Input tokens: 847
  • Output tokens: 3421
  • Total: 4268
           ↓
[AI_DIRECTOR] Received response
  • Acts generated: 2
  • Total entities: 8
  • Acts complete: 60%
           ↓
[AI_DIRECTOR] PHASE_2: Incomplete scene, continuing...
[GEMINI] Continuation call
  • Summarize previous acts: ~300 tokens
  • Continuation prompt: ~150 tokens
  • Generation budget: ~3000 tokens
           ↓
[GEMINI] Response: Acts 3-4 complete
  • Tokens used: 2847
           ↓
[AI_DIRECTOR] PHASE_3: Validation
  • All characters have positions: ✓
  • All entities have end_x/end_y: ✓
  • Total duration: 120.0s
           ↓
[EXECUTOR] Ready to render
[EXECUTOR] Starting execution
           ↓
Loop: For each frame at 60 FPS:
  [EXEC_ENTITY] thief | pos=(0.800, 0.000, 1.000) → (0.500, 0.000, 1.000)
  [EXEC_ENTITY] thief | pos=(0.780, 0.000, 1.000) | 10% complete
  [EXEC_ENTITY] thief | pos=(0.760, 0.000, 1.000) | 20% complete
  [EXEC_ENTITY] thief | pos=(0.740, 0.000, 1.000) | 30% complete
  ...
  [EXEC_ENTITY] thief | pos=(0.500, 0.000, 1.000) | 100% complete
           ↓
[RENDERER] Displaying smooth character movement with proper positioning
```

---

## ✅ SUCCESS CRITERIA

- [ ] AI generates intelligent scene choreography (not hardcoded loops)
- [ ] Token usage tracked and logged
- [ ] Incremental generation works (multi-phase AI calls)
- [ ] All entity positions logged with timestamps
- [ ] Character movement is smooth and reaches target positions
- [ ] Comprehensive logging for debugging
- [ ] No repeated looping of animations
- [ ] Scene completes based on act duration
- [ ] Characters respect z-layer ordering

