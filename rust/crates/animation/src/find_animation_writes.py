import json

log_path = "/home/brianatmokoo/.gemini/antigravity-ide/brain/2bd1f583-b1d6-4ba7-bc04-479665306bb5/.system_generated/logs/transcript.jsonl"
with open(log_path, 'r') as f:
    for line in f:
        try:
            data = json.loads(line)
            if "tool_calls" in data:
                for tc in data["tool_calls"]:
                    args = tc.get("args", {})
                    if "animation.rs" in args.get("TargetFile", "") or "animation.rs" in str(args.values()):
                        print(f"Step: {data.get('step_index')}, Tool: {tc.get('name')}, Args: {args}")
        except Exception as e:
            pass
