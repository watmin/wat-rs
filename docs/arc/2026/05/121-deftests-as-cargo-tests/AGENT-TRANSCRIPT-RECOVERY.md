# Recovery — replaying lost agent edits from JSONL transcripts

**Captured 2026-05-01 mid-arc-121.** Discovered the move
during arc 119's debug — useful in any session where an
agent's edits get lost (revert, crash, mistaken `git checkout
HEAD --`, working-tree reset).

## When this matters

A subagent run via the `Agent` tool produces a complete JSONL
transcript on disk. Every tool call (Write / Edit / Read /
Bash) is recorded with its full input. If the agent's edits
later disappear from the working tree — for any reason — the
transcript is the source of truth and the edits are
recoverable.

## Where the transcripts live

```
/home/<user>/.claude/projects/<session-slug>/subagents/agent-<agent-id>.jsonl
```

The `Agent` tool's launch message includes `agentId: <id>`. The
session also exposes a symlink at:

```
/tmp/claude-<uid>/<session-slug>/tasks/<agent-id>.output
```

Both point at the same file. Use either path.

## The discipline

**Do NOT cat / head / tail / Read the transcript directly** —
the file contains every tool input + output verbatim, including
file contents from Read calls. Dumping it into context will
overflow.

**Do operate on it without reading it.** Use `jq` and small
Python scripts that emit only summary information (counts,
file paths) — never payload.

## Probe before recovery

```bash
F=/path/to/agent-<id>.jsonl

# Size + line count
wc -l "$F"

# Tool-use distribution (no payload)
jq -r 'select(.message.content?) | .message.content[]?
       | select(.type == "tool_use") | .name' "$F" | sort | uniq -c

# File paths touched (paths only, no payload)
jq -r 'select(.message.content?) | .message.content[]?
       | select(.type == "tool_use" and (.name == "Edit" or .name == "Write"))
       | "\(.name) \(.input.file_path)"' "$F"
```

These give you the scope without showing any file content.

## Recovery script

Apply Write / Edit tool inputs in chronological order to the
target files. Python is convenient because Edit semantics
(unique-old-string, replace_all flag) are easy to express.

```python
#!/usr/bin/env python3
"""Replay an agent's tool-use transcript onto the working tree."""
import json, sys
from pathlib import Path

TRANSCRIPT = "/path/to/agent-<id>.jsonl"

def apply_write(path, content):
    Path(path).write_text(content)

def apply_edit(path, old, new, replace_all=False):
    p = Path(path); text = p.read_text()
    if replace_all:
        if old not in text:
            raise RuntimeError(f"old_string not found in {path}")
        text = text.replace(old, new)
    else:
        n = text.count(old)
        if n == 0: raise RuntimeError(f"old_string not found in {path}")
        if n > 1:  raise RuntimeError(f"old_string not unique in {path} (n={n})")
        text = text.replace(old, new, 1)
    p.write_text(text)

def main():
    ops = []
    with open(TRANSCRIPT) as f:
        for line in f:
            try: rec = json.loads(line.strip())
            except json.JSONDecodeError: continue
            content = (rec.get("message") or {}).get("content") or []
            if not isinstance(content, list): continue
            for blk in content:
                if not isinstance(blk, dict): continue
                if blk.get("type") != "tool_use": continue
                if blk.get("name") not in ("Write", "Edit"): continue
                ops.append((rec.get("timestamp",""), blk.get("name"),
                            blk.get("input", {})))
    ops.sort(key=lambda o: o[0])
    print(f"Found {len(ops)} Write+Edit records", file=sys.stderr)
    for i, (ts, name, inp) in enumerate(ops, 1):
        path = inp.get("file_path", "")
        if name == "Write":
            apply_write(path, inp.get("content", ""))
        else:
            apply_edit(path, inp.get("old_string",""),
                       inp.get("new_string",""),
                       bool(inp.get("replace_all", False)))
        print(f"  [{i}/{len(ops)}] {name} {path} OK", file=sys.stderr)
    return 0

if __name__ == "__main__": sys.exit(main())
```

## Pre-flight before running

1. **Confirm the target files are at the state the agent
   started from.** If the agent did a Write first (whole-file
   replacement), starting state doesn't matter for that file.
   If the agent did Edits only, the file must be at HEAD (or
   wherever the agent picked it up).

2. **Apply in a test branch first if uncertain.** `git
   checkout -b recovery-attempt` before running the script.

3. **Verify after.** `git diff --stat` should show edits
   landing on the expected files. `git diff <file>` confirms
   per-file content.

## Edge cases

- **Sequential edits whose `old_string` repeats** — the script
  refuses to apply a non-replace_all Edit when `old_string`
  matches multiple times. The agent itself would have failed
  the same way; if you see this error, the transcript is
  probably out of order or there was an intermediate state
  difference. Investigate via the timestamps.

- **Replace_all edits** — the script handles them via the
  `replace_all` flag in the tool input.

- **Reads / Bash** — ignored. The recovery only replays Write
  and Edit. Read calls are the agent's input, not output;
  Bash calls might have side effects but those should already
  be in the working tree state at recovery time.

## What this trick is NOT

- Not a substitute for committing often. Commit + push is
  still the discipline. This trick recovers from mistakes
  AGAINST that discipline.
- Not a way to "undo" a commit. Once committed, use `git
  revert` / `git reflog` instead.
- Not a way to read a transcript's content into your context
  — the script never echoes file contents.

## Provenance for arc 119 / arc 121

Used in arc 121's debugging on 2026-05-01 after a mistaken
`git checkout HEAD --` revert wiped a sub-agent's wat-test
edits. The agent had run for several minutes producing a
correct discipline-correction sweep; the edits were
recovered intact from the transcript and arc 119 step 7
resumed without re-delegating to the agent.

User direction (2026-05-01):
> we cannot forget this trick.... write a note into the
> latest arc about this move
