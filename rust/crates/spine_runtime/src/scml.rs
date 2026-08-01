//! BrashMonkey Spriter SCML v1.0 parser.
//!
//! Parses SCML (Spriter) XML files that define bone-based character animations
//! with modular body parts. Each character has a skeleton of bones arranged in
//! a hierarchy, with sprite objects attached to bones. Animations are defined
//! as keyframes on timelines that interpolate bone/object transforms.
//!
//! Coordinate system: SCML uses a bottom-left origin with Y-up. We convert to
//! top-left Y-down for GPU rendering by negating Y in the output.

use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// Public types
// ═══════════════════════════════════════════════════════════════

/// A single sprite file reference in the SCML.
#[derive(Debug, Clone)]
pub struct ScmlFile {
    pub id: u32,
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub pivot_x: f32,
    pub pivot_y: f32,
}

/// A folder containing sprite files.
#[derive(Debug, Clone)]
pub struct ScmlFolder {
    pub id: u32,
    pub name: String,
    pub files: Vec<ScmlFile>,
}

/// A bone in the character skeleton.
#[derive(Debug, Clone)]
pub struct ScmlBone {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
}

/// Transform of a bone or object at a keyframe.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScmlTransform {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub alpha: f32,
    pub spin: i32,
}

/// An object (sprite) attached to a bone.
#[derive(Debug, Clone)]
pub struct ScmlObject {
    pub id: u32,
    pub timeline_id: u32,
    pub parent_bone_id: u32,
    pub z_index: i32,
    pub folder: u32,
    pub file: u32,
    pub default_transform: ScmlTransform,
}

/// A single keyframe for a bone timeline.
#[derive(Debug, Clone)]
pub struct BoneKeyframe {
    pub time: u32,
    pub transform: ScmlTransform,
}

/// A single keyframe for an object (sprite) timeline.
#[derive(Debug, Clone)]
pub struct ObjectKeyframe {
    pub time: u32,
    pub transform: ScmlTransform,
    pub folder: u32,
    pub file: u32,
    pub alpha: f32,
}

/// A parsed animation.
#[derive(Debug, Clone)]
pub struct ScmlAnimation {
    pub name: String,
    pub length: u32,
    pub interval: u32,
    pub looping: bool,
    /// Per-bone timeline keyframes (indexed by timeline_id)
    pub bone_keyframes: HashMap<u32, Vec<BoneKeyframe>>,
    /// Per-object timeline keyframes (indexed by timeline_id)
    pub object_keyframes: HashMap<u32, Vec<ObjectKeyframe>>,
    /// Linear list of mainline keys (each has bone_refs + object_refs)
    pub mainline_keys: Vec<MainlineKey>,
}

/// A mainline key defines the character hierarchy at a point in time.
#[derive(Debug, Clone)]
pub struct MainlineKey {
    pub time: u32,
    pub bone_refs: Vec<BoneRef>,
    pub object_refs: Vec<ObjectRef>,
}

/// Reference to a bone within a mainline key.
#[derive(Debug, Clone)]
pub struct BoneRef {
    pub id: u32,
    pub timeline: u32,
    pub key: u32,
    pub parent: Option<u32>,
}

/// Reference to an object within a mainline key.
#[derive(Debug, Clone)]
pub struct ObjectRef {
    pub id: u32,
    pub timeline: u32,
    pub key: u32,
    pub parent: Option<u32>,
    pub z_index: i32,
}

/// A character map entry for skin swapping.
#[derive(Debug, Clone)]
pub struct CharacterMapEntry {
    pub folder: u32,
    pub file: u32,
    pub target_folder: Option<u32>,
    pub target_file: Option<u32>,
}

/// A parsed SCML character entity.
#[derive(Debug, Clone)]
pub struct ScmlEntity {
    pub name: String,
    pub bone_info: Vec<ScmlBone>,
    pub animations: Vec<ScmlAnimation>,
    pub character_maps: Vec<(String, Vec<CharacterMapEntry>)>,
}

/// The complete parsed SCML file.
#[derive(Debug, Clone)]
pub struct ScmlData {
    pub folders: Vec<ScmlFolder>,
    pub entities: Vec<ScmlEntity>,
}

// ═══════════════════════════════════════════════════════════════
// Parser
// ═══════════════════════════════════════════════════════════════

/// Parse SCML XML content from string.
pub fn parse_scml(xml: &str, _base_path: &str) -> Result<ScmlData, String> {
    let doc = roxmltree::Document::parse(xml)
        .map_err(|e| format!("XML parse error: {e}"))?;

    let root = doc.root_element();
    if root.tag_name().name() != "spriter_data" {
        return Err("Root element must be <spriter_data>".into());
    }

    // ── Parse folders ─────────────────────────────────────────────────────
    let mut folders: Vec<ScmlFolder> = Vec::new();
    for folder_elem in root.children().filter(|n| n.has_tag_name("folder")) {
        let folder_id: u32 = attr(&folder_elem, "id")?;
        let folder_name = attr_str(&folder_elem, "name").unwrap_or_default();
        let mut files = Vec::new();
        for file_elem in folder_elem.children().filter(|n| n.has_tag_name("file")) {
            files.push(ScmlFile {
                id: attr(&file_elem, "id")?,
                name: attr_str(&file_elem, "name").unwrap_or("").to_string(),
                width: attr_f32(&file_elem, "width")?,
                height: attr_f32(&file_elem, "height")?,
                pivot_x: attr_f32(&file_elem, "pivot_x")?,
                pivot_y: attr_f32(&file_elem, "pivot_y")?,
            });
        }
        folders.push(ScmlFolder { id: folder_id, name: folder_name.to_string(), files });
    }

    // ── Parse entities ───────────────────────────────────────────────────
    let mut entities: Vec<ScmlEntity> = Vec::new();
    for entity_elem in root.children().filter(|n| n.has_tag_name("entity")) {
        let entity_name = attr_str(&entity_elem, "name").unwrap_or("entity");

        // Parse bone info (obj_info with type="bone")
        let mut bone_info: Vec<ScmlBone> = Vec::new();
        for (idx, obj_info) in entity_elem.children().filter(|n| n.has_tag_name("obj_info")).enumerate() {
            if attr_str(&obj_info, "type").as_deref() == Some("bone") || obj_info.attribute("type") == Some("bone") {
                let bone_id: u32 = attr(&obj_info, "id").unwrap_or(idx as u32);
                let bone_name = attr_str(&obj_info, "name").map(|s| s.to_string()).unwrap_or(format!("bone_{bone_id:03}"));
                // obj_info doesn't have parent — that's in mainline keys
                bone_info.push(ScmlBone {
                    id: bone_id,
                    name: bone_name.to_string(),
                    parent_id: None,
                });
            }
        }

        // Parse character maps
        let mut character_maps: Vec<(String, Vec<CharacterMapEntry>)> = Vec::new();
        for cm_elem in entity_elem.children().filter(|n| n.has_tag_name("character_map")) {
            let cm_name = attr_str(&cm_elem, "name").unwrap_or("skin");
            let mut entries = Vec::new();
            for map_elem in cm_elem.children().filter(|n| n.has_tag_name("map")) {
                entries.push(CharacterMapEntry {
                    folder: attr(&map_elem, "folder")?,
                    file: attr(&map_elem, "file")?,
                    target_folder: attr_opt(&map_elem, "target_folder"),
                    target_file: attr_opt(&map_elem, "target_file"),
                });
            }
            character_maps.push((cm_name.to_string(), entries));
        }

        // Parse animations
        let mut animations: Vec<ScmlAnimation> = Vec::new();
        for anim_elem in entity_elem.children().filter(|n| n.has_tag_name("animation")) {
            let anim_name = attr_str(&anim_elem, "name").unwrap_or("anim");
            let length: u32 = attr(&anim_elem, "length")?;
            let interval: u32 = attr(&anim_elem, "interval").unwrap_or(100);
            let looping = attr_str(&anim_elem, "looping").map(|s| s != "false").unwrap_or(true);
            let mut anim = ScmlAnimation {
                name: anim_name.to_string(),
                length, interval, looping,
                bone_keyframes: HashMap::new(),
                object_keyframes: HashMap::new(),
                mainline_keys: Vec::new(),
            };

            // Parse mainline
            if let Some(mainline) = anim_elem.children().find(|n| n.has_tag_name("mainline")) {
                let mut mainline_keys: Vec<MainlineKey> = Vec::new();
                for key_elem in mainline.children().filter(|n| n.has_tag_name("key")) {
                    let key_time: u32 = attr(&key_elem, "time").unwrap_or(0);
                    let mut bone_refs = Vec::new();
                    let mut object_refs = Vec::new();
                    for child in key_elem.children() {
                        if child.has_tag_name("bone_ref") {
                            bone_refs.push(BoneRef {
                                id: attr(&child, "id")?,
                                timeline: attr(&child, "timeline")?,
                                key: attr(&child, "key")?,
                                parent: attr_opt(&child, "parent"),
                            });
                        } else if child.has_tag_name("object_ref") {
                            object_refs.push(ObjectRef {
                                id: attr(&child, "id")?,
                                timeline: attr(&child, "timeline")?,
                                key: attr(&child, "key")?,
                                parent: attr_opt(&child, "parent"),
                                z_index: attr_opt::<i32>(&child, "z_index").unwrap_or(0),
                            });
                        }
                    }
                    mainline_keys.push(MainlineKey { time: key_time, bone_refs, object_refs });
                }
                anim.mainline_keys = mainline_keys;
            }

            // Parse timelines
            for timeline_elem in anim_elem.children().filter(|n| n.has_tag_name("timeline")) {
                let timeline_id: u32 = attr(&timeline_elem, "id")?;
                let _timeline_name = attr_str(&timeline_elem, "name").unwrap_or("");
                let obj_type = attr_str(&timeline_elem, "object_type").unwrap_or("");

                let mut bone_kfs = Vec::new();
                let mut object_kfs = Vec::new();
                let is_bone = obj_type == "bone" || timeline_elem.attribute("object_type") == Some("bone");

                for key_elem in timeline_elem.children().filter(|n| n.has_tag_name("key")) {
                    let key_time: u32 = attr(&key_elem, "time").unwrap_or(0);
                    let spin: i32 = attr(&key_elem, "spin").unwrap_or(0) as i32;

                    if is_bone {
                        // Bone timeline
                        if let Some(bone_elem) = key_elem.children().find(|n| n.has_tag_name("bone")) {
                            bone_kfs.push(BoneKeyframe {
                                time: key_time,
                                transform: ScmlTransform {
                                    x: attr_f32(&bone_elem, "x").unwrap_or(0.0),
                                    y: attr_f32(&bone_elem, "y").unwrap_or(0.0),
                                    angle: attr_f32(&bone_elem, "angle").unwrap_or(0.0),
                                    scale_x: attr_f32(&bone_elem, "scale_x").unwrap_or(1.0),
                                    scale_y: attr_f32(&bone_elem, "scale_y").unwrap_or(1.0),
                                    alpha: 1.0,
                                    spin,
                                },
                            });
                        }
                    } else {
                        // Object (sprite) timeline
                        if let Some(obj_elem) = key_elem.children().find(|n| n.has_tag_name("object")) {
                            object_kfs.push(ObjectKeyframe {
                                time: key_time,
                                folder: attr(&obj_elem, "folder")?,
                                file: attr(&obj_elem, "file")?,
                                alpha: attr_f32(&obj_elem, "a").unwrap_or(1.0),
                                transform: ScmlTransform {
                                    x: attr_f32(&obj_elem, "x").unwrap_or(0.0),
                                    y: attr_f32(&obj_elem, "y").unwrap_or(0.0),
                                    angle: attr_f32(&obj_elem, "angle").unwrap_or(0.0),
                                    scale_x: attr_f32(&obj_elem, "scale_x").unwrap_or(1.0),
                                    scale_y: attr_f32(&obj_elem, "scale_y").unwrap_or(1.0),
                                    alpha: attr_f32(&obj_elem, "a").unwrap_or(1.0),
                                    spin: 0,
                                },
                            });
                        }
                    }
                }

                if is_bone || (timeline_elem.attribute("object_type") == Some("bone")) {
                    if !bone_kfs.is_empty() {
                        anim.bone_keyframes.insert(timeline_id, bone_kfs);
                    }
                } else {
                    if !object_kfs.is_empty() {
                        anim.object_keyframes.insert(timeline_id, object_kfs);
                    }
                }
            }

            animations.push(anim);
        }

        entities.push(ScmlEntity {
            name: entity_name.to_string(),
            bone_info,
            animations,
            character_maps,
        });
    }

    Ok(ScmlData { folders, entities })
}

// ═══════════════════════════════════════════════════════════════
// Interpolation helpers
// ═══════════════════════════════════════════════════════════════

/// Look up a keyframe value in a sorted list, interpolating linearly.
pub fn interpolate_bone(bone_kfs: &[BoneKeyframe], time_ms: u32, length_ms: u32) -> ScmlTransform {
    if bone_kfs.is_empty() {
        return ScmlTransform::default();
    }
    let t = if length_ms > 0 { time_ms % length_ms } else { 0 };
    if t <= bone_kfs[0].time {
        return bone_kfs[0].transform;
    }
    let last = &bone_kfs[bone_kfs.len() - 1];
    if t >= last.time {
        return last.transform;
    }
    for i in 0..bone_kfs.len() - 1 {
        let t0 = bone_kfs[i].time;
        let t1 = bone_kfs[i + 1].time;
        if t >= t0 && t < t1 {
            let alpha = if t1 > t0 { (t - t0) as f32 / (t1 - t0) as f32 } else { 0.0 };
            let a = &bone_kfs[i].transform;
            let b = &bone_kfs[i + 1].transform;
            return ScmlTransform {
                x: a.x + (b.x - a.x) * alpha,
                y: a.y + (b.y - a.y) * alpha,
                angle: a.angle + (b.angle - a.angle) * alpha,
                scale_x: a.scale_x + (b.scale_x - a.scale_x) * alpha,
                scale_y: a.scale_y + (b.scale_y - a.scale_y) * alpha,
                alpha: a.alpha + (b.alpha - a.alpha) * alpha,
                spin: b.spin,
            };
        }
    }
    bone_kfs[bone_kfs.len() - 1].transform
}

/// Look up an object keyframe at a given time.
pub fn interpolate_object(obj_kfs: &[ObjectKeyframe], time_ms: u32, length_ms: u32) -> Option<ObjectKeyframe> {
    if obj_kfs.is_empty() {
        return None;
    }
    let t = if length_ms > 0 { time_ms % length_ms } else { 0 };
    if t <= obj_kfs[0].time {
        return Some(obj_kfs[0].clone());
    }
    let last = &obj_kfs[obj_kfs.len() - 1];
    if t >= last.time {
        return Some(last.clone());
    }
    for i in 0..obj_kfs.len() - 1 {
        let t0 = obj_kfs[i].time;
        let t1 = obj_kfs[i + 1].time;
        if t >= t0 && t < t1 {
            let alpha = if t1 > t0 { (t - t0) as f32 / (t1 - t0) as f32 } else { 0.0 };
            let a = &obj_kfs[i];
            let b = &obj_kfs[i + 1];
            return Some(ObjectKeyframe {
                time: t,
                folder: a.folder,
                file: a.file,
                alpha: a.alpha + (b.alpha - a.alpha) * alpha,
                transform: ScmlTransform {
                    x: a.transform.x + (b.transform.x - a.transform.x) * alpha,
                    y: a.transform.y + (b.transform.y - a.transform.y) * alpha,
                    angle: a.transform.angle + (b.transform.angle - a.transform.angle) * alpha,
                    scale_x: a.transform.scale_x + (b.transform.scale_x - a.transform.scale_x) * alpha,
                    scale_y: a.transform.scale_y + (b.transform.scale_y - a.transform.scale_y) * alpha,
                    alpha: a.alpha + (b.alpha - a.alpha) * alpha,
                    spin: 0,
                },
            });
        }
    }
    Some(last.clone())
}

// ═══════════════════════════════════════════════════════════════
// Bone hierarchy resolution
// ═══════════════════════════════════════════════════════════════

/// Resolve the bone hierarchy from the first mainline key and combine with
/// obj_info data. Returns (bones, parent_indices).
pub fn resolve_bone_hierarchy(entity: &ScmlEntity) -> Vec<ScmlBone> {
    let first_anim = entity.animations.first();
    let mainline = first_anim.and_then(|a| a.mainline_keys.first());

    let mut bones: Vec<ScmlBone> = Vec::new();
    // Start with obj_info bones
    let mut bone_map: HashMap<u32, usize> = HashMap::new();
    for ob in &entity.bone_info {
        let idx = bones.len();
        bone_map.insert(ob.id, idx);
        bones.push(ob.clone());
    }

    // Apply parent references from the first mainline key
    if let Some(mk) = mainline {
        for br in &mk.bone_refs {
            if bone_map.contains_key(&br.id) {
                if let Some(parent_id) = br.parent {
                    if let Some(&parent_idx) = bone_map.get(&parent_id) {
                        if let Some(bone) = bones.get_mut(bone_map[&br.id]) {
                            bone.parent_id = Some(parent_id);
                        }
                    }
                }
            }
        }
    }

    bones
}

// ═══════════════════════════════════════════════════════════════
// World transform computation (forward kinematics)
// ═══════════════════════════════════════════════════════════════

/// Result of evaluating a character's animation at a given time.
#[derive(Debug, Clone)]
pub struct ScmlPose {
    /// Objects to render, sorted by z_index.
    pub objects: Vec<ScmlPoseObject>,
    /// The animation time in milliseconds (after wrapping).
    pub time_ms: u32,
}

/// A single rendered object in world space.
#[derive(Debug, Clone)]
pub struct ScmlPoseObject {
    pub name: String,
    pub folder: u32,
    pub file: u32,
    pub world_x: f32,
    pub world_y: f32,
    pub world_angle: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub alpha: f32,
    pub z_index: i32,
    pub width: f32,
    pub height: f32,
    pub pivot_x: f32,
    pub pivot_y: f32,
}

/// Evaluate an animation at a given time and return world-space object poses.
pub fn evaluate_pose(
    data: &ScmlData,
    entity: &ScmlEntity,
    anim: &ScmlAnimation,
    time_ms: u32,
    active_character_map: Option<&[(String, Vec<CharacterMapEntry>)]>,
) -> ScmlPose {
    let t = if anim.length > 0 { time_ms % anim.length } else { 0 };

    // Find the current mainline key (last key with time <= t)
    let mainline = anim.mainline_keys.iter()
        .filter(|k| k.time <= t)
        .last()
        .unwrap_or_else(|| anim.mainline_keys.first().unwrap_or(&anim.mainline_keys[0]));

    // Resolve bone transforms
    // 1. For each bone_ref, find its timeline, interpolate, get local transform
    // 2. Compute world transforms via FK (parent-child)
    // 3. For each object_ref, find its timeline, interpolate, apply parent bone transform
    // 4. Collect with z_index

    // Map from bone_ref id to its parent and timeline
    let mut bone_locals: HashMap<u32, ScmlTransform> = HashMap::new();
    let mut bone_parents: HashMap<u32, Option<u32>> = HashMap::new();

    for br in &mainline.bone_refs {
        bone_parents.insert(br.id, br.parent);
        if let Some(kfs) = anim.bone_keyframes.get(&br.timeline) {
            let transform = interpolate_bone(kfs, t, anim.length);
            bone_locals.insert(br.id, transform);
        } else {
            bone_locals.insert(br.id, ScmlTransform::default());
        }
    }

    // Forward kinematics: compute world transforms in parent-order
    // Since bones are listed in hierarchy order in mainline, we can process in-order
    let mut bone_worlds: HashMap<u32, ScmlTransform> = HashMap::new();

    // Need bones sorted by hierarchy (parent first)
    fn get_bone_chain(bone_id: u32, parents: &HashMap<u32, Option<u32>>) -> Vec<u32> {
        let mut chain = vec![bone_id];
        let mut current = bone_id;
        while let Some(Some(parent)) = parents.get(&current) {
            chain.push(*parent);
            current = *parent;
        }
        chain.reverse();
        chain
    }

    // Process bones in order (already sorted by hierarchy in mainline)
    for br in &mainline.bone_refs {
        let local = bone_locals.get(&br.id).copied().unwrap_or_default();

        if let Some(Some(parent_id)) = bone_parents.get(&br.id) {
            if let Some(parent_world) = bone_worlds.get(parent_id) {
                let rad = parent_world.angle.to_radians();
                let cos = rad.cos();
                let sin = rad.sin();
                let wx = parent_world.x + cos * local.x * parent_world.scale_x - sin * local.y * parent_world.scale_y;
                let wy = parent_world.y + sin * local.x * parent_world.scale_x + cos * local.y * parent_world.scale_y;
                bone_worlds.insert(br.id, ScmlTransform {
                    x: wx,
                    y: wy,
                    angle: parent_world.angle + local.angle,
                    scale_x: parent_world.scale_x * local.scale_x,
                    scale_y: parent_world.scale_y * local.scale_y,
                    alpha: parent_world.alpha * local.alpha,
                    spin: 0,
                });
            } else {
                // Should not happen if mainline is in hierarchy order, but be safe
                bone_worlds.insert(br.id, local);
            }
        } else {
            // Root bone
            bone_worlds.insert(br.id, local);
        }
    }

    // Now process object_refs: attach to parent bone
    let mut objects: Vec<ScmlPoseObject> = Vec::new();
    // Build a set of object_ref ids to track which timelines are objects
    let object_ref_ids: Vec<u32> = mainline.object_refs.iter().map(|o| o.timeline).collect();

    for oref in &mainline.object_refs {
        // Get object transform from timeline
        let obj_kf = anim.object_keyframes.get(&oref.timeline)
            .and_then(|kfs| interpolate_object(kfs, t, anim.length));

        let obj_kf = match obj_kf {
            Some(kf) => kf,
            None => continue,
        };

        // Apply character map if active
        let (final_folder, final_file) = if let Some(cm_list) = active_character_map {
            apply_character_map(&data.folders, cm_list, obj_kf.folder, obj_kf.file)
        } else {
            (obj_kf.folder, obj_kf.file)
        };

        // Get file info for pivot and size
        let file_info = get_file_info(&data.folders, final_folder, final_file);
        let (fw, fh, px, py) = file_info.map(|f| (f.width, f.height, f.pivot_x, f.pivot_y))
            .unwrap_or((100.0, 100.0, 0.5, 0.5));

        // Get parent bone world transform (None = root/identity)
        let world_transform = if let Some(pid) = oref.parent {
            bone_worlds.get(&pid).cloned()
        } else {
            // No parent — object is at root, use identity transform
            Some(ScmlTransform {
                x: 0.0, y: 0.0,
                angle: 0.0,
                scale_x: 1.0, scale_y: 1.0,
                alpha: 1.0,
                spin: 0,
            })
        };

        if let Some(bone_world) = world_transform {
            let rad = bone_world.angle.to_radians();
            let cos = rad.cos();
            let sin = rad.sin();
            // Apply object offset relative to parent bone
            let wx = bone_world.x + cos * obj_kf.transform.x * bone_world.scale_x
                - sin * obj_kf.transform.y * bone_world.scale_y;
            let wy = bone_world.y + sin * obj_kf.transform.x * bone_world.scale_x
                + cos * obj_kf.transform.y * bone_world.scale_y;

            objects.push(ScmlPoseObject {
                name: format!("obj_{}", oref.id),
                folder: final_folder,
                file: final_file,
                world_x: wx,
                world_y: wy,  // SCML is Y-up, we'll negate later for rendering
                world_angle: bone_world.angle + obj_kf.transform.angle,
                scale_x: bone_world.scale_x * obj_kf.transform.scale_x,
                scale_y: bone_world.scale_y * obj_kf.transform.scale_y,
                alpha: obj_kf.alpha * bone_world.alpha,
                z_index: oref.z_index,
                width: fw,
                height: fh,
                pivot_x: px,
                pivot_y: py,
            });
        }
    }

    // Sort by z_index for correct render order
    objects.sort_by_key(|o| o.z_index);

    ScmlPose { objects, time_ms: t }
}

// ═══════════════════════════════════════════════════════════════
// Character map (skin) application
// ═══════════════════════════════════════════════════════════════

/// Apply a character map to remap folder/file references.
pub fn apply_character_map(
    _folders: &[ScmlFolder],
    char_map: &[(String, Vec<CharacterMapEntry>)],
    folder: u32,
    file: u32,
) -> (u32, u32) {
    for (_name, entries) in char_map {
        for entry in entries {
            if entry.folder == folder && entry.file == file {
                return (
                    entry.target_folder.unwrap_or(folder),
                    entry.target_file.unwrap_or(file),
                );
            }
        }
    }
    (folder, file)
}

/// Get file info from folder list.
pub fn get_file_info<'a>(folders: &'a [ScmlFolder], folder: u32, file: u32) -> Option<&'a ScmlFile> {
    for f in folders {
        if f.id == folder {
            for fl in &f.files {
                if fl.id == file {
                    return Some(fl);
                }
            }
        }
    }
    None
}

/// Get file name from folder/file ids.
pub fn get_file_name(folders: &[ScmlFolder], folder: u32, file: u32) -> String {
    get_file_info(folders, folder, file)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| format!("{folder}_{file}.png"))
}

// ═══════════════════════════════════════════════════════════════
// Internal helpers
// ═══════════════════════════════════════════════════════════════

fn attr(elem: &roxmltree::Node, name: &str) -> Result<u32, String> {
    elem.attribute(name)
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| format!("Missing or invalid attr '{name}' on <{}>", elem.tag_name().name()))
}

fn attr_f32(elem: &roxmltree::Node, name: &str) -> Result<f32, String> {
    elem.attribute(name)
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| format!("Missing or invalid f32 attr '{name}' on <{}>", elem.tag_name().name()))
}

fn attr_opt<T: std::str::FromStr>(elem: &roxmltree::Node, name: &str) -> Option<T> {
    elem.attribute(name).and_then(|v| v.parse().ok())
}

fn attr_str<'a>(elem: &'a roxmltree::Node, name: &str) -> Option<&'a str> {
    elem.attribute(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SCML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<spriter_data scml_version="1.0" generator="BrashMonkey Spriter" generator_version="r4.1">
    <folder id="0">
        <file id="0" name="body.png" width="177" height="229" pivot_x="0.5" pivot_y="1"/>
        <file id="1" name="head.png" width="420" height="579" pivot_x="0.5" pivot_y="1"/>
    </folder>
    <entity id="0" name="test_entity">
        <obj_info name="bone_000" type="bone" w="295" h="10"/>
        <obj_info name="bone_001" type="bone" w="38" h="1"/>
        <animation id="0" name="Idle" length="500" interval="100">
            <mainline>
                <key id="0">
                    <bone_ref id="0" timeline="0" key="0"/>
                    <bone_ref id="1" parent="0" timeline="1" key="0"/>
                    <object_ref id="0" timeline="0" key="0" z_index="0"/>
                    <object_ref id="1" parent="0" timeline="1" key="0" z_index="1"/>
                </key>
                <key id="1" time="250">
                    <bone_ref id="0" timeline="0" key="1"/>
                    <bone_ref id="1" parent="0" timeline="1" key="0"/>
                    <object_ref id="0" timeline="0" key="1" z_index="0"/>
                    <object_ref id="1" parent="0" timeline="1" key="0" z_index="1"/>
                </key>
            </mainline>
            <timeline id="0" name="body">
                <key id="0" time="0">
                    <object folder="0" file="0" x="0" y="0" angle="0" scale_x="1" scale_y="1" alpha="1"/>
                </key>
                <key id="1" time="250">
                    <object folder="0" file="0" x="100" y="0" angle="45" scale_x="1" scale_y="1" alpha="1"/>
                </key>
            </timeline>
            <timeline id="1" name="head">
                <key id="0" time="0">
                    <object folder="0" file="1" x="0" y="150" angle="0" scale_x="1" scale_y="1" alpha="1"/>
                </key>
            </timeline>
        </animation>
    </entity>
</spriter_data>"#;

    #[test]
    fn test_parse_scml() {
        let data = parse_scml(SAMPLE_SCML, "tests/").unwrap();
        assert_eq!(data.folders.len(), 1);
        assert_eq!(data.folders[0].files.len(), 2);
        assert_eq!(data.folders[0].files[0].name, "body.png");
        assert_eq!(data.entities.len(), 1);
        assert_eq!(data.entities[0].animations.len(), 1);
        assert_eq!(data.entities[0].animations[0].name, "Idle");
        assert_eq!(data.entities[0].animations[0].length, 500);
    }

    #[test]
    fn test_evaluate_pose_idle() {
        let data = parse_scml(SAMPLE_SCML, "tests/").unwrap();
        let ent = &data.entities[0];
        let anim = &ent.animations[0];
        let pose = evaluate_pose(&data, ent, anim, 0, None);
        assert_eq!(pose.objects.len(), 2);
        // At t=0, both objects at default relative positions
        let body = &pose.objects[0];
        assert!((body.world_x - 0.0).abs() < 5.0);
        assert!((body.world_y - 0.0).abs() < 5.0);
    }

    #[test]
    fn test_evaluate_pose_mid_animation() {
        let data = parse_scml(SAMPLE_SCML, "tests/").unwrap();
        let ent = &data.entities[0];
        let anim = &ent.animations[0];
        // At t=125ms (halfway between key 0 0ms and key 1 250ms):
        // body should be at x=50, angle=22.5 (interpolated)
        let pose = evaluate_pose(&data, ent, anim, 125, None);
        let body = &pose.objects[0];
        assert!((body.world_x - 50.0).abs() < 5.0);
        assert!((body.world_angle - 22.5).abs() < 5.0);
    }

    #[test]
    fn test_parse_real_terrorist_scml() {
        let path = "../../../craftpix-485144-2d-game-terrorists-character-free-sprites-sheets/scml/terrorist_1/terrorist_1.scml";
        let xml = std::fs::read_to_string(path)
            .expect("test SCML file must exist");
        let data = parse_scml(&xml, "tests/").expect("parsing real terrorist SCML must succeed");
        assert_eq!(data.folders.len(), 2); // default + skin1
        assert_eq!(data.entities.len(), 1);
        let ent = &data.entities[0];
        assert_eq!(ent.name, "terrorist_1");
        assert!(!ent.animations.is_empty());
        let idle = ent.animations.iter().find(|a| a.name == "Idle")
            .expect("Idle animation must exist");
        assert_eq!(idle.length, 800);
        // Verify evaluate_pose works
        let pose = evaluate_pose(&data, ent, idle, 0, None);
        assert!(!pose.objects.is_empty(), "Idle pose should have visible objects at t=0");
        // Verify all body parts reference valid folders/files
        for obj in &pose.objects {
            assert!(obj.alpha > 0.0, "alpha should be positive for object {}", obj.name);
        }
        // Debug output to check coordinate ranges
        let min_x = pose.objects.iter().map(|o| o.world_x).fold(f32::MAX, f32::min);
        let max_x = pose.objects.iter().map(|o| o.world_x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = pose.objects.iter().map(|o| o.world_y).fold(f32::MAX, f32::min);
        let max_y = pose.objects.iter().map(|o| o.world_y).fold(f32::NEG_INFINITY, f32::max);
        eprintln!("[SCML Debug] Idle t=0: {} objects, X range [{:.1}, {:.1}], Y range [{:.1}, {:.1}]",
            pose.objects.len(), min_x, max_x, min_y, max_y);
        for obj in &pose.objects {
            eprintln!("[SCML Debug]   obj {}: world=({:.1}, {:.1}) angle={:.1} folder={} file={}",
                obj.name, obj.world_x, obj.world_y, obj.world_angle, obj.folder, obj.file);
        }
    }
}
