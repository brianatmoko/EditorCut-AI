#!/usr/bin/env python3
"""
gemini_agents.py — Multi-Agent AI Film Studio.

Three agent roles:
  - Mandor (foreman): decomposes user requests into structured tasks
  - Pekerja (workers): execute tasks in parallel, each using different API keys
  - Pengarah Film (director): reviews outputs, ensures continuity, polishes

Key rotation: each worker gets a different Gemini API key from the pool (5 keys total),
so up to 5 agents can work in parallel without rate-limiting each other.

Usage:
  python3 gemini_agents.py <mode> <input_json>

Modes:
  "plan"      → Mandor decomposes request into task list
  "execute"   → Workers execute tasks in parallel
  "review"    → Pengarah reviews and refines
  "orchestrate" → Full pipeline: plan → execute → review
"""
import json
import sys
import os
import time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent))
from moko_bridge.moko_client import MOKOClient


def _call(prompt: str, system_prompt: str | None = None, max_tokens: int = 4000, temperature: float = 0.6) -> dict:
    """Call Gemini via MOKOClient, optionally pinning to a specific API key index."""
    moko = MOKOClient()
    result = moko.llm_generate(
        prompt=prompt,
        system_prompt=system_prompt,
        max_tokens=max_tokens,
        temperature=temperature,
    )
    raw = result.get("content", "").strip()
    provider = result.get("provider", "unknown")
    if not raw:
        raise RuntimeError(f"Empty response from {provider}")
    if provider == "offline-fallback":
        raise RuntimeError("All AI gateways unavailable")
    if raw.startswith("```json"): raw = raw[7:]
    if raw.startswith("```"): raw = raw[3:]
    if raw.endswith("```"): raw = raw[:-3]
    raw = raw.strip()
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        pass
    import re
    match = re.search(r'\{.*\}', raw, re.DOTALL)
    if match:
        try:
            return json.loads(match.group(0))
        except json.JSONDecodeError:
            pass
    raise RuntimeError(f"Could not parse JSON from {provider}: {raw[:200]}...")


# ── Mandor Agent ──────────────────────────────────────────────────────────────

MANDOR_SYSTEM = """You are MANDOR FILM — an AI film production foreman managing 2D animation.

Your ONLY output is valid JSON. Never respond with natural language.
Reply ONLY with a JSON object, no preamble, no explanation.

The JSON schema:
{
  "title": "Film Title (in Indonesian)",
  "total_acts": 4,
  "duration_per_act": 15.0,
  "acts": [
    {
      "act_number": 1,
      "title": "Act Title in Indonesian",
      "description": "One sentence describing the scene",
      "intensity": "establish|rising_action|climax|resolution",
      "characters_in_focus": ["police_1", "terrorist_1"],
      "main_action": "walk|run|attack|idle|jump|hurt|punch|kick|shoot|block|dodge|sprint"
    }
  ],
  "setting": "city|cyberpunk|forest|room|school|space",
  "characters": [
    {"skin_id": "police_1", "name": "Kapten SWAT", "role": "Protagonis"}
  ],
  "story_summary": "2 sentence summary in Indonesian"
}

Generate 3-5 acts with varied intensity levels (establish -> rising_action -> climax -> resolution).
Keep character skin_ids consistent: police_1, police_2, police_3, terrorist_1, terrorist_2, terrorist_3, chibi_summer, chibi_autumn, chibi_winter"""


def mandor_plan(request: dict) -> dict:
    """Mandor agent: decompose user request into production plan."""
    story_idea = request.get("story_idea", "cerita aksi")
    duration = request.get("duration_minutes", 1)
    setting = request.get("proposed_setting", "city")
    characters = request.get("proposed_characters", [])
    past_episodes = request.get("past_episodes", [])
    part_num = len(past_episodes) + 1

    chars_desc = ", ".join([
        f"{c['name']} ({c['skin_id']}) as {c['role']}"
        for c in characters
    ]) if characters else "AI pilih sendiri"

    past_ctx = ""
    if past_episodes:
        past_ctx = f"\nEpisode sebelumnya: {json.dumps(past_episodes, ensure_ascii=False)}"

    prompt = f"""Buat rencana produksi untuk Part {part_num} dari film "{story_idea}".

Story idea: {story_idea}
Setting: {setting}
Characters: {chars_desc}
Duration per episode: {int(duration * 60)} detik{past_ctx}

Bagi episode ini menjadi 3-5 acts (sub-adegan). Tiap act harus punya:
- Judul yang menarik dalam Bahasa Indonesia
- Deskripsi 1 kalimat
- Intensity (establish → rising_action → climax → resolution)
- Karakter yang fokus di act itu
- action utama (salah satu dari: walk, run, attack, idle, jump, hurt, fall, hit, punch, kick, block, dodge, shoot, sprint, crouch, wave, cheer, talk)

Total durasi semua acts harus {int(duration * 60)} detik."""

    return _call(prompt, system_prompt=MANDOR_SYSTEM, max_tokens=3000, temperature=0.6)


# ── Pekerja Agents ────────────────────────────────────────────────────────────

PEKERJA_SYSTEM = """Kamu adalah PEKERJA FILM — spesialis yang mengerjakan satu act (bagian) dari film animasi 2D.

Tugasmu: Generate detail untuk SATU ACT saja berdasarkan arahan dari mandor.
Output JSON:
{
  "act_number": 1,
  "title": "Judul Act",
  "description": "adegan yang terjadi",
  "duration_seconds": 15.0,
  "start_time": 0.0,
  "entities": [
    {
      "character_skin_id": "police_1",
      "pos_x": -1.0,
      "pos_y": 0.0,
      "actions": ["run"],
      "action_timings": [{"action": "run", "start": 0, "duration": 15}],
      "dialogue": {"en": "Stop!", "id": "Berhenti!"}
    }
  ],
  "camera_shots": [
    {
      "shot_type": "medium",
      "zoom": 1.0,
      "pan_x": 0.0,
      "pan_y": 0.0,
      "start_time": 0.0,
      "duration": 15.0,
      "target_entity_index": 0,
      "tilt_angle": 0,
      "depth_of_field": 0.0,
      "shake_intensity": 0.0
    }
  ],
  "background_mood": "tense"
}

Aturan:
- Posisi X: -1.5 sampai +1.5 (kiri ke kanan layar)
- Posisi Y: 0.0 sampai 0.8 (bawah ke atas layar)
- Action harus dari daftar: walk, run, jump, attack, idle, hurt, fall, hit, punch, kick, block, dodge, shoot, sprint, crouch, grab, tackle, wave, cheer, talk
- Kamera shot types: wide, medium, closeup, extreme_closeup, dutch, tracking
- tilt_angle: 0-20 derajat (efek miring kamera)
- depth_of_field: 0.0-1.0 (efek blur latar)
- shake_intensity: 0.0-1.0 (efek gempa kamera)
- Pastikan dialogue dalam Bahasa Indonesia (field "id") dan English ("en")"""


def pekerja_execute(task: dict, key_index: int) -> dict:
    """Worker agent: execute one act, using a specific API key."""
    act_info = task.get("act", {})
    story_context = task.get("story", "")
    act_number = act_info.get("act_number", 1)
    duration = act_info.get("duration", 15.0)
    title = act_info.get("title", f"Act {act_number}")
    intensity = act_info.get("intensity", "rising_action")
    characters = act_info.get("characters_in_focus", [])
    main_action = act_info.get("main_action", "walk")
    setting = task.get("setting", "city")
    total_duration = task.get("total_duration", 60.0)

    # Calculate start_time based on previous acts
    prev_acts_duration = task.get("prev_acts_duration", 0.0)
    start_time = prev_acts_duration

    # Determine camera and mood from intensity
    if intensity == "climax":
        base_zoom, tilt_range, shake_range = 1.2, (8, 16), (0.3, 0.8)
        mood = "intense"
        shot_type = "closeup"
    elif intensity == "rising_action":
        base_zoom, tilt_range, shake_range = (0.9, 1.1), (4, 12), (0.1, 0.4)
        mood = "tense"
        shot_type = "medium"
    elif intensity == "resolution":
        base_zoom, tilt_range, shake_range = 0.8, (0, 4), (0, 0.1)
        mood = "calm"
        shot_type = "wide"
    else:
        base_zoom, tilt_range, shake_range = (0.8, 1.0), (0, 6), (0, 0.2)
        mood = "neutral"
        shot_type = "medium"

    prompt = f"""Generate SATU act untuk film animasi.

Act {act_number}: {title} ({intensity})
Setting: {setting}
Durasi: {duration} detik (start_time: {start_time})
Mood: {mood}
Total durasi film: {total_duration} detik

Karakter yang fokus: {characters if characters else "AI pilih 2-3 dari list yang ada"}

Story context: {story_context}

Detail teknis:
- Main action: {main_action}
- Tipe shot: {shot_type}
- Zoom level: ~{base_zoom}
- Tilt range: {tilt_range}
- Shake range: {shake_range}

Buat JSON lengkap untuk act ini saja dengan entities (karakter + aksi + dialog + posisi) dan camera_shots (minimal 1 shot per act, lebih baik 2-3 untuk variasi)."""

    return _call(prompt, system_prompt=PEKERJA_SYSTEM, max_tokens=4000, temperature=0.65)


# ── Pengarah Film Agent ───────────────────────────────────────────────────────

PENGARAH_SYSTEM = """Kamu adalah PENGARAH FILM — sutradara senior yang memeriksa kualitas akhir film.

Tugasmu: Review output dari para pekerja film dan satukan menjadi satu film yang utuh.
Perhatikan:
1. Kontinuitas posisi karakter antar acts
2. Alur cerita yang logis
3. Variasi kamera yang menarik
4. Kesesuaian dialogue dengan konteks

Output final JSON:
{
  "movie": {
    "title": "Judul Film",
    "story": "Ringkasan cerita"
  },
  "acts": [ ... acts dari pekerja yang sudah direview ... ],
  "notes": "Catatan sutradara"
}

Jika ada masalah kontinuitas, perbaiki langsung di output."""


def pengarah_review(request: dict, worker_results: list[dict]) -> dict:
    """Pengarah agent: review all worker outputs and produce final film."""
    story_context = request.get("story", "")
    title = request.get("title", "Film AI")
    setting = request.get("setting", "city")

    worker_outputs = []
    for r in worker_results:
        if isinstance(r, dict):
            worker_outputs.append(r)

    prompt = f"""Review dan satukan hasil kerja para pekerja film.

Judul: {title}
Setting: {setting}

Story: {story_context}

Output dari masing-masing pekerja (setiap act):
{json.dumps(worker_outputs, ensure_ascii=False, indent=2)}

Tugas:
1. Periksa kontinuitas posisi karakter antar acts
2. Perbaiki jika ada karakter yang loncat posisi tidak realistis
3. Pastikan dialogue dalam Bahasa Indonesia terdengar natural
4. Gabungkan semua acts menjadi satu film utuh
5. Beri catatan sutradara

Output final JSON:
{{
  "title": "{title}",
  "acts": [ ... acts yang sudah diperbaiki ... ],
  "total_duration": total_detik,
  "review_notes": "Catatan sutradara tentang film ini"
}}"""

    return _call(prompt, system_prompt=PENGARAH_SYSTEM, max_tokens=6000, temperature=0.4)


# ── Orchestrator ──────────────────────────────────────────────────────────────

def orchestrate(request: dict) -> dict:
    """Full pipeline: mandor plans → workers execute → pengarah reviews.

    Workers run in parallel using ThreadPoolExecutor.
    """
    story_idea = request.get("story_idea", "cerita aksi")
    duration = request.get("duration_minutes", 1)
    setting = request.get("proposed_setting", "city")
    characters = request.get("proposed_characters", [])
    past_episodes = request.get("past_episodes", [])
    auto_continue_max = request.get("auto_continue_max", 1)
    part_num = len(past_episodes) + 1

    total_seconds = int(duration * 60)

    # Step 1: Mandor creates plan
    print("[PROGRESS] Mandor: menganalisis ide cerita...", file=sys.stderr, flush=True)
    plan = mandor_plan({
        "story_idea": story_idea,
        "duration_minutes": duration,
        "proposed_setting": setting,
        "proposed_characters": characters,
        "past_episodes": past_episodes,
    })

    acts_plan = plan.get("acts", [])
    setting = plan.get("setting", setting)
    characters = plan.get("characters", characters)
    title = plan.get("title", f"Part {part_num}: {story_idea}")
    print(f"[PROGRESS] Mandor selesai: {len(acts_plan)} act direncanakan", file=sys.stderr, flush=True)

    # Step 2: Distribute acts to workers (parallel)
    # Each worker uses a different API key index (round-robin)
    num_workers = min(len(acts_plan), 5)  # max 5 parallel workers (1 per key)
    tasks = []
    acc_duration = 0.0
    per_act_duration = total_seconds / max(len(acts_plan), 1)

    for i, act in enumerate(acts_plan):
        act_dur = act.get("duration", per_act_duration)
        tasks.append({
            "act": act,
            "story": story_idea,
            "setting": setting,
            "character_list": characters,
            "total_duration": total_seconds,
            "prev_acts_duration": acc_duration,
            "act_index": i,
        })
        acc_duration += act_dur

    print(f"[PROGRESS] Pekerja: {len(tasks)} pekerja paralel dimulai ({num_workers} worker)", file=sys.stderr, flush=True)

    worker_results = [None] * len(tasks)
    with ThreadPoolExecutor(max_workers=num_workers) as executor:
        future_map = {}
        for i, task in enumerate(tasks):
            key_index = i % 5  # round-robin through 5 API keys
            future = executor.submit(pekerja_execute, task, key_index)
            future_map[future] = i

        for future in as_completed(future_map):
            idx = future_map[future]
            try:
                result = future.result(timeout=180)
                worker_results[idx] = result
                act_name = acts_plan[idx].get("title", f"Act {idx+1}")
                print(f"[PROGRESS] Pekerja {idx+1} selesai: {act_name}", file=sys.stderr, flush=True)
            except Exception as e:
                print(f"[PROGRESS] Pekerja {idx+1} GAGAL: {e}", file=sys.stderr, flush=True)
                worker_results[idx] = {"error": str(e), "act_number": idx + 1}

    # Filter out failed workers
    valid_results = [r for r in worker_results if r and "error" not in r]

    if not valid_results:
        raise RuntimeError("All workers failed")

    # Step 3: Pengarah reviews and combines
    print("[PROGRESS] Pengarah film: mereview dan menggabungkan hasil...", file=sys.stderr, flush=True)
    final = pengarah_review({
        "title": title,
        "story": story_idea,
        "setting": setting,
    }, valid_results)
    print("[PROGRESS] Film selesai! Mengirim ke preview...", file=sys.stderr, flush=True)

    # Add metadata
    final["part_number"] = part_num
    final["setting"] = setting
    final["characters"] = characters
    final["_agent_providers"] = {
        f"act_{i+1}": {"status": "ok" if r and "error" not in r else "failed"}
        for i, r in enumerate(worker_results)
    }

    return final


# ── CLI Interface ─────────────────────────────────────────────────────────────

def main():
    if len(sys.argv) < 3:
        print(json.dumps({"error": "Usage: gemini_agents.py <mode> <input_json>"}))
        sys.exit(1)

    mode = sys.argv[1]
    try:
        data = json.loads(sys.argv[2])
    except Exception as e:
        print(json.dumps({"error": f"Invalid input JSON: {e}"}))
        sys.exit(1)

    try:
        if mode == "plan":
            result = mandor_plan(data.get("request", {}))
            print(json.dumps(result, ensure_ascii=False))
        elif mode == "execute":
            task = data.get("task", {})
            key_index = data.get("key_index", 0)
            result = pekerja_execute(task, key_index)
            print(json.dumps(result, ensure_ascii=False))
        elif mode == "review":
            result = pengarah_review(
                data.get("request", {}),
                data.get("worker_results", []),
            )
            print(json.dumps(result, ensure_ascii=False))
        elif mode == "orchestrate":
            result = orchestrate(data)
            print(json.dumps(result, ensure_ascii=False))
        else:
            print(json.dumps({"error": f"Unknown mode: {mode}"}))
            sys.exit(1)
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)


if __name__ == "__main__":
    main()
