import json
import re

log_path = "/home/brianatmokoo/.gemini/antigravity-ide/brain/2bd1f583-b1d6-4ba7-bc04-479665306bb5/.system_generated/logs/transcript.jsonl"
files_to_restore = [
    "shapes.rs",
    "render.rs",
    "pose.rs",
    "poses.rs",
    "animator.rs",
    "script.rs",
    "character.rs",
]

file_contents = {}

# Compile regex for each file name to match paths even with backslashes or escapes
patterns = {f: re.compile(rf"{f.replace('.', r'\.')}") for f in files_to_restore}

with open(log_path, 'r') as f:
    for line in f:
        try:
            data = json.loads(line)
            if "tool_calls" in data:
                for tc in data["tool_calls"]:
                    name = tc.get("name")
                    args = tc.get("args", {})
                    
                    # Look at TargetFile argument
                    target = str(args.get("TargetFile", ""))
                    # Clean up backslashes and newlines in path to search
                    clean_target = target.replace('\\n', '').replace('\\', '').replace('\n', '').replace('"', '')
                    
                    matched_file = None
                    for f_name, pat in patterns.items():
                        if pat.search(clean_target):
                            matched_file = f_name
                            break
                            
                    if not matched_file:
                        continue
                        
                    if name == "write_to_file":
                        code = args.get("CodeContent", "")
                        if code:
                            file_contents[matched_file] = code
                    elif name == "replace_file_content":
                        target_content = args.get("TargetContent", "")
                        repl_content = args.get("ReplacementContent", "")
                        if matched_file in file_contents:
                            file_contents[matched_file] = file_contents[matched_file].replace(target_content, repl_content)
        except Exception as e:
            pass

# Now write them back
for f_name, code in file_contents.items():
    if isinstance(code, str):
        # Resolve all escaping
        code = code.replace('\\n', '\n').replace('\\t', '\t').replace('\\"', '"').replace('\\\\', '\\')
        # strip quotes if double encoded
        code = code.strip()
        if code.startswith('"') and code.endswith('"'):
            code = code[1:-1]
        code = code.strip()
        
    out_path = f"/home/brianatmokoo/Documents/Linux/Opencut/rust/crates/animation/src/{f_name}"
    with open(out_path, "w") as out:
        out.write(code)
    print(f"Restored: {f_name} (Length: {len(code)})")
