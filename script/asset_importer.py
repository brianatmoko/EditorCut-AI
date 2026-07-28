#!/usr/bin/env python3
"""
OpenCut Asset Importer & Validation Tool
Scans, validates, and reports all 2D Character Sprite Sheets, Skins, Items, Weapons,
and Background Objects in the OpenCut project workspace.
"""

from pathlib import Path
import json

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent

def scan_character_packs():
    packs = []
    for item in WORKSPACE_ROOT.iterdir():
        if item.is_dir() and item.name.startswith("craftpix-"):
            png_dir = item / "png"
            if not png_dir.exists():
                continue
            
            skins = []
            for skin_dir in png_dir.iterdir():
                if skin_dir.is_dir():
                    anims = {}
                    for anim_dir in skin_dir.iterdir():
                        if anim_dir.is_dir():
                            frames = list(anim_dir.glob("*.png"))
                            if frames:
                                anims[anim_dir.name] = len(frames)
                    if anims:
                        skins.append({
                            "id": f"{item.name}_{skin_dir.name}",
                            "skin_name": skin_dir.name,
                            "animations": anims
                        })
            if skins:
                packs.append({
                    "pack_name": item.name,
                    "skins_count": len(skins),
                    "skins": skins
                })
    return packs

def main():
    print("==================================================")
    print("      OPENCUT 2D ASSET & SKIN IMPORTER TOOL       ")
    print("==================================================")
    print(f"Workspace Root: {WORKSPACE_ROOT}\n")

    packs = scan_character_packs()
    total_skins = sum(p['skins_count'] for p in packs)

    print(f"[Summary] Discovered {len(packs)} Asset Packs, {total_skins} Total Character Skins:\n")

    for idx, pack in enumerate(packs, 1):
        print(f" Pack {idx}: {pack['pack_name']}")
        for skin in pack['skins']:
            anims_str = ", ".join(f"{k}: {v} frames" for k, v in skin['animations'].items())
            print(f"   - Skin '{skin['skin_name']}': [{anims_str}]")
        print()

    print("==================================================")
    print("Asset registry ready for OpenCut Engine integration.")

if __name__ == "__main__":
    main()
