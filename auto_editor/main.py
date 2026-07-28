#!/usr/bin/env python3
"""OpenCut AI Auto-Editor — CLI entry point."""

from __future__ import annotations
import argparse
import sys
from pathlib import Path
from typing import Optional

sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor import __version__
from auto_editor.config.settings_loader import load_config, save_config
from auto_editor.orchestrator.intent_router import IntentRouter


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="opencut-ai-auto",
        description="OpenCut AI Auto-Editor — Token-efficient video editing automation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  opencut-ai-auto edit ./footage/ --script script.txt --output result.mp4
  opencut-ai-auto edit ./footage/ --mode offline
  opencut-ai-auto batch ./projects/ --format mp4
  opencut-ai-auto voiceover --text narasi.txt --voice id
  opencut-ai-auto subtitle video.mp4 --language id
  opencut-ai-auto estimate ./footage/ --script script.txt
        """
    )
    parser.add_argument("--version", action="version", version=f"opencut-ai-auto v{__version__}")
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    edit_parser = subparsers.add_parser("edit", help="Auto-edit video from footage directory")
    edit_parser.add_argument("footage", type=str, help="Path to directory containing footage")
    edit_parser.add_argument("--script", "-s", type=str, help="Path to script/narration file")
    edit_parser.add_argument("--output", "-o", type=str, default="./output.mp4", help="Output video path")
    edit_parser.add_argument("--mode", "-m", type=str, choices=["offline", "hybrid", "cloud"], default=None)
    edit_parser.add_argument("--style", type=str, help="Editing style (cinematic, vlog, tutorial)")
    edit_parser.add_argument("--duration", "-d", type=int, help="Target duration in seconds")
    edit_parser.add_argument("--prompt", "-p", type=str, help="Natural language editing instruction")

    batch_parser = subparsers.add_parser("batch", help="Batch render multiple projects")
    batch_parser.add_argument("projects_dir", type=str, help="Directory containing project folders")
    batch_parser.add_argument("--format", "-f", type=str, default="mp4", choices=["mp4", "mov", "webm"])
    batch_parser.add_argument("--resolution", "-r", type=str, default="1080p", choices=["720p", "1080p", "4k"])
    batch_parser.add_argument("--mode", "-m", type=str, choices=["offline", "hybrid", "cloud"], default=None)

    vo_parser = subparsers.add_parser("voiceover", help="Generate voiceover audio from text")
    vo_parser.add_argument("--text", "-t", type=str, required=True, help="Path to text/script file")
    vo_parser.add_argument("--voice", "-v", type=str, default="default", help="Voice profile ID")
    vo_parser.add_argument("--output", "-o", type=str, default="./voiceover.wav", help="Output audio path")
    vo_parser.add_argument("--language", "-l", type=str, default="id", help="Language code")
    vo_parser.add_argument("--speed", type=float, default=1.0, help="Speech speed (0.5-2.0)")

    sub_parser = subparsers.add_parser("subtitle", help="Generate subtitles from video")
    sub_parser.add_argument("video", type=str, help="Path to video file")
    sub_parser.add_argument("--language", "-l", type=str, default="id", help="Language code")
    sub_parser.add_argument("--output", "-o", type=str, help="Output SRT path (default: auto)")
    sub_parser.add_argument("--format", "-f", type=str, default="srt", choices=["srt", "vtt", "ass"])

    est_parser = subparsers.add_parser("estimate", help="Estimate token cost before running")
    est_parser.add_argument("footage", type=str, help="Path to footage directory")
    est_parser.add_argument("--script", "-s", type=str, help="Path to script file")
    est_parser.add_argument("--prompt", "-p", type=str, help="Editing instruction")

    subparsers.add_parser("list-templates", help="List available layout templates")

    config_parser = subparsers.add_parser("config", help="View or modify configuration")
    config_parser.add_argument("--show", action="store_true", help="Show current configuration")
    config_parser.add_argument("--set", "-s", type=str, action="append", help="Set config value (KEY=VALUE format)")

    return parser


def cmd_edit(args: argparse.Namespace) -> int:
    print(f"[Edit] Footage: {args.footage}")
    print(f"[Edit] Output: {args.output}")
    print(f"[Edit] Mode: {args.mode or 'hybrid'}")
    if args.prompt:
        router = IntentRouter()
        intent, params = router.classify(args.prompt)
        plan = router.create_plan(args.prompt)
        print(f"[Edit] Intent: {intent.value}")
        print(f"[Edit] Plan: {plan.duration}s, {plan.style.value}, {plan.aspect_ratio.value}")
    print("[Edit] Not fully implemented yet — Agent 2 will complete workflow execution.")
    return 0


def cmd_batch(args: argparse.Namespace) -> int:
    projects_dir = Path(args.projects_dir)
    if not projects_dir.exists():
        print(f"[Error] Projects directory not found: {args.projects_dir}", file=sys.stderr)
        return 1
    project_folders = [f for f in projects_dir.iterdir() if f.is_dir()]
    print(f"[Batch] Found {len(project_folders)} projects in {args.projects_dir}")
    print(f"[Batch] Format: {args.format}, Resolution: {args.resolution}")
    print("[Batch] Not fully implemented yet.")
    return 0


def cmd_voiceover(args: argparse.Namespace) -> int:
    text_path = Path(args.text)
    if not text_path.exists():
        print(f"[Error] Text file not found: {args.text}", file=sys.stderr)
        return 1
    print(f"[Voiceover] Text: {args.text}")
    print(f"[Voiceover] Voice: {args.voice}")
    print(f"[Voiceover] Language: {args.language}")
    print(f"[Voiceover] Output: {args.output}")
    print("[Voiceover] Not fully implemented yet — Agent 2 will complete TTS integration.")
    return 0


def cmd_subtitle(args: argparse.Namespace) -> int:
    video_path = Path(args.video)
    if not video_path.exists():
        print(f"[Error] Video file not found: {args.video}", file=sys.stderr)
        return 1
    output = args.output or Path(args.video).with_suffix(f".{args.format}")
    print(f"[Subtitle] Video: {args.video}")
    print(f"[Subtitle] Language: {args.language}")
    print(f"[Subtitle] Output: {output}")
    print("[Subtitle] Not fully implemented yet — Agent 2 will complete ASR integration.")
    return 0


def cmd_estimate(args: argparse.Namespace) -> int:
    config = load_config()
    footage_path = Path(args.footage)
    if not footage_path.exists():
        print(f"[Error] Footage directory not found: {args.footage}", file=sys.stderr)
        return 1
    video_files = list(footage_path.glob("*.mp4")) + list(footage_path.glob("*.mov"))
    audio_files = list(footage_path.glob("*.mp3")) + list(footage_path.glob("*.wav"))
    print(f"\n=== Token Estimation ===")
    print(f"Mode: {config.mode.value}")
    print(f"Footage: {len(video_files)} videos, {len(audio_files)} audio files")
    estimates = {"offline": {"planning": 2500, "execution": 0, "total": 2500},
                 "hybrid": {"planning": 3500, "execution": 1500, "total": 5000},
                 "cloud": {"planning": 8000, "execution": 12000, "total": 20000}}
    est = estimates.get(config.mode.value, estimates["hybrid"])
    print(f"Planning: ~{est['planning']} tokens")
    print(f"Execution: ~{est['execution']} tokens")
    print(f"Total: ~{est['total']} tokens")
    print(f"Estimated cost: ${est['total'] * 0.00015:.4f} (if API used)")
    return 0


def cmd_list_templates(args: argparse.Namespace) -> int:
    from auto_editor.orchestrator.template_db import TemplateDB
    db = TemplateDB()
    templates = db.list_all()
    if not templates:
        print("No templates found.")
        return 0
    print(f"\nAvailable Templates ({len(templates)}):")
    print(f"{'Name':<25} {'Style':<15} {'Aspect':<10} {'Description'}")
    print("-" * 75)
    for t in templates:
        print(f"{t['name']:<25} {t['style']:<15} {t['aspect_ratio']:<10} {t['description']}")
    return 0


def cmd_config(args: argparse.Namespace) -> int:
    config_path = Path(__file__).parent / "config" / "settings.yaml"
    config = load_config()
    if args.show:
        print(f"\nCurrent Configuration ({config.mode.value} mode):")
        print(f"  Local LLM: {config.local.llm_model}")
        print(f"  Local TTS: {config.local.tts_model}")
        print(f"  Local ASR: {config.local.asr_model}")
        print(f"  API Provider: {config.api.llm_provider}")
        print(f"  API Model: {config.api.llm_model}")
        print(f"  Max VRAM: {config.resources.max_vram_gb}GB")
        print(f"  Max RAM: {config.resources.max_ram_gb}GB")
        print(f"  Confidence Threshold: {config.behavior.confidence_threshold}")
        print(f"  Cache Enabled: {config.behavior.cache_enabled}")
    if args.set:
        for kv in args.set:
            if "=" not in kv:
                print(f"[Error] Invalid format: {kv}. Use KEY=VALUE", file=sys.stderr)
                return 1
            key, value = kv.split("=", 1)
            print(f"[Config] Would set {key}={value}")
    return 0


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    if not args.command:
        parser.print_help()
        return 0
    handlers = {
        "edit": cmd_edit, "batch": cmd_batch, "voiceover": cmd_voiceover,
        "subtitle": cmd_subtitle, "estimate": cmd_estimate,
        "list-templates": cmd_list_templates, "config": cmd_config,
    }
    handler = handlers.get(args.command)
    if handler:
        return handler(args)
    print(f"[Error] Unknown command: {args.command}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
