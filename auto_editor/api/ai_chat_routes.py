"""AI-powered animation chat endpoint for OpenCut Studio.

POST /api/studio/ai-chat  — Accept user prompt, call OpenRouter AI,
                            return structured animation timeline JSON.
"""
from __future__ import annotations

import asyncio
import json
import logging
import os
from pathlib import Path

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/api/studio")

# ── Load API key from config ─────────────────────────────────────────────────

_CONFIG_PATH = Path(__file__).parent.parent.parent / "opencut_config.json"

def _load_api_key() -> str:
    try:
        with open(_CONFIG_PATH) as f:
            return json.load(f).get("openrouter_api_key", "")
    except Exception:
        return os.getenv("OPENROUTER_API_KEY", "")


def _load_model() -> str:
    try:
        with open(_CONFIG_PATH) as f:
            data = json.load(f)
            return data.get("gateways", {}).get("openrouter", {}).get("model", "meta-llama/llama-3.1-8b-instruct:free")
    except Exception:
        return "meta-llama/llama-3.1-8b-instruct:free"


def _load_openrouter_url() -> str:
    try:
        with open(_CONFIG_PATH) as f:
            data = json.load(f)
            return data.get("gateways", {}).get("openrouter", {}).get("url", "https://openrouter.ai/api/v1")
    except Exception:
        return "https://openrouter.ai/api/v1"


# ── Director-level System Prompt ─────────────────────────────────────────────

SYSTEM_PROMPT = """You are an expert 3D animation director for OpenCut Studio.

When the user describes a scene or story, you think like a film director:
- Choose the RIGHT environment & lighting for the mood
- Place props that make the scene believable and rich
- Plan character movements that tell the story naturally
- Design camera movement to enhance the cinematic feel
- Populate the scene with contextually appropriate objects

You respond with ONLY valid JSON. No markdown, no explanation, no code blocks.

═══════════════════════════════════════════════
COORDINATE SYSTEM
═══════════════════════════════════════════════
- X axis: negative = left side of scene, positive = right side
- Y axis: 0 = ground level, positive = up
- Z axis: 0 = center of scene, negative = deeper/background, positive = foreground
- Character starts at approximately [0, 0, 0.5]
- Foreground zone: z = 0 to 2 (close to camera)
- Midground zone: z = -1 to -4 (middle of scene)
- Background zone: z = -5 to -10 (far back)

═══════════════════════════════════════════════
AVAILABLE CHARACTERS (characterType)
═══════════════════════════════════════════════
- "stickman"  — Simple stick figure (general purpose)
- "soldier"   — Army soldier (battles, patrols, marching, guarding)
- "robot"     — Expressive robot (tech demos, sci-fi, dancing, punching)
- "michelle"  — Female dancer (celebrations, dances, fashion shows, pop)

═══════════════════════════════════════════════
CHARACTER ACTIONS (timeline clips)
═══════════════════════════════════════════════
- "idle"   — Standing still, breathing
- "walk"   — Walking forward {add "direction": "right" or "left"}
- "run"    — Running fast {add "direction": "right" or "left"}
- "jump"   — Jump in place
- "wave"   — Wave at someone/camera
- "dance"  — Full body dance move
- "think"  — Hand-on-chin thinking pose
- "punch"  — Combat punch

═══════════════════════════════════════════════
AVAILABLE 3D PROP MODELS (type "model_prop")
═══════════════════════════════════════════════
CHARACTERS/CREATURES:
- "horse"      — Running horse (animated). Scale: 0.004. Place at y=0
- "flamingo"   — Flying flamingo (animated). Scale: 0.035. Place at y=2.5 to 5
- "parrot"     — Flying parrot (animated). Scale: 0.03. Place at y=3 to 6
- "fox"        — Cute animated fox. Scale: 0.02. Place at y=0
- "astronaut"  — Astronaut in spacesuit (animated). Scale: 1.0. Place at y=0 or floating

VEHICLES:
- "ferrari"    — Ferrari sports car. Scale: 1.0. Place at y=0
- "toy_car"    — Wooden toy car (small). Scale: 1.5. Place at y=0

FURNITURE / INDOOR:
- "chair"      — Luxury velvet armchair. Scale: 1.8. Place at y=0
- "boombox"    — Retro boombox radio. Scale: 2.5. Place at y=0

PROPS / OBJECTS:
- "duck"          — Yellow rubber duck (small). Scale: 0.008. Place at y=0
- "damaged_helmet"— Sci-fi damaged helmet. Scale: 0.5. Place at y=0.5
- "avocado"       — Giant decorative avocado. Scale: 15.0. Place at y=0
- "lantern"       — Hanging lantern lamp. Scale: 0.12. Place at y=0.5 to 2
- "water_bottle"  — Water bottle. Scale: 1.2. Place at y=0


═══════════════════════════════════════════════
ENVIRONMENT SETTINGS
═══════════════════════════════════════════════
You can set the scene environment to match the user's request.
- theme: "city"      → City street with buildings, trees, Ferrari, birds. Good for: urban, modern, public scenes
- theme: "room"      → Cozy indoor room with wood floor, walls, chair, boombox, lantern. Good for: home, music, relaxing
- theme: "school"    → Classroom with blackboard, student desks, windows. Good for: education, kids, lessons
- theme: "space"     → Outer space with stars, planets, asteroids, floating astronaut. Good for: sci-fi, adventure
- theme: "desert"    → Gurun pasir with orange sand, cacti, and desert rock pillars. Good for: travel, hot weather, dry/barren scenery
- theme: "forest"    → Lush green forest with lots of pine trees and mossy foliage. Good for: nature, camping, mysterious woods
- theme: "ocean"     → Deep ocean / underwater with floating bubbles, sea kelp/weeds, and corals. Good for: diving, swimming, sea creature stories
- theme: "cyberpunk" → Futuristic city at night with dark skyscrapers and glowing neon billboards. Good for: high tech, hacking, neon sci-fi
- theme: "arctic"    → Frozen arctic polar zone with pure white snow, glaciers, and falling snow. Good for: winter, cold, polar exploration
- theme: "volcano"   → Volcanic inferno with lava cracks on the ground, ash embers, and rocky spires. Good for: high danger, lava, hot action
- theme: "studio"    → Massive professional animation studio / filming soundstage with multiple sets (Urban, Fantasy, Sci-Fi Lab), soft-box lights, green screen, cameras, dolly tracks, clapperboards. Good for: filmmaking, shooting, behind the scenes, high-end production

LIGHTING — choose what fits the mood:
- "noon"   → Bright white daylight, crisp shadows. Energetic, cheerful, clear visibility
- "sunset" → Warm orange-gold light, long shadows, romantic. Dreamy, nostalgic, dramatic
- "night"  → Dark blue-black sky, moonlight, glowing lamps. Mysterious, cool, cinematic

═══════════════════════════════════════════════
SCENE COMPOSITION RULES
═══════════════════════════════════════════════
Think like a set designer. For each scene:

CITY scenes:
- Place vehicles on the road sides: ferrari at [3.5, 0, -1] or [-3.5, 0, -2]
- Birds flying in sky: flamingo at [y: 4-6], parrot at [y: 3-5]
- Character walks/runs down the center lane

ROOM/INDOOR scenes:
- Furniture against walls: chair at [-2.5, 0, -3], boombox at [3, 0, -2]
- Lamps hanging: lantern at [0, 2, -3]
- Character stands/dances in center: [0, 0, 0]

SCHOOL scenes:
- Multiple chairs as desks: place 4-6 chairs in rows
- Character stands near blackboard area: x=0, z=-3 to -5

SPACE scenes:
- Astronaut floating: use "float" concept with y: 1.5 to 3
- No ground-based props make sense in space theme

DESERT scenes:
- Add cacti and rocks at side positions (e.g. x: -6 to -8 or 6 to 8)
- Character walks down the sand path

FOREST scenes:
- Position multiple trees around (e.g. x: -5, 5, -8, 8, etc.) to frame the character
- Add fox sitting at side position [3, 0, -2] or horse running in the background

OCEAN scenes:
- Make character walk slowly or float (e.g. y: 0.5 to 1.5)
- Add duck at [2, 0, -2] as a funny sea creature, or sea corals at side

CYBERPUNK scenes:
- Place sci-fi helmet at [-2, 0.3, -2]
- Add neon colors, place boombox on floor, character walks in neon rain/glow

ARCTIC scenes:
- Glacier rocks at sides: [6, 0, -6] or [-6, 0, -6]
- Add cute arctic fox at side position

VOLCANO scenes:
- Rocky spires and lava rivers at sides
- Character stands in hot warning zones, looking scared/thinking

STUDIO scenes:
- Set up as a filming set/animation soundstage. Put cameras, softboxes, boomboxes, chairs, and decorative props.
- Place cameras at [-5, 1, 2] or [5, 1, 2].
- Place a director's chair at [8, 0, 4].
- Place decorative set pieces or props like a damaged helmet at [-2, 0.3, -2].

═══════════════════════════════════════════════
CAMERA KEYFRAMES
═══════════════════════════════════════════════
You can design cinematic camera movement! Add "cameraKeyframes" array.
Camera position [x, y, z] and lookAt [x, y, z] at time t:
- Opening shot: usually pulled back [0, 5, 12] looking at [0, 1, 0]
- Close-up: move closer [0, 3, 5] looking at [0, 1.5, 0]
- Follow shot: offset from character, move with them
- Wide angle: [0, 8, 18] for establishing shots

Examples:
- Dramatic reveal: start wide [0, 6, 14], push in to [0, 3, 6]
- Side tracking: camera moves left/right as character walks
- Low angle hero shot: camera at [0, 1, 6] looking up at [0, 2.5, 0]

═══════════════════════════════════════════════
INFOGRAPHIC ELEMENTS (optional)
═══════════════════════════════════════════════
- type "bar_chart"     — data: [{label, value:0-100, color:#hex}]
- type "pie_chart"     — data: [{label, value:0-100, color:#hex}]
- type "progress_ring" — value: 0-100, color: #hex

═══════════════════════════════════════════════
COMPLETE RESPONSE FORMAT
═══════════════════════════════════════════════
{
  "duration": <total seconds 8.0 to 30.0>,
  "characterType": "robot" | "soldier" | "michelle" | "stickman",
  "environment": {
    "theme": "city" | "room" | "school" | "space" | "desert" | "forest" | "ocean" | "cyberpunk" | "arctic" | "volcano" | "studio",
    "timeOfDay": "noon" | "sunset" | "night"
  },

  "cameraKeyframes": [
    {"t": 0.0, "position": [0, 5, 12], "lookAt": [0, 1.0, 0]},
    {"t": 4.0, "position": [0, 3, 7],  "lookAt": [0, 1.5, 0]},
    {"t": 8.0, "position": [2, 2, 5],  "lookAt": [0, 1.0, 0]}
  ],
  "characters": [{
    "id": "hero",
    "timeline": [
      {"t": 0.0, "action": "idle", "duration": 1.5},
      {"t": 1.5, "action": "walk", "duration": 4.0, "direction": "right"},
      {"t": 5.5, "action": "wave", "duration": 2.0},
      {"t": 7.5, "action": "idle", "duration": 0.5}
    ]
  }],
  "elements": [
    {
      "id": "prop_ferrari",
      "type": "model_prop",
      "modelId": "ferrari",
      "position": [3.5, 0.0, -1.5],
      "scale": 1.0,
      "rotation": [0, -0.4, 0],
      "start": 0.0,
      "duration": 8.0
    },
    {
      "id": "bird1",
      "type": "model_prop",
      "modelId": "flamingo",
      "position": [-2.0, 4.5, -3.0],
      "scale": 0.035,
      "rotation": [0, 0, 0],
      "start": 0.0,
      "duration": 8.0
    }
  ],
  "reply": "<friendly, descriptive reply in the user's language (Indonesian/English) explaining the scene>"
}

═══════════════════════════════════════════════
SCENE DESIGN RULES
═══════════════════════════════════════════════
1. ALWAYS populate the scene richly — minimum 2-3 props for non-space/ocean/arctic scenes
2. Character timeline must cover the entire duration with NO gaps (clips end-to-end)
3. Props that don't move use start=0 and duration=full scene duration
4. Camera keyframes should flow naturally — at least 2-3 keyframes per scene
5. Position props so they DON'T overlap the character's center path (x=0, z=0)
6. Scale props correctly — don't make ferrari the size of a toy
7. For school: place 3-6 chair props as student desks in rows
8. For room: always include chair + boombox or lantern for ambiance
9. For space: astronaut should float (y: 1.5-3.0), not stand on ground
10. For city at night: ferrari headlights visible, birds still in sky
11. "reply" must describe the scene vividly in 1-2 sentences
12. Keep duration >= 8 seconds for a satisfying scene
13. If user says "sekolah/school": theme=school, add chairs as desks, toy_car on teacher's desk
14. If user says "ruangan/kamar/rumah": theme=room, add chair + boombox + lantern
15. If user says "angkasa/space": theme=space, timeOfDay=night, add floating astronaut
16. If user says "kota/city/jalan": theme=city, add ferrari + flamingo + parrot
17. If user says "malam": timeOfDay=night. If "sore/sunset": timeOfDay=sunset
18. Always infer the MOST interesting and contextually rich scene from the prompt
"""

# ── Request / Response ────────────────────────────────────────────────────────

class AIChatRequest(BaseModel):
    prompt: str
    history: list[dict] = []


class AIChatResponse(BaseModel):
    reply: str
    script: dict
    raw: str = ""


# ── AI call ───────────────────────────────────────────────────────────────────

def _call_ai_sync(messages: list[dict]) -> str:
    """Call AI via MOKOClient auto-switch gateway chain (runs in thread pool)."""
    from moko_bridge.moko_client import MOKOClient

    system = None
    user_parts = []
    for msg in messages:
        if msg["role"] == "system":
            system = msg["content"]
        else:
            user_parts.append(msg["content"])

    prompt = "\n".join(user_parts) if user_parts else ""

    client = MOKOClient()
    result = client.llm_generate(
        prompt=prompt,
        system_prompt=system,
        max_tokens=2000,
        temperature=0.75,
    )

    content = result.get("content", "")
    provider = result.get("provider", "unknown")
    if not content:
        raise RuntimeError(f"AI provider returned empty content (provider={provider})")
    if provider == "offline-fallback":
        raise RuntimeError("All AI gateways unavailable — offline fallback")
    logger.info("AI response via provider=%s client=%s", provider, result.get("client"))
    return content


async def _call_openrouter(messages: list[dict]) -> str:
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, _call_ai_sync, messages)



def _extract_json(raw: str) -> dict:
    """Extract JSON from AI response, even if it has extra text."""
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        pass
    # Find outermost { ... } block
    start = raw.find("{")
    end = raw.rfind("}") + 1
    if start != -1 and end > start:
        try:
            return json.loads(raw[start:end])
        except json.JSONDecodeError:
            pass
    raise ValueError(f"No valid JSON in response: {raw[:300]}")


def _build_fallback_script(prompt: str) -> dict:
    """Context-aware fallback scene builder when AI fails."""
    p = prompt.lower()

    # ── Detect environment ────────────────────────────────────────────────
    theme = "city"
    time_of_day = "noon"

    if any(w in p for w in ["ruangan", "kamar", "rumah", "room", "indoor", "dalam"]):
      theme = "room"
    elif any(w in p for w in ["sekolah", "kelas", "school", "classroom"]):
      theme = "school"
    elif any(w in p for w in ["angkasa", "luar angkasa", "space", "planet", "bintang"]):
      theme = "space"
      time_of_day = "night"
    elif any(w in p for w in ["gurun", "pasir", "desert", "padang pasir"]):
      theme = "desert"
    elif any(w in p for w in ["hutan", "rimba", "pohon", "forest"]):
      theme = "forest"
    elif any(w in p for w in ["laut", "pantai", "bawah air", "ocean", "air"]):
      theme = "ocean"
    elif any(w in p for w in ["cyberpunk", "neon", "futuristik"]):
      theme = "cyberpunk"
      time_of_day = "night"
    elif any(w in p for w in ["kutub", "salju", "dingin", "arctic", "es", "ice"]):
      theme = "arctic"
    elif any(w in p for w in ["gunung berapi", "api", "lahar", "lava", "volcano"]):
      theme = "volcano"
    elif any(w in p for w in ["studio", "syuting", "shooting", "soundstage", "set", "kamera", "film"]):
      theme = "studio"


    if any(w in p for w in ["malam", "night", "gelap"]):
      time_of_day = "night"
    elif any(w in p for w in ["sore", "sunset", "magrib"]):
      time_of_day = "sunset"

    # ── Detect character ──────────────────────────────────────────────────
    char_type = "stickman"
    if any(w in p for w in ["robot", "mecha", "cyber"]):
        char_type = "robot"
    elif any(w in p for w in ["soldier", "tentara", "prajurit", "militer"]):
        char_type = "soldier"
    elif any(w in p for w in ["dance", "tari", "menari", "michelle", "penari"]):
        char_type = "michelle"

    # ── Build timeline ────────────────────────────────────────────────────
    timeline = [{"t": 0.0, "action": "idle", "duration": 1.0}]
    t = 1.0

    def add(action, dur, **kw):
        nonlocal t
        clip = {"t": round(t, 2), "action": action, "duration": dur}
        clip.update(kw)
        timeline.append(clip)
        t += dur

    if any(w in p for w in ["jalan", "walk", "berjalan"]):
        add("walk", 4.0, direction="right")
    if any(w in p for w in ["lari", "run", "berlari"]):
        add("run", 3.0, direction="right")
    if any(w in p for w in ["lompat", "jump"]):
        add("jump", 1.2)
        add("idle", 0.5)
    if any(w in p for w in ["dance", "tari", "menari"]):
        add("dance", 5.0)
    if any(w in p for w in ["wave", "lambaikan", "halo", "dadah"]):
        add("wave", 2.5)
    if any(w in p for w in ["pikir", "think", "berpikir"]):
        add("think", 2.5)
    if any(w in p for w in ["punch", "pukul", "tinju", "tendang"]):
        add("punch", 0.6)
        add("idle", 0.3)
        add("punch", 0.6)

    if t <= 1.5:
        add("walk", 3.0, direction="right")
        add("think", 2.0)
        add("wave", 2.0)

    add("idle", 1.0)
    total = round(t, 2)

    # ── Scene-specific props ──────────────────────────────────────────────
    props = []
    if theme == "city":
        props = [
            {"id": "f1", "type": "model_prop", "modelId": "ferrari",
             "position": [3.5, 0, -1.5], "scale": 1.0, "rotation": [0, -0.3, 0],
             "start": 0, "duration": total},
            {"id": "b1", "type": "model_prop", "modelId": "flamingo",
             "position": [-2.0, 4.5, -3.0], "scale": 0.035, "rotation": [0, 0, 0],
             "start": 0, "duration": total},
            {"id": "b2", "type": "model_prop", "modelId": "parrot",
             "position": [1.5, 5.5, -4.0], "scale": 0.03, "rotation": [0, 0, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "room":
        props = [
            {"id": "c1", "type": "model_prop", "modelId": "chair",
             "position": [-2.5, 0, -3.5], "scale": 1.8, "rotation": [0, 0.3, 0],
             "start": 0, "duration": total},
            {"id": "bb1", "type": "model_prop", "modelId": "boombox",
             "position": [3.0, 0, -2.5], "scale": 2.5, "rotation": [0, -0.2, 0],
             "start": 0, "duration": total},
            {"id": "l1", "type": "model_prop", "modelId": "lantern",
             "position": [0, 1.8, -4.5], "scale": 0.12, "rotation": [0, 0, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "school":
        props = [
            {"id": "d1", "type": "model_prop", "modelId": "chair",
             "position": [-3.0, 0, -2.5], "scale": 1.4, "rotation": [0, 0.05, 0],
             "start": 0, "duration": total},
            {"id": "d2", "type": "model_prop", "modelId": "chair",
             "position": [-1.0, 0, -2.5], "scale": 1.4, "rotation": [0, 0, 0],
             "start": 0, "duration": total},
            {"id": "d3", "type": "model_prop", "modelId": "chair",
             "position": [1.5, 0, -2.5], "scale": 1.4, "rotation": [0, -0.05, 0],
             "start": 0, "duration": total},
            {"id": "tc", "type": "model_prop", "modelId": "toy_car",
             "position": [0.3, 0, -5.0], "scale": 1.2, "rotation": [0, 0.3, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "space":
        props = [
            {"id": "a1", "type": "model_prop", "modelId": "astronaut",
             "position": [-3.0, 2.0, -3.0], "scale": 1.2, "rotation": [0.2, 0.5, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "desert":
        props = [
            {"id": "h1", "type": "model_prop", "modelId": "horse",
             "position": [4.0, 0, -3.0], "scale": 0.004, "rotation": [0, -0.5, 0],
             "start": 0, "duration": total},
            {"id": "f2", "type": "model_prop", "modelId": "fox",
             "position": [-3.5, 0, -1.8], "scale": 0.02, "rotation": [0, 0.4, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "forest":
        props = [
            {"id": "f3", "type": "model_prop", "modelId": "fox",
             "position": [2.5, 0, -2.0], "scale": 0.02, "rotation": [0, -0.3, 0],
             "start": 0, "duration": total},
            {"id": "h2", "type": "model_prop", "modelId": "horse",
             "position": [-4.5, 0, -5.0], "scale": 0.004, "rotation": [0, 0.5, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "ocean":
        props = [
            {"id": "d4", "type": "model_prop", "modelId": "duck",
             "position": [2.0, 0.5, -2.0], "scale": 0.008, "rotation": [0, 0, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "cyberpunk":
        props = [
            {"id": "h3", "type": "model_prop", "modelId": "damaged_helmet",
             "position": [-2.0, 0.3, -2.0], "scale": 0.8, "rotation": [0.2, 0.4, 0],
             "start": 0, "duration": total},
            {"id": "bb2", "type": "model_prop", "modelId": "boombox",
             "position": [2.5, 0, -1.8], "scale": 2.2, "rotation": [0, -0.4, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "arctic":
        props = [
            {"id": "f4", "type": "model_prop", "modelId": "fox",
             "position": [-3.0, 0, -2.5], "scale": 0.02, "rotation": [0, 0.5, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "volcano":
        props = [
            {"id": "dh2", "type": "model_prop", "modelId": "damaged_helmet",
             "position": [0, 0.5, -3.0], "scale": 1.0, "rotation": [0, 0, 0],
             "start": 0, "duration": total},
        ]
    elif theme == "studio":
        props = [
            {"id": "c1", "type": "model_prop", "modelId": "chair",
             "position": [-3.0, 0, -2.0], "scale": 1.8, "rotation": [0, 0.3, 0],
             "start": 0, "duration": total},
            {"id": "bb1", "type": "model_prop", "modelId": "boombox",
             "position": [3.0, 0, -2.5], "scale": 2.5, "rotation": [0, -0.4, 0],
             "start": 0, "duration": total},
            {"id": "l1", "type": "model_prop", "modelId": "lantern",
             "position": [-2.0, 1.8, -4.5], "scale": 0.12, "rotation": [0, 0, 0],
             "start": 0, "duration": total},
            {"id": "h3", "type": "model_prop", "modelId": "damaged_helmet",
             "position": [0.5, 0.3, -3.0], "scale": 0.8, "rotation": [0.2, 0.4, 0],
             "start": 0, "duration": total},
        ]


    return {
        "duration": total,
        "characterType": char_type,
        "environment": {"theme": theme, "timeOfDay": time_of_day},
        "cameraKeyframes": [
            {"t": 0.0, "position": [0, 5, 12], "lookAt": [0, 1.0, 0]},
            {"t": total * 0.5, "position": [0, 3, 7], "lookAt": [0, 1.5, 0]},
            {"t": total * 0.85, "position": [1.5, 2, 5.5], "lookAt": [0, 1.0, 0]},
        ],
        "characters": [{"id": "hero", "timeline": timeline}],
        "elements": props,
        "reply": f"Adegan '{theme}' siap dimainkan! (offline mode)",
    }


# ── Route ─────────────────────────────────────────────────────────────────────

@router.post("/ai-chat")
async def ai_chat(request: AIChatRequest):
    """Call OpenRouter AI to generate a complete, director-quality animation scene."""
    messages = [{"role": "system", "content": SYSTEM_PROMPT}]

    # Include up to 8 prior messages for richer context continuity
    for msg in request.history[-8:]:
        messages.append(msg)

    messages.append({"role": "user", "content": request.prompt})

    raw = ""
    try:
        raw = await _call_openrouter(messages)
        script = _extract_json(raw)

        # Normalize required fields
        script.setdefault("duration", 10.0)
        script.setdefault("characterType", "stickman")
        script.setdefault("environment", {"theme": "city", "timeOfDay": "noon"})
        script.setdefault("cameraKeyframes", [
            {"t": 0.0, "position": [0, 5, 12], "lookAt": [0, 1.0, 0]},
            {"t": script["duration"] * 0.5, "position": [0, 3, 7], "lookAt": [0, 1.5, 0]},
        ])
        script.setdefault("characters", [{"id": "hero", "timeline": [
            {"t": 0.0, "action": "idle", "duration": 2.0},
            {"t": 2.0, "action": "walk", "duration": 4.0, "direction": "right"},
            {"t": 6.0, "action": "idle", "duration": 2.0},
        ]}])
        script.setdefault("elements", [])

        reply = script.pop("reply", "Adegan siap! ▶ Klik Start Preview untuk menontonnya.")

        return {"status": "ok", "reply": reply, "script": script, "raw": raw}

    except Exception as e:
        err_str = str(e)[:120]
        logger.error("AI director error: %s — using fallback script", err_str)
        script = _build_fallback_script(request.prompt)
        reply = script.pop("reply", "")
        tag = " (offline mode)"
        if "429" in err_str or "Rate limited" in err_str or "quota" in err_str:
            tag = " (API rate limited — offline mode)"
        elif "timeout" in err_str.lower() or "Timeout" in err_str:
            tag = " (timeout — offline mode)"
        return {"status": "ok", "reply": reply + tag, "script": script, "raw": err_str}



# ── AI Storyboard System Prompt ──────────────────────────────────────────────

STORY_SYSTEM_PROMPT = """You are an expert movie director AI. The user will give you a story topic.
You must break down this topic into a storyboard of exactly 4 connected sequential scenes.
The total duration of all scenes combined should be around 40 to 60 seconds (each scene ~10 to 15 seconds).

For each scene, you must generate:
1. "sceneId": unique identifier (e.g. "scene_1", "scene_2")
2. "title": a brief title of the scene
3. "narration": 1-2 sentence voiceover script that will be shown as subtitles (in Indonesian if prompt is in Indonesian, otherwise English)
4. "duration": float (10.0 to 15.0)
5. "characterType": "stickman" | "soldier" | "robot" | "michelle" for the main character in this scene
6. "environment": {"theme": "city" | "room" | "space" | "school" | "desert" | "forest" | "ocean" | "cyberpunk" | "arctic" | "volcano" | "studio", "timeOfDay": "noon" | "sunset" | "night"}
7. "cameraTarget": "hero" (focus on character) or "none" (default) or the id of a prop (e.g. "siput_prop") to dynamically focus the camera on that target!
8. "cameraKeyframes": list of camera positions
9. "characters": standard timeline of actions for the main character (covering the full duration with NO gaps)
10. "elements": list of 3D props (type "model_prop" with modelId, position, scale, rotation, start, duration).

CHARACTER/PROP MAPPINGS FOR STORIES (e.g. Snail vs Rabbit):
- Snail / Siput: use modelId "duck" (Yellow duck) or "fox" (sitting) or "water_bottle"
- Rabbit / Kelinci: use modelId "fox" (Cute fox) or "horse" (running) or "robot"
- Always populate scenes with relevant scenery props (cacti, trees, cars, etc.)

RESPONSE FORMAT (return ONLY valid JSON):
{
  "title": "<overall story title>",
  "scenes": [
    {
      "sceneId": "scene_1",
      "title": "<scene title>",
      "narration": "<subtitle text>",
      "duration": 12.0,
      "characterType": "robot",
      "environment": { "theme": "forest", "timeOfDay": "noon" },
      "cameraTarget": "hero",
      "cameraKeyframes": [
        {"t": 0.0, "position": [0, 5, 12], "lookAt": [0, 1.0, 0]},
        {"t": 6.0, "position": [0, 3, 7], "lookAt": [0, 1.5, 0]}
      ],
      "characters": [{"id": "hero", "timeline": [{"t": 0.0, "action": "idle", "duration": 2.0}, {"t": 2.0, "action": "walk", "duration": 10.0, "direction": "right"}]}],
      "elements": [
        {"id": "siput_prop", "type": "model_prop", "modelId": "duck", "position": [-3.0, 0.0, 0.5], "scale": 0.008, "start": 0.0, "duration": 12.0}
      ]
    },
    ...
  ]
}
"""

def _build_fallback_story(prompt: str) -> dict:
    """Fallback storyboard generator if AI fails/timeouts."""
    p = prompt.lower()
    return {
        "title": "Cerita Balapan Siput vs Kelinci",
        "scenes": [
            {
                "sceneId": "scene_1",
                "title": "Babak 1: Tantangan Balapan",
                "narration": "Kelinci yang sombong menantang Siput untuk berlomba lari di hutan pada siang hari.",
                "duration": 10.0,
                "characterType": "robot",
                "environment": {"theme": "forest", "timeOfDay": "noon"},
                "cameraTarget": "hero",
                "cameraKeyframes": [
                    {"t": 0.0, "position": [0, 5, 12], "lookAt": [0, 1.0, 0]},
                    {"t": 5.0, "position": [0, 3, 7], "lookAt": [0, 1.5, 0]},
                ],
                "characters": [{"id": "hero", "timeline": [
                    {"t": 0.0, "action": "idle", "duration": 2.0},
                    {"t": 2.0, "action": "wave", "duration": 3.0},
                    {"t": 5.0, "action": "idle", "duration": 5.0},
                ]}],
                "elements": [
                    {"id": "siput_prop", "type": "model_prop", "modelId": "duck", "position": [-2.0, 0.0, 0.5], "scale": 0.008, "start": 0, "duration": 10.0},
                    {"id": "fox_prop", "type": "model_prop", "modelId": "fox", "position": [3.0, 0, -2], "scale": 0.02, "start": 0, "duration": 10.0}
                ]
            },
            {
                "sceneId": "scene_2",
                "title": "Babak 2: Kelinci Tertidur",
                "narration": "Sore pun tiba, Kelinci tertidur lelap di bawah pohon karena terlalu percaya diri.",
                "duration": 10.0,
                "characterType": "robot",
                "environment": {"theme": "forest", "timeOfDay": "sunset"},
                "cameraTarget": "siput_prop",
                "cameraKeyframes": [
                    {"t": 0.0, "position": [-2, 2, 6], "lookAt": [-2, 0.5, 0.5]},
                    {"t": 6.0, "position": [0, 3, 7], "lookAt": [0, 1.0, 0]},
                ],
                "characters": [{"id": "hero", "timeline": [
                    {"t": 0.0, "action": "idle", "duration": 10.0},
                ]}],
                "elements": [
                    {"id": "siput_prop", "type": "model_prop", "modelId": "duck", "position": [-1.0, 0.0, 0.5], "scale": 0.008, "start": 0, "duration": 10.0},
                ]
            },
            {
                "sceneId": "scene_3",
                "title": "Babak 3: Siput Mendekati Garis Finish",
                "narration": "Malam pun larut, Siput terus merangkak pantang menyerah mendekati garis finish.",
                "duration": 10.0,
                "characterType": "stickman",
                "environment": {"theme": "forest", "timeOfDay": "night"},
                "cameraTarget": "hero",
                "cameraKeyframes": [
                    {"t": 0.0, "position": [2, 2, 5], "lookAt": [2, 0.8, 0]},
                    {"t": 5.0, "position": [0, 3, 8], "lookAt": [0, 1.0, 0]},
                ],
                "characters": [{"id": "hero", "timeline": [
                    {"t": 0.0, "action": "walk", "duration": 8.0, "direction": "right"},
                    {"t": 8.0, "action": "idle", "duration": 2.0},
                ]}],
                "elements": [
                    {"id": "lantern_prop", "type": "model_prop", "modelId": "lantern", "position": [3.0, 0.5, -1.0], "scale": 0.12, "start": 0, "duration": 10.0},
                ]
            },
            {
                "sceneId": "scene_4",
                "title": "Babak 4: Siput Menang!",
                "narration": "Kelinci terbangun terlambat, dan Siput akhirnya memenangkan perlombaan tersebut!",
                "duration": 12.0,
                "characterType": "stickman",
                "environment": {"theme": "forest", "timeOfDay": "noon"},
                "cameraTarget": "hero",
                "cameraKeyframes": [
                    {"t": 0.0, "position": [0, 3, 6], "lookAt": [0, 1.0, 0]},
                    {"t": 6.0, "position": [2, 2, 5], "lookAt": [1.5, 1.2, 0]},
                ],
                "characters": [{"id": "hero", "timeline": [
                    {"t": 0.0, "action": "dance", "duration": 6.0},
                    {"t": 6.0, "action": "wave", "duration": 4.0},
                    {"t": 10.0, "action": "idle", "duration": 2.0},
                ]}],
                "elements": [
                    {"id": "fox_prop", "type": "model_prop", "modelId": "fox", "position": [-3.0, 0, -2], "scale": 0.02, "start": 0, "duration": 12.0},
                    {"id": "duck_prop", "type": "model_prop", "modelId": "duck", "position": [1.5, 0.0, 0.5], "scale": 0.008, "start": 0, "duration": 12.0},
                ]
            }
        ]
    }

class AIStoryRequest(BaseModel):
    prompt: str

@router.post("/ai-story")
async def ai_story(request: AIStoryRequest):
    """Call OpenRouter AI to generate a complete, structured 4-scene storyboard."""
    messages = [
        {"role": "system", "content": STORY_SYSTEM_PROMPT},
        {"role": "user", "content": request.prompt}
    ]

    try:
        raw = await _call_openrouter(messages)
        storyboard = _extract_json(raw)

        # Normalize required fields in generated storyboard
        storyboard.setdefault("title", "OpenCut Story")
        scenes = storyboard.get("scenes", [])
        if not isinstance(scenes, list) or len(scenes) == 0:
            raise ValueError("No scenes list found in response")

        for idx, scene in enumerate(scenes):
            scene.setdefault("sceneId", f"scene_{idx+1}")
            scene.setdefault("title", f"Scene {idx+1}")
            scene.setdefault("narration", "")
            scene.setdefault("duration", 10.0)
            scene.setdefault("characterType", "stickman")
            scene.setdefault("environment", {"theme": "city", "timeOfDay": "noon"})
            scene.setdefault("cameraTarget", "none")
            scene.setdefault("cameraKeyframes", [
                {"t": 0.0, "position": [0, 5, 12], "lookAt": [0, 1.0, 0]},
                {"t": scene["duration"] * 0.5, "position": [0, 3, 7], "lookAt": [0, 1.5, 0]}
            ])
            scene.setdefault("characters", [{"id": "hero", "timeline": [
                {"t": 0.0, "action": "idle", "duration": 2.0},
                {"t": 2.0, "action": "walk", "duration": 4.0, "direction": "right"},
                {"t": 6.0, "action": "idle", "duration": scene["duration"] - 6.0}
            ]}])
            scene.setdefault("elements", [])

        return {"status": "ok", "storyboard": storyboard}

    except Exception as e:
        logger.error("AI story generation error: %s", e)
        fallback = _build_fallback_story(request.prompt)
        return {"status": "ok", "storyboard": fallback, "fallback": True}

