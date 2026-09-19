#!/usr/bin/env python3
"""Drop retired `'`-suffixed names from `.wat` COMMENT PROSE. Census by default; `--apply` writes.

WHY THIS EXISTS — the tooling gap it closes
-------------------------------------------
Arc 278 "0z" reclaimed the plain names for the IPC/kernel primes via the self-hosted codemod
(`wat-scripts/fixes/reclaim-ipc-prime-names.wat`). That codemod walks the FORM TREE, so it could
not touch comments — and `CLAUDE.md` says so outright: "Comments are NOT rewritten (it walks the
form tree); prose is a separate manual pass."

At 16 sites "manual" is fine. Measured 2026-09-19 it is ~340 sites across 14 files, and the drift
has cost real time: a reader hunting `spawn-program'` or `recv'` finds a verb that does not exist.
There was no supported tool for it (`python`/`sed` are banned for `.wat` because a structural
rewrite can corrupt forms), so this is that tool, scoped so the ban's rationale cannot bite:

  ⛔ IT CANNOT TOUCH CODE. Each line is split at the first `;;` that is OUTSIDE a string literal;
     only the text after it is rewritten. The form tree is unreachable by construction, not by
     care. A `;;` inside a string (a panic message) is not a comment opener and is left alone.

THE NAME LIST IS PROVEN DEAD, NOT ASSUMED
-----------------------------------------
Verbs — a real run (`wat <fixture>`, NOT `--check`) reports `UnknownFunction`:
    recv' send' connect' select' accept' close' listener' poll'
⚠ `--check` CANNOT arbitrate a `:wat::` name: `resolve/walk.rs` blanket-accepts a reserved-prefix
  head as resolved, so an unknown `:wat::kernel::definitely-not-a-verb` passes it silently. The
  lint's documented arbiter is wrong for exactly these names; a real run is the decider.
Control: `readln'` does NOT report UnknownFunction — it is a live defmacro expansion target, so
  the instrument discriminates rather than reporting deadness for everything.

Types — zero occurrences in Rust non-comment lines AND zero in wat code, i.e. absence of use:
    Peer' Address' Thread' Process' ThreadSelfPeer'

⛔ NOT ON THE LIST, each for a stated reason (from the codemod's own header):
    readln'   structurally required — the live `readln` defmacro expands to it
    Frame'    the positional-constructor idiom (Frame is the record, Frame' builds one)
    fire-rules' fire-once' fire-rules-explain' step-payload'
              the rete DUAL-IMPL — unprimed is the wat oracle, primed the native kernel,
              differential-tested against each other. Never collapse.
    deftest' deftest-hermetic'
              only appear in comments that DESCRIBE the rename ("... is reclaimed to this name
              (0z: prime -> plain)"). Stripping those makes the sentence say nothing.

Idempotent: it drops a trailing `'`, so a second run matches nothing.
"""

import argparse
import pathlib
import sys

# Longest-first so a shorter name cannot pre-empt a longer one that contains it.
RETIRED = [
    "recv-all-loop'",
    "serve-dispatch-op'",
    "ThreadSelfPeer'",
    "spawn-process'",
    "spawn-program'",
    "spawn-thread'",
    "recv-all'",
    "try-send'",
    "retag-op'",
    "listener'",
    "connect'",
    "Address'",
    "Process'",
    "accept'",
    "select'",
    "Thread'",
    "close'",
    "allow'",
    "deny'",
    "poll'",
    "recv'",
    "send'",
    "Peer'",
]


# The comment opener per language. wat: `;;`. Rust: `//` (covers `///` and `//!`).
OPENER = {"wat": ";;", "rust": "//"}
GLOB = {"wat": "*.wat", "rust": "*.rs"}


def comment_start(line: str, opener: str = ";;") -> int:
    """Index of the first `opener` OUTSIDE a string literal, or -1. This is the whole safety
    story: everything before it is code and is never touched.

    ⛔ Why string-awareness is load-bearing in BOTH languages, measured:
      wat  — `bracket.wat:703` holds a panic message containing `;;`-free but primed text; a
             naive split would have rewritten inside a string literal (a FORM change).
      rust — `crates/wat-reader/src/lexer.rs:1393` is `lex_tokens(":wat::kernel::Thread'<…>")`,
             a TEST FIXTURE asserting the lexer accepts a primed keyword. Rewriting it would
             change what the test tests. And `retired_name_justified` governs Rust string
             literals with earned runes — touching one could void a rune or the lint itself.
    """
    in_str = False
    esc = False
    i = 0
    while i < len(line):
        c = line[i]
        if esc:
            esc = False
        elif c == "\\":
            esc = True
        elif c == '"':
            in_str = not in_str
        elif not in_str and line.startswith(opener, i):
            return i
        i += 1
    return -1


def rewrite(line: str, counts: dict, opener: str = ";;") -> str:
    cut = comment_start(line, opener)
    if cut < 0:
        return line
    code, prose = line[:cut], line[cut:]
    for name in RETIRED:
        # ⛔ POSSESSIVES AND CONTRACTIONS ARE NOT PRIMED NAMES. `Peer's received-Op position`
        # contains the substring `Peer'`, and a blind replace yields `Peers received-Op` —
        # measured damage: the first run of this script ate two real possessives in
        # wat/check-adjacent prose before the guard existed. A primed NAME is never followed by
        # a letter; a possessive/contraction always is. So strip only when the next character
        # is absent or non-alphabetic.
        out = []
        i = 0
        while True:
            j = prose.find(name, i)
            if j < 0:
                out.append(prose[i:])
                break
            after = prose[j + len(name):j + len(name) + 1]
            out.append(prose[i:j])
            # ⛔ `Thread'<I,O>` is LEXER SYNTAX under discussion, not a stale API name. The
            # retired parametric form was `(Thread' :- [I O])`; the ANGLE form is the very
            # construct `AngleTypeHeadInName` rejects, so prose showing it must keep the `'`
            # or the sentence contradicts itself ("has `'` immediately before `<`"). Measured:
            # this guard is what keeps crates/wat-reader/src/lexer.rs's 7 rule-documenting
            # comments intact.
            if after.isalpha() or after == "<":
                out.append(name)          # `listener's` / `recv's` / `Thread'<` — leave alone
            else:
                out.append(name[:-1])     # the primed name — drop the `'`
                counts[name] = counts.get(name, 0) + 1
            i = j + len(name)
        prose = "".join(out)
    return code + prose


# ⛔ The cases that cost real damage, kept WITH the tool so the guard cannot regress silently.
# Rows 1-2 are the measured defect: the first run of this script ate `Peer's` into `Peers`.
SELFTEST = [
    (";; widening in Peer's received-Op position", ";; widening in Peer's received-Op position"),
    (";; Thread'/Process' ARE Peer's", ";; Thread/Process ARE Peer's"),
    (";; crash-aware `recv' sp` below", ";; crash-aware `recv sp` below"),
    (";; a thread listener's handle IS the grant", ";; a thread listener's handle IS the grant"),
    ('  (foo "a ;; not-a-comment recv\' here")', '  (foo "a ;; not-a-comment recv\' here")'),
    (";; ThreadSelfPeer' and Peer' both strip", ";; ThreadSelfPeer and Peer both strip"),
    ("// `make<`, `Thread'<`) is a lex error", "// `make<`, `Thread'<`) is a lex error"),
    ("// `Thread'<I,O>` has `'` immediately before `<`", "// `Thread'<I,O>` has `'` immediately before `<`"),
]


def selftest() -> int:
    bad = 0
    for src, want in SELFTEST:
        got = rewrite(src, {}, "//" if src.lstrip().startswith("//") else ";;")
        ok = got == want
        bad += 0 if ok else 1
        print(f"  {'ok  ' if ok else 'FAIL'}  {src!r}\n        -> {got!r}")
        if not ok:
            print(f"        want {want!r}")
    print(f"\n  selftest: {len(SELFTEST) - bad}/{len(SELFTEST)} passed")
    return 1 if bad else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="write changes (default: census only)")
    ap.add_argument("--root", action="append", default=None, help="directory to walk (repeatable)")
    ap.add_argument("--lang", choices=("wat", "rust"), default="wat", help="comment syntax + glob")
    ap.add_argument("--selftest", action="store_true", help="run the guard's own cases and exit")
    args = ap.parse_args()

    if args.selftest:
        return selftest()

    opener = OPENER[args.lang]
    roots = args.root or (["wat"] if args.lang == "wat" else ["src", "crates"])

    total = 0
    per_name: dict = {}
    touched = []
    paths = [q for r in roots for q in sorted(pathlib.Path(r).rglob(GLOB[args.lang]))]
    for path in paths:
        src = path.read_text()
        counts: dict = {}
        out = "".join(rewrite(ln, counts, opener) for ln in src.splitlines(keepends=True))
        hits = sum(counts.values())
        if not hits:
            continue
        # ⛔ The guard that makes this safe to run unattended: the number of CHANGED LINES must
        # equal the number of changed COMMENT lines. If a code line moved, refuse the file.
        a, b = src.splitlines(), out.splitlines()
        assert len(a) == len(b), f"{path}: line count changed"
        for la, lb in zip(a, b):
            if la != lb:
                cut = comment_start(la, opener)
                assert cut >= 0, f"{path}: a NON-COMMENT line changed: {la!r}"
                assert la[:cut] == lb[:cut], f"{path}: code before {opener} changed: {la!r}"
        total += hits
        touched.append((str(path), hits, dict(sorted(counts.items()))))
        for k, v in counts.items():
            per_name[k] = per_name.get(k, 0) + v
        if args.apply:
            path.write_text(out)

    for p, h, c in touched:
        print(f"  {p}: {h}  {c}")
    print(f"\n  files: {len(touched)}   occurrences: {total}")
    for k, v in sorted(per_name.items(), key=lambda kv: -kv[1]):
        print(f"    {k:<22} {v}")
    print("\n  MODE:", "APPLIED" if args.apply else "census only (pass --apply to write)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
