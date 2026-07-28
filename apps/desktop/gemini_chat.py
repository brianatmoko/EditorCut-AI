#!/usr/bin/env python3
"""
gemini_chat.py — AI Story Director Conversation Engine.

Usage:
  python3 gemini_chat.py <mode> <input_json>

Modes:
  "greet"    → Returns initial greeting message
  "reply"    → Given chat history + user message, returns AI reply + next state
  "generate" → Given confirmed plan JSON + past episodes, calls gemini_director to generate movie

Input JSON schema for "reply" mode:
{
  "user_message": "saya ingin cerita polisi",
  "ai_state": "waiting_for_story_idea",
  "story_context": {
    "story_idea": "",
    "duration_minutes": 1,
    "proposed_setting": "",
    "proposed_characters": [],
    "episode_synopsis": "",
    "past_episodes": []
  }
}

Output JSON for "reply" mode:
{
  "ai_reply": "Bagus! Berapa menit per episode?...",
  "next_state": "waiting_for_duration",
  "story_context": { ...updated... },
  "quick_replies": ["1 menit", "2 menit", "30 detik"]
}

Input JSON for "generate" mode:
{
  "story_context": { ... confirmed plan ... },
  "past_episodes": [{ "part_number": 1, "title": "...", "summary": "..." }]
}

Output for "generate" mode: CinematicMovie JSON (same as gemini_director.py)
"""
import sys
import json
import subprocess
import os
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent))

from moko_bridge.moko_client import MOKOClient

AVAILABLE_SETTINGS = ["city", "cyberpunk", "forest", "room", "school", "space"]
AVAILABLE_CHARACTERS = {
    "police_1": "Kapten SWAT",
    "police_2": "Petugas Polisi",
    "police_3": "Patroli",
    "terrorist_1": "Komandan Teroris",
    "terrorist_2": "Teroris (RPG)",
    "terrorist_3": "Teroris Penjaga",
    "chibi_summer": "Anak Kecil (Hijau)",
    "chibi_autumn": "Anak Kecil",
    "chibi_winter": "Anak Kecil (Musim Dingin)",
}

def call_gemini(prompt_text: str) -> dict:
    moko = MOKOClient()
    result = moko.llm_generate(
        prompt=prompt_text,
        max_tokens=4000,
        temperature=0.85,
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
    # Try direct parse first
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        pass
    # Fallback: extract first JSON object from text (handles preamble/prose around JSON)
    import re
    match = re.search(r'\{.*\}', raw, re.DOTALL)
    if match:
        try:
            return json.loads(match.group(0))
        except json.JSONDecodeError:
            pass
    raise RuntimeError(f"Could not parse JSON from {provider}: {raw[:200]}...")


def handle_greet():
    return {
        "ai_reply": "Halo! 🎬 Saya adalah AI Director kamu.\n\nCeritakan ide cerita yang ingin kamu buat! Bisa tentang apa saja — aksi, petualangan, misteri...",
        "next_state": "waiting_for_story_idea",
        "story_context": {},
        "quick_replies": ["Polisi vs Teroris", "Penculikan Anak", "Kejar-kejaran Mobil"]
    }


def handle_reply(data: dict) -> dict:
    user_message = data.get("user_message", "").strip()
    ai_state = data.get("ai_state", "waiting_for_story_idea")
    ctx = data.get("story_context", {})
    past_episodes = ctx.get("past_episodes", [])

    # State machine
    if ai_state == "waiting_for_story_idea":
        ctx["story_idea"] = user_message
        # Ask for duration
        reply = f"Menarik! 🎯 Cerita tentang \"{user_message}\" kedengarannya seru.\n\nBerapa menit per episode? Saya sarankan **1 menit** agar cepat dan padat."
        return {
            "ai_reply": reply,
            "next_state": "waiting_for_duration",
            "story_context": ctx,
            "quick_replies": ["30 detik", "1 menit", "2 menit"]
        }

    elif ai_state == "waiting_for_duration":
        # Parse duration from user message
        msg_lower = user_message.lower()
        if "30" in msg_lower or "tiga puluh" in msg_lower:
            duration = 0.5
        elif "2 menit" in msg_lower or "dua" in msg_lower:
            duration = 2
        else:
            duration = 1  # default 1 minute
        ctx["duration_minutes"] = duration

        # Ask Gemini to propose a setting and characters based on the story idea
        story_idea = ctx.get("story_idea", "cerita aksi")
        past_ctx = ""
        if past_episodes:
            past_ctx = f"\nEpisode sebelumnya: {json.dumps(past_episodes, ensure_ascii=False)}"

        prompt = f"""You are an AI movie director assistant. Based on the story idea below, propose:
1. The best background/setting from: {AVAILABLE_SETTINGS}
2. 2–4 best characters from available skins
3. A short exciting episode synopsis (2 sentences in Indonesian)

Story idea: {story_idea}{past_ctx}

Return ONLY valid JSON:
{{
  "proposed_setting": "city",
  "proposed_characters": [
    {{"skin_id": "police_1", "name": "Kapten SWAT", "role": "Protagonis"}},
    {{"skin_id": "terrorist_1", "name": "Komandan Teroris", "role": "Antagonis"}}
  ],
  "episode_synopsis": "Kapten SWAT mengejar teroris di jalan raya kota. Pertarungan sengit terjadi di persimpangan!",
  "episode_title": "Operasi Tangkap"
}}"""

        try:
            proposal = call_gemini(prompt)
            ctx["proposed_setting"] = proposal.get("proposed_setting", "city")
            ctx["proposed_characters"] = proposal.get("proposed_characters", [])
            ctx["episode_synopsis"] = proposal.get("episode_synopsis", "")
            ctx["episode_title"] = proposal.get("episode_title", "Episode")

            setting_name = ctx["proposed_setting"].title()
            chars_text = ", ".join([f"{c['name']} ({c['role']})" for c in ctx["proposed_characters"]])
            dur_text = "1 menit" if duration == 1 else ("30 detik" if duration == 0.5 else f"{duration} menit")

            reply = (
                f"Saya menganalisis ide cerita kamu... 🔍\n\n"
                f"📍 **Latar**: {setting_name}\n"
                f"👥 **Karakter**: {chars_text}\n"
                f"⏱️ **Durasi**: {dur_text}\n\n"
                f"📖 **Sinopsis**: {ctx['episode_synopsis']}\n\n"
                f"Apakah kamu setuju dengan rencana ini?"
            )
            return {
                "ai_reply": reply,
                "next_state": "waiting_for_setting_approval",
                "story_context": ctx,
                "quick_replies": ["✅ Setuju, Generate!", "🔄 Ganti Latar", "👥 Ganti Karakter"]
            }
        except Exception as e:
            return {
                "ai_reply": f"Maaf, ada gangguan koneksi AI ({e}). Coba lagi?",
                "next_state": "waiting_for_duration",
                "story_context": ctx,
                "quick_replies": ["Coba lagi"]
            }

    elif ai_state == "waiting_for_setting_approval":
        msg_lower = user_message.lower()
        if "ganti latar" in msg_lower or "latar" in msg_lower:
            # Cycle to next setting
            current = ctx.get("proposed_setting", "city")
            idx = AVAILABLE_SETTINGS.index(current) if current in AVAILABLE_SETTINGS else 0
            ctx["proposed_setting"] = AVAILABLE_SETTINGS[(idx + 1) % len(AVAILABLE_SETTINGS)]
            setting_name = ctx["proposed_setting"].title()
            chars_text = ", ".join([f"{c['name']} ({c['role']})" for c in ctx.get("proposed_characters", [])])
            return {
                "ai_reply": f"Saya ganti latarnya! 🔄\n\n📍 **Latar baru**: {setting_name}\n👥 **Karakter**: {chars_text}\n\n📖 {ctx.get('episode_synopsis', '')}\n\nBagaimana sekarang?",
                "next_state": "waiting_for_setting_approval",
                "story_context": ctx,
                "quick_replies": ["✅ Setuju, Generate!", "🔄 Ganti Latar Lagi"]
            }
        elif "ganti karakter" in msg_lower or "karakter" in msg_lower:
            return {
                "ai_reply": "Fitur ganti karakter manual akan segera hadir! Untuk sekarang, saya akan gunakan karakter yang sudah dipilih AI.\n\nSetuju untuk lanjut generate?",
                "next_state": "waiting_for_setting_approval",
                "story_context": ctx,
                "quick_replies": ["✅ Setuju, Generate!"]
            }
        else:
            # User approved — ready to generate
            part_num = len(past_episodes) + 1
            title = ctx.get("episode_title", f"Episode {part_num}")
            return {
                "ai_reply": f"🎬 Siap! Saya akan generate **Part {part_num}: {title}** sekarang.\n\n⏳ Proses ini membutuhkan ~10 detik. Preview akan menampilkan loading...\n\n*Tekan Generate untuk mulai, atau pilih auto-continue untuk menulis naskah lengkap!*",
                "next_state": "ready_to_generate",
                "story_context": ctx,
                "quick_replies": [
                    "🎬 Generate Part Ini Saja",
                    "🎬 Auto-Generate 3 Episode",
                    "🎬 Auto-Generate 5 Episode"
                ]
            }

    elif ai_state == "episode_done":
        # User wants to continue with next episode
        past_episodes = ctx.get("past_episodes", [])
        part_num = len(past_episodes) + 1
        msg_lower = user_message.lower()

        if any(w in msg_lower for w in ["ya", "lanjut", "next", "part", "episode", "iya"]):
            # Generate next episode context
            recap = ""
            if past_episodes:
                last = past_episodes[-1]
                recap = f"Di episode sebelumnya (Part {last['part_number']}): {last['title']} — {last['summary']}"

            return {
                "ai_reply": f"Lanjut ke **Part {part_num}**! 🎬\n\n{recap}\n\nCeritakan apa yang ingin terjadi di episode berikutnya? Atau biarkan AI yang memilih kelanjutan cerita!",
                "next_state": "waiting_for_story_idea",
                "story_context": ctx,
                "quick_replies": ["Lanjutkan cerita otomatis!", "Saya ingin ubah arah cerita"]
            }
        else:
            return {
                "ai_reply": "Baik! Kalau mau lanjut kapan saja, ketik 'lanjut' atau tekan tombol 'Part Berikutnya'. 🎬",
                "next_state": "episode_done",
                "story_context": ctx,
                "quick_replies": ["Lanjut Part Berikutnya"]
            }

    else:
        # Fallback
        return {
            "ai_reply": "Maaf, saya tidak mengerti. Coba mulai ulang dengan menceritakan ide cerita kamu!",
            "next_state": "waiting_for_story_idea",
            "story_context": ctx,
            "quick_replies": []
        }


def handle_generate(data: dict) -> None:
    """Call multi-agent orchestrator which plans, parallel-executes, and reviews."""
    ctx = data.get("story_context", {})
    past_episodes = ctx.get("past_episodes", [])

    story_idea = ctx.get("story_idea", "cerita aksi")
    setting = ctx.get("proposed_setting", "city")
    characters = ctx.get("proposed_characters", [])
    synopsis = ctx.get("episode_synopsis", "")
    title = ctx.get("episode_title", "Episode")
    duration = ctx.get("duration_minutes", 1)
    part_num = len(past_episodes) + 1

    agent_input = {
        "story_idea": story_idea,
        "duration_minutes": duration,
        "proposed_setting": setting,
        "proposed_characters": characters,
        "episode_title": title,
        "episode_synopsis": synopsis,
        "past_episodes": past_episodes,
    }

    # Call gemini_agents.py orchestrate mode via subprocess
    script_dir = os.path.dirname(os.path.abspath(__file__))
    agents_path = os.path.join(script_dir, "gemini_agents.py")

    try:
        result = subprocess.run(
            [sys.executable, agents_path, "orchestrate", json.dumps(agent_input, ensure_ascii=False)],
            capture_output=True, text=True, timeout=600
        )
    except subprocess.TimeoutExpired:
        print(json.dumps({"error": "Agent orchestrator timeout after 600s"}))
        sys.exit(1)
    except Exception as e:
        print(json.dumps({"error": f"Agent orchestrator invocation failed: {e}"}))
        sys.exit(1)

    if result.returncode != 0 or not result.stdout.strip():
        # Fallback to gemini_director.py
        _fallback_generate(data)
        return

    # Parse agent output and convert to CinematicMovie format
    try:
        agent_output = json.loads(result.stdout.strip())
        if "error" in agent_output:
            _fallback_generate(data)
            return
    except json.JSONDecodeError:
        _fallback_generate(data)
        return

    # Convert agent output (acts with entities/cameras) → CinematicMovie JSON
    movie = _agent_output_to_movie(agent_output, story_idea, setting, characters, int(duration * 60))
    print(json.dumps(movie, ensure_ascii=False))


def _agent_output_to_movie(agent_out: dict, story_idea: str, setting: str,
                           characters: list[dict], total_seconds: int) -> dict:
    """Convert agent orchestrator output to CinematicMovie JSON format."""
    # Agent output may have title in movie sub-dict or directly
    movie_meta = agent_out.get("movie", agent_out)
    title = movie_meta.get("title", agent_out.get("title", story_idea))
    acts_data = agent_out.get("acts", movie_meta.get("acts", []))

    acts = []
    for i, act_data in enumerate(acts_data):
        if not isinstance(act_data, dict) or "error" in act_data:
            continue

        entities = []
        for ent in act_data.get("entities", []):
            skin_id = ent.get("character_skin_id", "police_1")
            # Find character name
            char_name = skin_id
            for c in characters:
                if c.get("skin_id") == skin_id:
                    char_name = c.get("name", skin_id)
                    break

            action_timings = ent.get("action_timings", [])
            if not action_timings:
                action_timings = [{"action": "idle", "start": 0, "duration": act_data.get("duration_seconds", 15.0)}]

            entities.append({
                "character_skin_id": skin_id,
                "character_name": char_name,
                "pos_x": ent.get("pos_x", 0.0),
                "pos_y": ent.get("pos_y", 0.3),
                "facing": -1.0 if i > 0 else 1.0,
                "actions": ent.get("actions", ["idle"]),
                "action_timings": action_timings,
                "dialogue": ent.get("dialogue", {}),
            })

        # Camera shots with defaults
        camera_shots = act_data.get("camera_shots", [])
        if not camera_shots:
            camera_shots = [{
                "shot_type": "medium", "zoom": 1.0, "pan_x": 0.0, "pan_y": 0.0,
                "start_time": 0.0, "duration": act_data.get("duration_seconds", 15.0),
                "target_entity_index": 0, "tilt_angle": 0, "depth_of_field": 0.0,
                "shake_intensity": 0.0,
            }]

        act_duration = act_data.get("duration_seconds", 15.0)
        acts.append({
            "title": act_data.get("title", f"Act {i+1}"),
            "description": act_data.get("description", ""),
            "start_time": act_data.get("start_time", i * act_duration),
            "duration": act_duration,
            "background_mood": act_data.get("background_mood", "neutral"),
            "entities": entities,
            "camera_shots": camera_shots,
        })

    if not acts:
        return {"error": "No valid acts produced by agents"}

    return {
        "title": title,
        "story": story_idea,
        "setting": setting,
        "total_duration": total_seconds,
        "acts": acts,
        "_generated_by": "gemini_agents",
        "_agent_providers": agent_out.get("_agent_providers", {}),
    }


def _fallback_generate(data: dict) -> None:
    """Fallback to gemini_director.py if agent orchestrator fails."""
    ctx = data.get("story_context", {})
    past_episodes = ctx.get("past_episodes", [])

    story_idea = ctx.get("story_idea", "cerita aksi")
    setting = ctx.get("proposed_setting", "city")
    characters = ctx.get("proposed_characters", [])
    synopsis = ctx.get("episode_synopsis", "")
    title = ctx.get("episode_title", "Episode")
    duration = ctx.get("duration_minutes", 1)
    part_num = len(past_episodes) + 1

    chars_desc = ", ".join([f"{c['name']} ({c['skin_id']}) as {c['role']}" for c in characters])
    past_context_json = json.dumps(past_episodes, ensure_ascii=False)

    full_prompt = (
        f"Part {part_num}: {title}\n"
        f"Story idea: {story_idea}\n"
        f"Setting: {setting}\n"
        f"Characters: {chars_desc}\n"
        f"Synopsis: {synopsis}\n"
        f"Duration: {int(duration * 60)} seconds"
    )

    script_dir = os.path.dirname(os.path.abspath(__file__))
    director_path = os.path.join(script_dir, "gemini_director.py")

    try:
        result = subprocess.run(
            [sys.executable, director_path, full_prompt, past_context_json],
            capture_output=True, text=True, timeout=600
        )
    except subprocess.TimeoutExpired:
        print(json.dumps({"error": "Director timeout after 600s"}))
        sys.exit(1)
    except Exception as e:
        print(json.dumps({"error": f"Director invocation failed: {e}"}))
        sys.exit(1)

    if result.returncode != 0 or not result.stdout.strip():
        print(json.dumps({"error": f"Director failed: {result.stderr[:300]}"}))
        sys.exit(1)

    print(result.stdout.strip())


def main():
    if len(sys.argv) < 3:
        print(json.dumps({"error": "Usage: gemini_chat.py <mode> <input_json>"}))
        sys.exit(1)

    mode = sys.argv[1]
    try:
        data = json.loads(sys.argv[2])
    except Exception as e:
        print(json.dumps({"error": f"Invalid input JSON: {e}"}))
        sys.exit(1)

    try:
        if mode == "greet":
            print(json.dumps(handle_greet()))
        elif mode == "reply":
            print(json.dumps(handle_reply(data)))
        elif mode == "generate":
            handle_generate(data)  # prints movie JSON directly
        else:
            print(json.dumps({"error": f"Unknown mode: {mode}"}))
            sys.exit(1)
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)


if __name__ == "__main__":
    main()
