#!/usr/bin/env python3
"""
Recover scratch/* (excl. 2026/05/*) from JSONL transcripts.

Adapted from wat-rs/docs/arc/2026/05/121-deftests-as-cargo-tests/
AGENT-TRANSCRIPT-RECOVERY.md.

Improvements over the v1 doctrine:
- Pre-scans tool_results to identify Edits that errored in the
  ORIGINAL session (so we skip them as the agent's own retries).
- Replays a whitelist of Bash ops chronologically alongside
  Write/Edit (only `sed -i` against scratch paths). Other Bash
  is ignored.
"""
import json, os, re, shlex, subprocess, sys
from pathlib import Path

TRANSCRIPTS = [
    "/home/watmin/.claude/projects/-home-watmin-work-holon/46d24d6a-885e-4859-8351-c42ba28a7a01.jsonl",
    "/home/watmin/.claude/projects/-home-watmin-work-holon/33f9a929-0816-4d87-9689-212e6bea4a83.jsonl",
    "/home/watmin/.claude/projects/-home-watmin-work-holon/eb796aee-fdce-4e20-9c32-707dc9ae7504.jsonl",
    "/home/watmin/.claude/projects/-home-watmin-work-holon/be679993-dd21-4707-90ae-b89fd11acbf8.jsonl",
    "/home/watmin/.claude/projects/-home-watmin-work-holon/bc87fd88-050a-4542-bf0c-ccb5a18db436.jsonl",
    "/home/watmin/.claude/projects/-home-watmin-work-holon/bc87fd88-050a-4542-bf0c-ccb5a18db436/subagents/agent-aaa0128a639b57930.jsonl",
]

SRC_PREFIX = "/home/watmin/work/holon/scratch/"
DST_PREFIX = "/home/watmin/work/scratch/"
SKIP_PREFIX = "/home/watmin/work/holon/scratch/2026/05/"

def rewrite(path):
    if not path.startswith(SRC_PREFIX): return None
    if path.startswith(SKIP_PREFIX): return None
    return DST_PREFIX + path[len(SRC_PREFIX):]

# --- Bash whitelist: only `sed -i` against scratch paths ---
def is_safe_bash(cmd):
    """Return True iff cmd is a sed -i operation we can safely replay."""
    s = cmd.strip()
    if "rm -rf" in s or "rm -f" in s: return False
    # The pattern we know about: `cd <scratch>/... && sed -i 'EXPR' file...`
    # or just `sed -i 'EXPR' /path/.../file`
    if "sed -i" not in s: return False
    if SRC_PREFIX not in s and "scratch/" not in s: return False
    return True

def rewrite_bash(cmd):
    """Rewrite holon-scratch paths to new-scratch paths in a Bash command."""
    return cmd.replace(SRC_PREFIX, DST_PREFIX)

def apply_bash(cmd):
    rewritten = rewrite_bash(cmd)
    return subprocess.run(rewritten, shell=True, check=True,
                          capture_output=True, text=True)

def apply_write(path, content):
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content)

def apply_edit(path, old, new, replace_all=False):
    p = Path(path)
    if not p.exists():
        raise RuntimeError(f"target missing for Edit: {path}")
    text = p.read_text()
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

def scan_errored_ids(transcripts):
    """Return set of tool_use_ids whose tool_result was is_error=true."""
    bad = set()
    for transcript in transcripts:
        if not Path(transcript).exists(): continue
        with open(transcript) as f:
            for line in f:
                line = line.strip()
                if not line: continue
                try: rec = json.loads(line)
                except json.JSONDecodeError: continue
                msg = rec.get("message") or {}
                content = msg.get("content") or []
                if not isinstance(content, list): continue
                for blk in content:
                    if not isinstance(blk, dict): continue
                    if blk.get("type") != "tool_result": continue
                    if blk.get("is_error"):
                        tid = blk.get("tool_use_id")
                        if tid: bad.add(tid)
    return bad

def collect_ops(transcripts, skip_ids):
    """Collect (timestamp, kind, payload) tuples for replay."""
    ops = []
    for transcript in transcripts:
        if not Path(transcript).exists():
            print(f"missing transcript: {transcript}", file=sys.stderr)
            continue
        with open(transcript) as f:
            for line in f:
                line = line.strip()
                if not line: continue
                try: rec = json.loads(line)
                except json.JSONDecodeError: continue
                msg = rec.get("message") or {}
                content = msg.get("content") or []
                if not isinstance(content, list): continue
                ts = rec.get("timestamp", "")
                for blk in content:
                    if not isinstance(blk, dict): continue
                    if blk.get("type") != "tool_use": continue
                    name = blk.get("name")
                    inp = blk.get("input") or {}
                    tid = blk.get("id")
                    if tid in skip_ids:
                        continue
                    if name in ("Write", "Edit"):
                        raw_path = inp.get("file_path", "")
                        new_path = rewrite(raw_path)
                        if new_path is None: continue
                        ops.append((ts, name, new_path, inp, tid))
                    elif name == "Bash":
                        cmd = inp.get("command", "")
                        if is_safe_bash(cmd):
                            ops.append((ts, "Bash", None, {"command": cmd}, tid))
    return ops

def main():
    skip_ids = scan_errored_ids(TRANSCRIPTS)
    print(f"pre-scan: {len(skip_ids)} originally-errored tool_use_ids to skip",
          file=sys.stderr)
    ops = collect_ops(TRANSCRIPTS, skip_ids)
    ops.sort(key=lambda o: o[0])
    print(f"collected {len(ops)} ops touching scratch (excl. 2026/05/*)",
          file=sys.stderr)
    counts = {}
    for op in ops: counts[op[1]] = counts.get(op[1], 0) + 1
    for k, v in sorted(counts.items()):
        print(f"  {k}: {v}", file=sys.stderr)
    fails = []
    for i, (ts, name, path, inp, tid) in enumerate(ops, 1):
        try:
            if name == "Write":
                apply_write(path, inp.get("content", ""))
            elif name == "Edit":
                apply_edit(path, inp.get("old_string",""),
                           inp.get("new_string",""),
                           bool(inp.get("replace_all", False)))
            elif name == "Bash":
                apply_bash(inp.get("command", ""))
        except Exception as e:
            fails.append((i, name, path or inp.get("command","")[:80], str(e), tid))
            print(f"  [{i}/{len(ops)}] {name} {path or inp.get('command','')[:80]} FAIL: {e}",
                  file=sys.stderr)
            continue
    print(f"\nDONE. {len(ops)-len(fails)}/{len(ops)} ops applied.",
          file=sys.stderr)
    if fails:
        print(f"\n{len(fails)} failures:", file=sys.stderr)
        for i, name, path, err, tid in fails:
            print(f"  [{i}] {name} {path} (tid={tid}): {err}", file=sys.stderr)
    return 0 if not fails else 1

if __name__ == "__main__":
    sys.exit(main())
