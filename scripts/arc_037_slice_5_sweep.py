#!/usr/bin/env python3
"""Arc 037 slice 5 — sweep set-dims! and set-capacity-mode! :error callers.

Both are redundant under arc 037:
- set-dims! is a no-op on the encoder path (the router decides d per
  construction; config.dims is unread).
- set-capacity-mode! :error sets the default.

Non-default capacity modes (:abort) are preserved. Non-trivial dim
claims (if anyone had them) are also preserved by only stripping
`set-dims!` with a plain integer arg — which is all of them, since
the parser only accepts integers there.

Reports per-file deletion counts and the total.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Matches a full line (with leading whitespace) containing either:
#   (:wat::config::set-dims! <any>)
# or
#   (:wat::config::set-capacity-mode! :error)
# Anchored on the opening paren of the setter to avoid chewing docs.
PAT_DIMS = re.compile(r"^\s*\(:wat::config::set-dims!\s+\d+\s*\)\s*$")
PAT_CAP_ERROR = re.compile(
    r"^\s*\(:wat::config::set-capacity-mode!\s+:error\s*\)\s*$"
)


def should_drop(line: str) -> bool:
    return bool(PAT_DIMS.match(line) or PAT_CAP_ERROR.match(line))


def sweep_file(path: Path) -> tuple[int, int]:
    """Return (lines_dropped, set_dims_kept_because_non_trivial)."""
    text = path.read_text()
    lines = text.splitlines(keepends=True)
    kept: list[str] = []
    dropped = 0
    for line in lines:
        if should_drop(line):
            dropped += 1
            continue
        kept.append(line)
    if dropped:
        path.write_text("".join(kept))
    return dropped, 0


def main() -> None:
    targets: list[Path] = []
    for pattern in ("**/*.wat", "**/*.rs"):
        for p in ROOT.rglob(pattern):
            # Skip vendor / build artifacts.
            if "target" in p.parts:
                continue
            targets.append(p)

    total = 0
    per_file: list[tuple[Path, int]] = []
    for path in targets:
        dropped, _ = sweep_file(path)
        if dropped:
            total += dropped
            per_file.append((path, dropped))

    per_file.sort(key=lambda kv: -kv[1])
    for path, n in per_file:
        rel = path.relative_to(ROOT)
        print(f"  {n:>4}  {rel}")
    print(f"\ntotal lines dropped: {total} across {len(per_file)} files")


if __name__ == "__main__":
    main()
