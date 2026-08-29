#!/usr/bin/env python3
"""p6c-disposition-census — per-FQDN disposition table for the P6-c campaign.

WHY THIS EXISTS (arc 255, STONE P6-c-0). `dispatch_keyword_head_value`'s giant match
(`src/runtime.rs:5365-6884`) needs a per-FQDN disposition before any arm is homed to the
`#[wat_intrinsic]` registry: INTRINSIC-READY / NEEDS-SHAPE / SPECIAL-FORM / MULTI-SITE. The unit
is the FQDN, not the arm — one arm can carry several FQDNs (`|` alternation, or a nested `match
head {}` cluster), and one FQDN can be served by more than one dispatch site (only one of which
may be live). See `docs/arc/2026/06/255-builtin-registry/NOTE-p6c-is-a-campaign-not-a-stone.md`
and `BRIEF-STONE-P6-c-0-the-disposition-census.md`.

WHAT IT MEASURES
  1. Parses the giant match with a small brace/paren/string-aware scanner (NOT a fixed-indent
     regex — the prior attempt at this census tried "12-space indent" and got 0 arms, then a
     bare `"lit" =>` regex and silently ate the guard/wildcard arms; both are recorded in the
     NOTE as wrong for a documented reason). Reports the raw top-level arm count.
  2. Extracts every FQDN string literal in each arm's PATTERN (handles `|` alternation and
     `name @ (alt1 | alt2 | ...)` bind-patterns), and recurses into any `match head {` found
     INSIDE an arm's body (a cluster arm dispatches further FQDNs one level down — e.g. the
     ten `:wat::eval-*!` forms share one outer arm).
  3. For each FQDN, extracts the single leading delegate call in the arm body (if there is
     one) and its raw argument list, and classifies a CANDIDATE disposition by comparing that
     argument list against the `#[wat_intrinsic]` BINDING shim's call shape. **UPDATED arc 255
     Stone P6-c-1**: the shim no longer forwards a fixed `(env, sym, list_span)` triple — it
     forwards exactly the context params the CALLEE'S OWN signature declares, in the order the
     callee declares them (`sniff_args` sniffs an ORDERED sequence now, not a
     `seen_context: bool`; `crates/wat-macros/src/wat_intrinsic.rs`'s BINDING call sites are
     `#fn_name(args, #(#tail_tokens),*)` / `#fn_name(#(#arg_forwards,)* #(#tail_tokens),*)`).
     So a candidate's trailing call args need only be A CONTIGUOUS SUFFIX drawn from
     `{env, sym, list_span}` (each at most once, any order, any subset — 0 to 3 of them) — no
     longer all three, no longer that one fixed order. `.clone()`-hiding an OWNED tail param
     (`eval_apply`'s owned `Span`), an extra non-context arg, or a body that is not one
     delegating call, are still reported as the REASON the candidate is NEEDS-SHAPE.
  4. Flags a `SPECIAL FORM` comment near an arm (case-insensitive) as SPECIAL-FORM-CANDIDATE.
  5. Flags a `starts_with(":...")` guard pattern (the `:rust::` namespace arm) as its own
     PREFIX-GUARD class, and the final bare `other`/`_` arm as CATCH-ALL — neither is one of
     the four dispositions and neither should be force-fit into one (BRIEF STOP-1).
  6. For every FQDN found, greps the WHOLE repo tree for the same literal appearing in another
     dispatch-shaped context (`"<fqdn>" =>`, `== "<fqdn>"`, `.starts_with("<fqdn>")`, or as a
     bare list entry) OUTSIDE its home line in the giant match, and reports every hit as a
     MULTI-SITE CANDIDATE — a hit here is not proof of a second live dispatch site (a doc
     comment, an error-message literal, or a `RUNTIME_DECLARATION_HEADS`-style table entry
     reads identically to a grep) and always needs a human to open the file and read it.

★ THE CANDIDATE LABEL IS NOT THE DISPOSITION. Every "-CANDIDATE" label in this tool's output is
a mechanical proxy from argument-list SHAPE, not a verdict — the brief's own stop-trigger #3
governs it: a hand-read control disagreeing with this tool wins, always. Known instance found
while building this tool: `:wat::core::apply`'s call site is `eval_apply(args, env, sym,
list_span.clone())` — env/sym/list_span in the exact BINDING order — which a naive order-only
check would pass as INTRINSIC-READY; reading `eval_apply`'s OWN declaration
(`src/runtime.rs:10621`) shows its 4th parameter is owned `Span`, not `&Span`, which the shim
never clones for the delegate call — so it is NEEDS-SHAPE. This tool's classifier therefore
does NOT trust the call-site order alone: it also greps the callee's own `fn` declaration (by
name, across `src/`) for its parameter list and cross-checks reference-vs-owned on the tail
three parameters. Even so, treat every non-obvious verdict as a lead to go read, not an answer.

WHAT THIS TOOL CANNOT SEE
  - It classifies from TEXT SHAPE, never from type-checking `cargo build` against the real
    macro shape. It cannot see const-generic bound mismatches, lifetime issues, or a handler
    that would compile under BINDING but must not be homed for a different reason (behavior,
    not shape) — e.g. `:wat::config::set-redef!`'s eval-time arm, which is INTRINSIC-SHAPED
    (`Ok(Value::Unit)`, trivially bindable) but is a DELIBERATE NO-OP; its correct behavior
    lives at freeze time, a different function, and homing the eval arm naively would move the
    no-op into the registry and strand the mutation behind. Only a human, reading the multi-site
    grep this tool prints, catches that; this tool prints the sites, not the verdict.
  - The callee-signature grep matches by BARE FUNCTION NAME across `src/`; an overloaded/shadowed
    name (rare in this codebase, unchecked) could pick the wrong declaration.
  - A multi-line pattern's own continuation line (e.g. the ten-way `head @ (... )` alternation)
    is walked by the same string/paren-aware scanner used for the whole match, not a per-line
    indent guess — but the scanner assumes head patterns are plain string literals or bound
    alternations; it does not understand tuple/struct patterns, so it would silently miss a
    dispatch that matched on anything other than the bare `&str` head.
  - It does not run inside a wat program and does not touch `wat-scripts/scratch-pad`'s
    `every_wat_scripts_file_loads` gate — this is a plain Rust-source census tool, kept as a
    `.py` alongside `fn-census.py` and the `.awk` census tools already in this directory, not a
    `.wat` codemod (no `.wat` file is read or written).
  - It moves nothing. It is read-only over `src/` and prints a report; `cargo build` is
    unaffected by running it.

USAGE
  wat-scripts/hunt/p6c-disposition-census.py                # full report to stdout
  wat-scripts/hunt/p6c-disposition-census.py --control "H1,H2,H3,H4,H5"
                                                              # only these FQDNs (for a
                                                              # hand-read control comparison)
  wat-scripts/hunt/p6c-disposition-census.py --json out.json # also dump the full structured
                                                              # table as JSON
No dependencies; plain python3, no venv needed.
"""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_RS = REPO_ROOT / "src" / "runtime.rs"

FQDN_RE = re.compile(r'"(:[^"]*)"')
CALL_RE = re.compile(r'([A-Za-z_][A-Za-z0-9_:]*)\s*\(')


# ─── string/paren/comment-aware scanner (shared by pattern and arm parsing) ──────────────────
class Scanner:
    def __init__(self, s):
        self.s = s
        self.n = len(s)
        self.i = 0

    def skip_ws_and_comments(self):
        while self.i < self.n:
            c = self.s[self.i]
            if c in " \t\n\r":
                self.i += 1
                continue
            if c == "/" and self.i + 1 < self.n and self.s[self.i + 1] == "/":
                nl = self.s.find("\n", self.i)
                self.i = nl + 1 if nl != -1 else self.n
                continue
            if c == "/" and self.i + 1 < self.n and self.s[self.i + 1] == "*":
                end = self.s.find("*/", self.i + 2)
                self.i = end + 2 if end != -1 else self.n
                continue
            break

    def consume_string(self):
        assert self.s[self.i] == '"'
        start = self.i
        self.i += 1
        while self.i < self.n:
            c = self.s[self.i]
            if c == "\\":
                self.i += 2
                continue
            if c == '"':
                self.i += 1
                break
            self.i += 1
        return self.s[start : self.i]

    def maybe_char_lit(self):
        # heuristic: a char literal closes within 4 chars; a lifetime ('a) does not.
        j = self.i + 1
        k = j
        while k < min(self.n, j + 4):
            if self.s[k] == "'":
                return True
            if self.s[k] == "\\":
                k += 2
                continue
            k += 1
        return False

    def consume_char_lit(self):
        start = self.i
        self.i += 1
        while self.i < self.n:
            c = self.s[self.i]
            if c == "\\":
                self.i += 2
                continue
            if c == "'":
                self.i += 1
                break
            self.i += 1
        return self.s[start : self.i]

    def consume_balanced(self, open_ch, close_ch):
        assert self.s[self.i] == open_ch
        start = self.i
        self.i += 1
        depth = 1
        while self.i < self.n and depth > 0:
            c = self.s[self.i]
            if c == '"':
                self.consume_string()
                continue
            if c == "'":
                if self.maybe_char_lit():
                    self.consume_char_lit()
                else:
                    self.i += 1
                continue
            if c == "/" and self.i + 1 < self.n and self.s[self.i + 1] == "/":
                nl = self.s.find("\n", self.i)
                self.i = nl + 1 if nl != -1 else self.n
                continue
            if c == "/" and self.i + 1 < self.n and self.s[self.i + 1] == "*":
                end = self.s.find("*/", self.i + 2)
                self.i = end + 2 if end != -1 else self.n
                continue
            if c == open_ch:
                depth += 1
                self.i += 1
                continue
            if c == close_ch:
                depth -= 1
                self.i += 1
                continue
            self.i += 1
        return self.s[start : self.i]


def build_offset_to_line(s, base_line):
    offsets = [0] * len(s)
    line = base_line
    for idx, ch in enumerate(s):
        offsets[idx] = line
        if ch == "\n":
            line += 1
    return offsets


def parse_arms(body_text, offset_to_line=None):
    """Parse a sequence of match arms out of the text strictly inside `match X { ... }`.
    Returns a list of dicts: pattern, body, pattern_start_line, arrow_line (None if no
    line map supplied — used for recursive/nested parses where line numbers aren't tracked)."""
    sc = Scanner(body_text)
    arms = []
    n = len(body_text)

    def line_of(pos):
        if offset_to_line is None:
            return None
        pos = max(0, min(pos, len(offset_to_line) - 1))
        return offset_to_line[pos]

    while True:
        pre_skip_pos = sc.i
        sc.skip_ws_and_comments()
        if sc.i >= n:
            break
        leading_text = body_text[pre_skip_pos : sc.i]
        pat_start = sc.i
        depth = 0
        arrow_pos = None
        while sc.i < n:
            c = sc.s[sc.i]
            if c == '"':
                sc.consume_string()
                continue
            if c == "'":
                if sc.maybe_char_lit():
                    sc.consume_char_lit()
                else:
                    sc.i += 1
                continue
            if c == "/" and sc.i + 1 < n and sc.s[sc.i + 1] == "/":
                nl = sc.s.find("\n", sc.i)
                sc.i = nl + 1 if nl != -1 else n
                continue
            if c == "/" and sc.i + 1 < n and sc.s[sc.i + 1] == "*":
                end = sc.s.find("*/", sc.i + 2)
                sc.i = end + 2 if end != -1 else n
                continue
            if c in "([{":
                depth += 1
                sc.i += 1
                continue
            if c in ")]}":
                depth -= 1
                sc.i += 1
                continue
            if depth == 0 and c == "=" and sc.i + 1 < n and sc.s[sc.i + 1] == ">":
                arrow_pos = sc.i
                sc.i += 2
                break
            sc.i += 1
        if arrow_pos is None:
            break
        pattern_text = body_text[pat_start:arrow_pos]
        sc.skip_ws_and_comments()
        body_start = sc.i
        if sc.i < n and sc.s[sc.i] == "{":
            sc.consume_balanced("{", "}")
            body_end = sc.i
            save = sc.i
            sc.skip_ws_and_comments()
            if sc.i < n and sc.s[sc.i] == ",":
                sc.i += 1
            else:
                sc.i = save
        else:
            depth2 = 0
            while sc.i < n:
                c = sc.s[sc.i]
                if c == '"':
                    sc.consume_string()
                    continue
                if c == "'":
                    if sc.maybe_char_lit():
                        sc.consume_char_lit()
                    else:
                        sc.i += 1
                    continue
                if c == "/" and sc.i + 1 < n and sc.s[sc.i + 1] == "/":
                    nl = sc.s.find("\n", sc.i)
                    sc.i = nl + 1 if nl != -1 else n
                    continue
                if c == "/" and sc.i + 1 < n and sc.s[sc.i + 1] == "*":
                    end = sc.s.find("*/", sc.i + 2)
                    sc.i = end + 2 if end != -1 else n
                    continue
                if c in "([{":
                    depth2 += 1
                    sc.i += 1
                    continue
                if c in ")]}":
                    if depth2 == 0:
                        break
                    depth2 -= 1
                    sc.i += 1
                    continue
                if depth2 == 0 and c == ",":
                    sc.i += 1
                    break
                sc.i += 1
            body_end = sc.i
        arms.append(
            {
                "pattern": pattern_text,
                "body": body_text[body_start:body_end],
                "leading": leading_text,
                "pattern_start_line": line_of(pat_start),
                "arrow_line": line_of(arrow_pos),
            }
        )
    return arms


def extract_fqdns(pattern_text):
    return [f'"{m}"' for m in FQDN_RE.findall(pattern_text)]


def find_prefix_guard(pattern_text):
    m = re.search(r'starts_with\(\s*"(:[^"]*)"\s*\)', pattern_text)
    return m.group(1) if m else None


def strip_comments(s):
    """Remove `//...` and `/* ... */` comments from `s`, string/char-literal-aware (a comment
    marker inside a string or char literal is left alone). Found while building this census:
    `eval_and`/`eval_or` (src/runtime.rs:11409) declare an inline `// rune:lint(unused-span)`
    comment on the trailing `_list_span` param whose OWN prose contains a comma
    ("... `arg.span()`, more precise ...") — `top_level_split` has no comment awareness, so
    that comma reads as a top-level parameter separator and tears the comment itself into two
    bogus "parameters", corrupting the by-ref/by-value check for every param after it. Applied
    to a parameter-list string ONLY (not general source text) before splitting."""
    sc = Scanner(s)
    out = []
    n = sc.n
    while sc.i < n:
        c = sc.s[sc.i]
        if c == '"':
            start = sc.i
            sc.consume_string()
            out.append(sc.s[start : sc.i])
            continue
        if c == "'":
            if sc.maybe_char_lit():
                start = sc.i
                sc.consume_char_lit()
                out.append(sc.s[start : sc.i])
            else:
                out.append(c)
                sc.i += 1
            continue
        if c == "/" and sc.i + 1 < n and sc.s[sc.i + 1] == "/":
            nl = sc.s.find("\n", sc.i)
            sc.i = nl if nl != -1 else n
            continue
        if c == "/" and sc.i + 1 < n and sc.s[sc.i + 1] == "*":
            end = sc.s.find("*/", sc.i + 2)
            sc.i = end + 2 if end != -1 else n
            continue
        out.append(c)
        sc.i += 1
    return "".join(out)


def top_level_split(s, sep=","):
    parts = []
    depth = 0
    cur = []
    i = 0
    n = len(s)
    while i < n:
        c = s[i]
        if c == '"':
            j = i + 1
            while j < n and s[j] != '"':
                if s[j] == "\\":
                    j += 1
                j += 1
            cur.append(s[i : j + 1])
            i = j + 1
            continue
        if c in "([{":
            depth += 1
        if c in ")]}":
            depth -= 1
        if c == sep and depth == 0:
            parts.append("".join(cur))
            cur = []
            i += 1
            continue
        cur.append(c)
        i += 1
    if cur:
        parts.append("".join(cur))
    return [p.strip() for p in parts if p.strip()]


def unwrap_single_stmt_block(b):
    """If `b` is exactly a `{ ... }` block whose content is ONE top-level statement/expression
    (no other top-level `;`-separated statement beside an optional trailing one), return that
    inner text; otherwise return `b` unchanged. Found while building this census: the giant
    match's overwhelmingly common arm shape is `"fqdn" => { delegate(args...) }` — a
    brace-wrapped single call — and without this unwrap step EVERY such arm (the majority of
    the match) was misclassified COMPLEX for the sole reason that its body starts with `{`,
    not an identifier. Caught by spot-checking the COMPLEX bucket by hand.
    """
    b2 = b.strip()
    if not (b2.startswith("{") and b2.endswith("}")):
        return b
    sc = Scanner(b2)
    sc.i = 0
    block = sc.consume_balanced("{", "}")
    if sc.i != len(b2):
        return b  # trailing content after the closing brace — not a bare block
    inner = block[1:-1]
    stmts = top_level_split(inner, sep=";")
    if len(stmts) != 1:
        return b  # multiple top-level statements — genuinely COMPLEX
    return stmts[0].strip()


def find_primary_call(body):
    """Find the single leading `name(...)` call in an arm body, if the body IS one
    (optionally `return`-prefixed) delegating call. Returns (name, [arg strings], trailing)
    or None if the body doesn't look like a single call (multi-statement, inline match, etc)."""
    b = unwrap_single_stmt_block(body.strip())
    b = b.strip()
    # A leading `//` comment inside the block (e.g. `{ // note\n delegate(...) }`) survives
    # unwrap_single_stmt_block (it has no top-level `;` to split on) but breaks a match at
    # position 0 — strip leading comments/whitespace before matching the call.
    _sc = Scanner(b)
    _sc.skip_ws_and_comments()
    b = b[_sc.i :]
    b = re.sub(r"^return\s+", "", b)
    m = CALL_RE.match(b)
    if not m:
        return None
    name = m.group(1)
    start = m.end() - 1
    sc = Scanner(b)
    sc.i = start
    call_text = sc.consume_balanced("(", ")")
    args_text = call_text[1:-1]
    rest = b[sc.i :].strip()
    # Body must be (near enough) JUST this call: allow a trailing `.map(...)`, `.into()`,
    # or a bare `,`/`;` — anything else (another statement, an `if`, a second call at top
    # level) means this arm is not a single delegating call and we report it as COMPLEX.
    if rest and not re.match(r"^(\.\w+\([^)]*\)|\.into\(\)|;|,)*$", rest):
        return None
    return name, top_level_split(args_text), rest


# ─── delegate-signature lookup (grep the callee's own `fn` declaration) ──────────────────────
_SIG_CACHE = {}


def find_fn_signature(fn_name):
    """Grep src/ for `fn <name>(` (bare name after the last `::`), return its raw parameter
    list text, or None if zero or >1 declarations are found (ambiguous — reported, not guessed)."""
    bare = fn_name.rsplit("::", 1)[-1]
    if bare in _SIG_CACHE:
        return _SIG_CACHE[bare]
    try:
        out = subprocess.run(
            ["grep", "-rn", "-E", rf"\bfn {re.escape(bare)}\s*\(", str(REPO_ROOT / "src")],
            capture_output=True,
            text=True,
            timeout=30,
        ).stdout
    except Exception:
        out = ""
    lines = [l for l in out.splitlines() if l.strip()]
    if len(lines) != 1:
        _SIG_CACHE[bare] = None
        return None
    path, lineno, _ = lines[0].split(":", 2)
    with open(path) as f:
        src_lines = f.readlines()
    # collect from the fn line until the matching ')' of the parameter list
    start_idx = int(lineno) - 1
    text = "".join(src_lines[start_idx : start_idx + 40])
    m = re.search(r"fn\s+" + re.escape(bare) + r"\s*\(", text)
    if not m:
        _SIG_CACHE[bare] = None
        return None
    sc = Scanner(text)
    sc.i = m.end() - 1
    params_text = sc.consume_balanced("(", ")")[1:-1]
    params = top_level_split(strip_comments(params_text))
    _SIG_CACHE[bare] = (path, lineno, params)
    return _SIG_CACHE[bare]


def classify_call(name, args):
    """Compare a call's raw argument list against the BINDING shim's rule — arc 255 Stone
    P6-c-1, POST-STONE: the macro forwards exactly the context params the CALLEE declares, in
    the order the callee declares them (`sniff_args` now records an ORDERED sequence, not a
    `seen_context: bool`) — any subset of `env`/`sym`/`list_span`, not a fixed all-three
    `(env, sym, list_span)` triple. `crates/wat-macros/src/wat_intrinsic.rs`'s BINDING call
    sites (`#fn_name(args, #(#tail_tokens),*)` / `#fn_name(#(#arg_forwards,)*
    #(#tail_tokens),*)`) forward whatever `ContextParam` sequence `sniff_args` sniffed off the
    callee's OWN signature — the call-site's PRE-EXISTING trailing order (this text) is
    therefore no longer the thing to match against a fixed literal; it only needs to be A
    CONTIGUOUS SUFFIX drawn from `{env, sym, list_span}`, each at most once, in WHATEVER order
    already appears (that order is exactly what the callee's own declaration already commits
    to, since real Rust code passing the wrong positional type wouldn't compile today).
    Returns (label, reason)."""
    context_names = {"env", "sym", "list_span"}
    # Walk from the end while each token is a context name (P6-c-1: order-agnostic, subset
    # allowed — 0, 1, 2, or 3 of the three, no longer "all three or NEEDS-SHAPE").
    i = len(args)
    while i > 0 and args[i - 1] in context_names:
        i -= 1
    tail = args[i:]
    leading = args[:i]

    if len(set(tail)) != len(tail):
        return "NEEDS-SHAPE", f"trailing context args repeat a name: {tail!r}"
    if not tail:
        return (
            "NEEDS-SHAPE",
            "no env/sym/list_span in the trailing position at all — not a context tail this "
            "census recognizes (arity-0 delegate, or the context names sit somewhere other "
            "than a trailing run)",
        )
    if any(a in context_names for a in leading):
        return (
            "NEEDS-SHAPE",
            f"a context name appears before the end of the leading wat-args: {args!r} — "
            "`sniff_args` requires every `&WatAST`/`&[WatAST]` param before ANY context param",
        )
    # Order/subset now matches the call-site shape UNCONDITIONALLY (arc 255 Stone P6-c-1 — no
    # fixed triple to fail against). Still check the CALLEE's own declared signature for
    # reference-vs-owned on however many trailing params there are (the `.clone()`-hiding
    # case: `eval_apply` takes an OWNED `Span`, which the shim's borrowed local can never
    # satisfy — this stone changed WHICH params get forwarded and in what order, not their
    # by-ref-ness, so this half of the check is unchanged in substance).
    sig = find_fn_signature(name)
    if sig is None:
        return (
            "NEEDS-SHAPE?",
            f"call-site tail {tail!r} is a subset of env/sym/list_span but `{name}`'s own `fn` "
            "declaration could not be uniquely located to verify by-ref vs owned — read it by hand",
        )
    path, lineno, params = sig
    if len(params) < len(tail):
        return (
            "NEEDS-SHAPE",
            f"`{name}` at {path}:{lineno} declares only {len(params)} params, fewer than the "
            f"call's {len(tail)}-long tail",
        )
    last_n = params[-len(tail):]
    bad = []
    for p in last_n:
        # param text looks like "env: &Environment" — flag if it's NOT a reference type
        ty = p.split(":", 1)[1].strip() if ":" in p else p
        if not ty.startswith("&"):
            bad.append(p)
    if bad:
        return (
            "NEEDS-SHAPE",
            f"`{name}` at {path}:{lineno} takes {bad} BY VALUE, not by reference — the shim's "
            "env/sym/list_span locals are borrows and can never satisfy an owned param",
        )
    return (
        "INTRINSIC-READY",
        f"`{name}` at {path}:{lineno} declares context tail {tail!r} — the macro now honours "
        "it directly (arc 255 Stone P6-c-1: order/subset, no longer a fixed triple)",
    )


# ─── SHAPE vs DESTINATION (arc 255 Stone P6-c-2) ──────────────────────────────────────────────
# P6-c-1 widened the shape rule and the instrument started reading SHAPE=fits (INTRINSIC-READY)
# as if it were a verdict — it isn't. SHAPE ("does this signature fit #[wat_intrinsic]?") and
# DESTINATION ("should this verb BE an intrinsic at all?") are independent questions; only a
# human rules the second one. `classify_arm` below therefore reports SHAPE ONLY. The comment
# heuristic that used to let a "SPECIAL FORM" comment silently DECIDE the disposition is demoted
# to `comment_hint` — a boolean carried alongside the SHAPE label that can only flag a row for
# human review (main() prints it as a suggestion), never assign a disposition. `:wat::stream::lazy`
# kept its correct ruling by that comment's accident, not by design; see DESTINATION_LEDGER below,
# which is what actually carries that ruling now.
def detect_special_form_hint(pattern, body, leading):
    """Non-deciding SUGGESTION only (arc 255 Stone P6-c-2) — a 'special form' comment near an
    arm no longer assigns a disposition by itself. It can only ADD a candidate for human review;
    main() prints it as a note next to whatever SHAPE/DESTINATION this row already carries."""
    return bool(
        re.search(r"special form", body, re.IGNORECASE)
        or re.search(r"special form", pattern, re.IGNORECASE)
        or re.search(r"special form", leading, re.IGNORECASE)
    )


# THE FROZEN DESTINATION LEDGER. Names, never a count (`[[feedback_a_gate_freezes_names_never_a_count]]`
# — same discipline as FROZEN_CHECKER_DEBT_LEDGER / KNOWN_UNREVIEWED). Seeded from the rulings
# P6-c-0 made by hand-reading the arms, and the six/thirteen this NOTE records:
#   SPECIAL-FORM           — quote, quasiquote, fn (P6-c-0 hand ruling); stream::lazy (its own
#                            in-place comment, now carried HERE instead of re-derived from it).
#   DECLARATION-GUARD      — core::def, core::defclause: unconditional
#                            Err(DeclarationInExpressionPosition) arms; the real processing is
#                            freeze-time (register_runtime_defs_form). No shape to fix; likely
#                            disposition is delete, once that cut is confirmed exhaustive.
#   UNKNOWN-RULED-PENDING  — if, do, match, and the CONTROL-FLOW-MULTI-MODE set (let, and, or,
#                            ann-form): each has 3+ simultaneously-live implementations (the
#                            giant-match arm, a TCO trampoline, a stepper model, ...). Homing the
#                            eval arm alone strands the siblings. A real disposition (the
#                            serve-dispatch-op precedent at runtime.rs:4415) exists but must be
#                            CHOSEN, not stumbled into by this census.
# ★ This ledger MUST go red the moment a name here stops matching a real FQDN in the giant match
# — see `check_ledger_freshness()` in main(). Do not silently drop a stale name.
# arc 255 Stone P6-c-3 — every reason below is BACKFILLED from what P6-c-0's rider and the P6-c
# NOTE actually recorded, plus this stone's own re-read of the cited source lines (re-read only to
# CONFIRM the existing ruling, never to invent a new one — STOP trigger #1). Nothing here is a new
# ruling; all 13 FQDNs were already in this ledger before this stone touched it.
DESTINATION_LEDGER = {
    '":wat::core::quote"': (
        "SPECIAL-FORM",
        "P6-c-0 hand ruling: capture-don't-eval, mirrors lazy-seq. Confirmed at "
        "src/runtime.rs:11978-11986, eval_quote's own doc comment: \"the inner form is NOT "
        "evaluated at quote time — no side effects fire, no functions are called.\"",
    ),
    '":wat::core::quasiquote"': (
        "SPECIAL-FORM",
        "P6-c-0 hand ruling. Confirmed at src/runtime.rs:12219-12246, eval_quasiquote's own doc "
        "comment: walks the template and evaluates/substitutes only at explicit unquote sites, "
        "returning the assembled form as a Value::wat__WatAST — not a fully-evaluated call.",
    ),
    '":wat::core::fn"': (
        "SPECIAL-FORM",
        "P6-c-0 hand ruling. Confirmed at src/runtime.rs:5514-5517 (arm comment: \"the canonical "
        "operator for function values\") and src/function/eval.rs:20-30 (eval_fn's doc comment: "
        "it produces a Function closure value — the body is captured, not evaluated, until the "
        "function is later called).",
    ),
    '":wat::stream::lazy"': (
        "SPECIAL-FORM",
        "in-place comment: capture-don't-eval, mirrors quote (P6-c NOTE) — ruling now lives "
        "HERE, not derived from that comment. Confirmed at src/runtime.rs:5534-5536 (arm "
        "comment: \"lazy-seq is a SPECIAL FORM (capture-don't-eval)... Mirrors quote.\") and "
        "src/runtime.rs:12064 (eval_lazy_seq's own doc: \"SPECIAL FORM (capture-don't-eval)\").",
    ),
    '":wat::core::def"': (
        "DECLARATION-GUARD",
        "unconditional Err(DeclarationInExpressionPosition); real processing is freeze-time "
        "register_runtime_defs_form (P6-c NOTE). Confirmed at src/runtime.rs:5486-5497 (arm "
        "comment + the Err construction) and src/runtime.rs:2627 (register_runtime_defs_form, "
        "the freeze-time function that actually processes top-level defs).",
    ),
    '":wat::core::defclause"': (
        "DECLARATION-GUARD",
        "unconditional Err(DeclarationInExpressionPosition) (P6-c NOTE). Confirmed at "
        "src/runtime.rs:5501-5508 (\"Stone 237.2 — :wat::core::defclause at expression position "
        "is a position violation\" + the Err construction) and src/runtime.rs:2627 "
        "(register_runtime_defs_form, same freeze-time processor as core::def).",
    ),
    '":wat::core::if"': (
        "UNKNOWN-RULED-PENDING",
        "CONTROL-FLOW-MULTI-MODE: giant-match arm + eval_if_tail (TCO trampoline) + step_if "
        "(stepper model) are simultaneously live (P6-c NOTE, \"The census came back\" section, "
        "the CONTROL-FLOW-MULTI-MODE bullet — this is the one member of the set the NOTE named "
        "all three sites for by name; re-found at src/runtime.rs, fn eval_if_tail and fn "
        "step_if, both present, confirming the class still holds, though at drifted line "
        "numbers from the NOTE's own 4411/23501).",
    ),
    '":wat::core::do"': (
        "UNKNOWN-RULED-PENDING",
        "CONTROL-FLOW-MULTI-MODE (P6-c NOTE, same bullet as core::if) — grouped with if/let/"
        "match/and/or/ann-form as having 3+ simultaneously-live implementations; the NOTE gives "
        "the concrete site enumeration for if only, not per-verb for this one.",
    ),
    '":wat::core::match"': (
        "UNKNOWN-RULED-PENDING",
        "CONTROL-FLOW-MULTI-MODE (P6-c NOTE, same bullet as core::if) — grouped classification, "
        "not individually site-enumerated in the NOTE.",
    ),
    '":wat::core::let"': (
        "UNKNOWN-RULED-PENDING",
        "CONTROL-FLOW-MULTI-MODE (P6-c NOTE, same bullet as core::if) — grouped classification, "
        "not individually site-enumerated in the NOTE.",
    ),
    '":wat::core::and"': (
        "UNKNOWN-RULED-PENDING",
        "CONTROL-FLOW-MULTI-MODE (P6-c NOTE, same bullet as core::if) — grouped classification, "
        "not individually site-enumerated in the NOTE.",
    ),
    '":wat::core::or"': (
        "UNKNOWN-RULED-PENDING",
        "CONTROL-FLOW-MULTI-MODE (P6-c NOTE, same bullet as core::if) — grouped classification, "
        "not individually site-enumerated in the NOTE.",
    ),
    '":wat::core::ann-form"': (
        "UNKNOWN-RULED-PENDING",
        "CONTROL-FLOW-MULTI-MODE (P6-c NOTE, same bullet as core::if) — grouped classification, "
        "not individually site-enumerated in the NOTE.",
    ),
}

# arc 255 Stone P6-c-W1 — the campaign's first wave, and the first four rulings this ledger ever
# ADDED (as opposed to carried forward from P6-c-0/the NOTE), then the first four to be HOMED
# (registered + arm deleted) in the SAME stone that ruled them. Ruled `INTRINSIC` first —
# confirmed SHAPE=INTRINSIC-READY, checker-known (`check.rs:18604-18639`, `params: vec![]` for
# all four — so homing added no `FROZEN_CHECKER_DEBT_LEDGER` debt), absent from
# `KNOWN_UNREVIEWED` (`src/rete/purity.rs`), single-dispatch-site (the only other grep hits per
# FQDN were the handler's own `check_nullary` call, one `require_encoding_ctx` call (global-seed
# only), a `resolve/mod.rs` `is_reserved_prefix` test assertion (dim-count only), and the
# TypeScheme registration itself) — then homed to `src/intrinsic/config.rs`, arm deleted
# (`runtime.rs:6290-6293` retired to a comment, per the NOTE's "retiring an arm means registering
# the verb and deleting the arm").
#
# ★ THEY ARE DELIBERATELY NOT LEFT AS `DESTINATION_LEDGER` ROWS. `check_ledger_freshness` (below)
# FATALs the instant a ledgered FQDN stops matching a literal pattern in the giant match — by
# design, to catch silent DRIFT. But an arm that is GONE because it was successfully, intentionally
# homed is not drift; it is the ledger's row completing its purpose. Every OTHER already-homed
# family (`:wat::math::*`, `:wat::stat::*`, the seven `:wat::kernel::` ambient verbs, …) was never
# entered here at all, for the identical reason: `DESTINATION_LEDGER`'s population is implicitly
# "verbs still dispatched from the giant match", not "every verb this campaign has ever ruled on".
# These four are simply the FIRST case where a rule-then-home round-trip happened inside one wave
# instead of across two — carrying their rows forward once the arm is gone would only ever trip
# this stale-name FATAL for a reason that is not a defect. The four rulings themselves (frozen
# here for citation, since the ledger dict above can no longer anchor them once their arm is
# gone; superseding code: `src/intrinsic/config.rs`):
#
#   :wat::config::dim-count — INTRINSIC. Nullary read of committed startup config:
#     `sym.encoding_ctx()` -> `ctx.dim_count`, falling back to `crate::config::DEFAULT_DIM_COUNT`
#     when no ctx is attached. No side effect, no multi-site implementation, no special-form
#     semantics. Confirmed at (pre-image) src/runtime.rs:20167-20182 (`eval_config_dim_count`'s
#     own doc: "Config accessors — read committed config fields at runtime"; body is
#     arity-check -> match -> return, nothing else); single dispatch site was runtime.rs:6290;
#     TypeScheme at src/check.rs:18604-18612 (`params: vec![]`, `ret: i64_ty()` — checker already
#     agreed arity is 0).
#   :wat::config::dim-capacity — INTRINSIC. Nullary read of committed startup config:
#     `sym.encoding_ctx()` -> `ctx.capacity`, falling back to `kanerva_capacity(DEFAULT_DIM_COUNT)`
#     when no ctx is attached. No side effect. Confirmed at (pre-image) src/runtime.rs:20183-20200
#     (`eval_config_dim_capacity`'s own doc: "Hologram-slot count for this program... Cached at
#     freeze; reads from `EncodingCtx`"); single dispatch site was runtime.rs:6291; TypeScheme at
#     src/check.rs:18613-18621 (`params: vec![]`, `ret: i64_ty()`).
#   :wat::config::global-seed — INTRINSIC. Nullary read of the committed atom-seeding seed:
#     `require_encoding_ctx` -> `ctx.config.global_seed`. No side effect (a required-ctx read,
#     not a mutation). Confirmed at (pre-image) src/runtime.rs:20223-20231
#     (`eval_config_global_seed`'s own doc: "committed atom-seeding seed as `:i64`"); single
#     dispatch site was runtime.rs:6292; TypeScheme at src/check.rs:18622-18630 (`params: vec![]`,
#     `ret: i64_ty()`).
#   :wat::config::noise-floor — INTRINSIC. Nullary read: `1/sqrt(dim-count)` at the program's
#     committed `d` (`sym.encoding_ctx()` or `DEFAULT_DIM_COUNT`). No side effect. Confirmed at
#     (pre-image) src/runtime.rs:20204-20219 (`eval_config_noise_floor_default_shim`'s own doc:
#     "`1/sqrt(dim-count)` at the program's `d`. Held for legacy callers"); single dispatch site
#     was runtime.rs:6293; TypeScheme at src/check.rs:18631-18639 (`params: vec![]`,
#     `ret: f64_ty()`).

# arc 255 Stone P6-c-W2 — the campaign's second wave: five candidates named
# (`:wat::stream::{empty,cons,next}`, `:wat::program::env`, `:wat::stdlib::sources`), FOUR
# ruled and homed. `:wat::stdlib::sources` is NOT one of them — STOP-A fired: its handler
# (`crate::io::eval_stdlib_sources`) returns `Result<Value, RuntimeError>`, and
# `crates/wat-macros/src/wat_intrinsic.rs::validate_return_type` (~line 371) rejects any
# handler return type other than `Result<Value, EvalBreak>` / `Result<TrackedValue,
# EvalBreak>` with a `compile_error!` — homing it AS WRITTEN would not compile, and coercing
# its return type is exactly the "cleanly" trapdoor the brief named in advance. Dropped from
# the wave; the pre-image dispatch arm and handler at src/io.rs:1837 are UNTOUCHED. Its
# `KNOWN_UNREVIEWED` line (`src/rete/purity.rs`) is therefore also untouched — the ledger
# shrinks by 3 this wave (the three stream verbs), not 4; `:wat::program::env` was never on
# that ledger (namespace-disposed `Impure` by `RULES` regardless of homing), so it contributes
# 0 either way. A different number than the brief's prediction (4) is a FINDING, not an error:
#   :wat::stream::empty — INTRINSIC. Zero-arg constructor, `Value::wat__stream__Stream(Arc::new
#     (Stream::Empty))`. No side effect, no multi-site implementation. Confirmed at (pre-image)
#     src/runtime.rs:11994-11999 (`eval_seq_empty`'s own doc: "Zero-arg constructor... Empty
#     terminator"); single dispatch site was runtime.rs:5532; TypeScheme at src/check.rs:20977-
#     20985 (`params: vec![]`, `ret: seq_t()` — checker already agreed arity is 0).
#   :wat::stream::cons — INTRINSIC. Pure reshape: evaluates `head`/`tail`, stores exactly what
#     it is handed as a new `Stream::Cons` cell; never enters `tail` to look inside (forcing is
#     `next`'s job). Confirmed at (pre-image) src/runtime.rs:12013-12019 (`eval_cons`'s own
#     doc: "Strict-head Cons cell... Returns a Stream::Cons{head,tail}"); single dispatch site
#     was runtime.rs:5533; TypeScheme at src/check.rs:20986-20994 (`params: vec![t_var(),
#     seq_t()]`, `ret: seq_t()`).
#   :wat::stream::next — INTRINSIC, but NOT `Pure`/`Deterministic` — a genuine judgement, not a
#     copy of cons/empty's. Forces a thunk via `crate::stream::realize`, which calls
#     `apply_function` on a captured wat closure (a `Thunk`) or runs a Rust closure (a
#     `NativeThunk`, backing lazy `map`/`filter`/`take`/`drop`) — either can run ARBITRARY code
#     this verb has no way to bound, exactly the shape `:wat::core::apply`/`:wat::eval` are
#     deliberately left unclassified for below ("purity is the form's, like apply"). Homed
#     `@Purity Effectful @Determinism Nondeterministic` (`src/intrinsic/stream.rs`).
#     Independent corroboration: `src/macros/eval.rs`'s `is_pure_total` expand-time-safe
#     allowlist already listed `cons`/`empty`/`lazy` and already did NOT list `next`, before
#     this stone touched either file. Confirmed at (pre-image) src/runtime.rs:12130-12142
#     (`eval_stream_next`'s own doc + body: forces via `realize`, destructures); single
#     dispatch site was runtime.rs:5539; TypeScheme at src/check.rs:20996-21009 (`params:
#     vec![seq_t()]`).
#   :wat::program::env — INTRINSIC. Nullary ambient read: `current_program_env()` reads a
#     `RefCell` thread-local installed ONCE per thread at a fixed pre-`:user::main` seam
#     (`install_program_env`) and never mutated afterward — the identical "committed-once,
#     read-many" shape `sym.encoding_ctx()` has for `:wat::config::dim-count` (W1, above). No
#     side effect. Confirmed at (pre-image) src/runtime.rs:19882-19890 (`eval_program_env`'s
#     own doc: "returns a clone of the current value, or a clean MalformedForm error");
#     single dispatch site was runtime.rs:5582; TypeScheme at src/check.rs:18689-18696
#     (`params: vec![]`, `ret: TypeExpr::Path(":wat::program::Env")`).
# Instrument, run against the pre-image (all five arms still present) with these four rows
# temporarily added to DESTINATION_LEDGER: HOMEABLE 0 -> 4 of 142, AWAITING 107 -> 103 of 142
# (RULED OUT unchanged at 13). Post-home, the ledger carries none of the four (same reasoning
# as W1: `check_ledger_freshness` FATALs the instant a ruled FQDN leaves the match, which is
# exactly what a successful home does) — HOMEABLE returns to 0; the population shrinking
# (142 -> 138) is the real meter, per W1's own correction.

# arc 255 Stone P6-c-3 — DEFAULT-DENY. `DESTINATION_DEFAULT` used to be `"INTRINSIC"`: a verb
# nobody had ruled on read as homeable by SILENCE — the exact shape of
# `if is_reserved_prefix(head) { return true }` (src/resolve/walk.rs:268) this whole arc exists to
# kill. It is now `"UNRULED"`, and `UNRULED` is NEVER homeable — see `HOMEABLE_DESTINATION` below,
# which is the only destination value that ever enters the homeable set, and which nothing is
# EVER given by default. A verb becomes homeable only by an explicit `(fqdn, "INTRINSIC", reason)`
# ruling added to DESTINATION_LEDGER by a human who read it — this stone adds none.
DESTINATION_DEFAULT = "UNRULED"
DESTINATION_DEFAULT_REASON = (
    "not in the frozen ledger — UNRULED, and UNRULED is never homeable (arc 255 Stone P6-c-3: "
    "default-deny). A human must read this verb and add a (fqdn, destination, reason) ruling to "
    "DESTINATION_LEDGER above before it can enter the homeable set — silence is no longer a ruling"
)
# The one destination value the homeable set actually keys on. Kept as a separate name from
# DESTINATION_DEFAULT (which is UNRULED) so the two can never accidentally collapse back into each
# other the way `"INTRINSIC"` used to double as both "the default" and "the homeable value".
HOMEABLE_DESTINATION = "INTRINSIC"

# Reasons that are present but carry no actual content — a name in the reason slot instead of a
# sentence. STOP trigger #2 (BRIEF-STONE-P6-c-3): "a reason you would have to invent" is a STOP;
# this list is the mechanical half of that check (empty/whitespace-only/boilerplate), not a
# substitute for reading each of the 13 reasons by eye (done separately, in the stone's report).
_PLACEHOLDER_REASONS = {
    "", "todo", "tbd", "fixme", "wip", "n/a", "na", "none", "unknown", "...", "tk", "xxx",
}


def destination_for(fqdn_literal):
    """READ the frozen ledger; never derive from SHAPE. A row not in the ledger gets the
    constant default above — a fixed prior, not a computation over this row's own shape data."""
    return DESTINATION_LEDGER.get(fqdn_literal, (DESTINATION_DEFAULT, DESTINATION_DEFAULT_REASON))


def validate_ledger():
    """arc 255 Stone P6-c-3: a ledger row is a (destination, reason) PAIR and the reason is
    load-bearing — a name alone is a name on a list, which is what this stone exists to stop.
    FATAL, naming the exact row, on: a malformed entry, a missing/blank destination, or a
    missing/empty/placeholder reason. Runs before anything else in main() so a corrupted ledger
    can never silently produce a report."""
    for fqdn, entry in DESTINATION_LEDGER.items():
        if not (isinstance(entry, tuple) and len(entry) == 2):
            print("\n## ⛔ FATAL — DESTINATION LEDGER ROW IS MALFORMED", file=sys.stderr)
            print(
                f"    {fqdn} : {entry!r} is not a (destination, reason) pair", file=sys.stderr
            )
            sys.exit(1)
        dest, reason = entry
        if not isinstance(dest, str) or not dest.strip():
            print("\n## ⛔ FATAL — DESTINATION LEDGER ROW HAS NO DESTINATION", file=sys.stderr)
            print(f"    {fqdn} : destination is {dest!r}", file=sys.stderr)
            sys.exit(1)
        if not isinstance(reason, str) or reason.strip().lower() in _PLACEHOLDER_REASONS:
            print("\n## ⛔ FATAL — DESTINATION LEDGER ROW HAS NO REASON", file=sys.stderr)
            print(
                f"    {fqdn} : destination={dest!r} but reason={reason!r} — a ruling is a "
                "(destination, reason) PAIR (arc 255 Stone P6-c-3); a name alone is not a ruling",
                file=sys.stderr,
            )
            sys.exit(1)


def check_ledger_freshness(all_fqdns):
    """STOP trigger #1 (BRIEF-STONE-P6-c-2): a name in the frozen ledger that no longer appears
    in the match must fail LOUDLY, naming it — never silently skip it. Call with the FULL FQDN
    population (pre `--control` filtering)."""
    stale = sorted(k for k in DESTINATION_LEDGER if k not in all_fqdns)
    if stale:
        print("\n## ⛔ FATAL — DESTINATION LEDGER IS STALE", file=sys.stderr)
        for s in stale:
            print(
                f"    ledgered FQDN {s} no longer appears in the giant match's FQDN set — "
                "update the ledger or re-verify this FQDN's fate before trusting this report",
                file=sys.stderr,
            )
        sys.exit(1)


def classify_arm(pattern, body, leading=""):
    """SHAPE ONLY. Returns (shape_label, shape_reason, comment_hint) — DESTINATION is looked up
    separately, per-FQDN, from DESTINATION_LEDGER (see destination_for()), never computed here."""
    guard = find_prefix_guard(pattern)
    if guard is not None:
        return "PREFIX-GUARD", f"namespace guard on prefix {guard!r}, not enumerable FQDNs", False
    fq = extract_fqdns(pattern)
    if not fq:
        return "CATCH-ALL", "bare wildcard/bound-name arm with no literal FQDN in its pattern", False
    hint = detect_special_form_hint(pattern, body, leading)
    call = find_primary_call(body)
    if call is None:
        return (
            "COMPLEX",
            "arm body is not one delegating call (multi-statement / inline control flow) — read by hand",
            hint,
        )
    name, args, _rest = call
    label, reason = classify_call(name, args)
    return label, reason, hint


def load_giant_match():
    with open(RUNTIME_RS) as f:
        lines = f.readlines()
    # Locate the function and its `match head {` deterministically instead of hardcoding
    # line numbers (line numbers drift; text anchors don't).
    fn_start = None
    for i, l in enumerate(lines):
        if re.match(r"fn dispatch_keyword_head_value\s*\(", l):
            fn_start = i
            break
    if fn_start is None:
        sys.exit("FATAL: could not find `fn dispatch_keyword_head_value(` in src/runtime.rs")
    # preamble text: from fn_start to the `match head {` that starts the giant match
    match_start = None
    for i in range(fn_start, len(lines)):
        if re.match(r"\s*match head \{", lines[i]):
            match_start = i
            break
    if match_start is None:
        sys.exit("FATAL: could not find the giant `match head {` after dispatch_keyword_head_value")
    preamble_text = "".join(lines[fn_start:match_start])

    # find the matching close of `match head { ... }` by brace-balance from match_start
    text_from_match = "".join(lines[match_start:])
    sc = Scanner(text_from_match)
    sc.i = text_from_match.index("{")
    sc.consume_balanced("{", "}")
    match_close_offset_in_segment = sc.i  # position right after the closing '}'
    match_block_text = text_from_match[: match_close_offset_in_segment]
    body_inner = match_block_text[match_block_text.index("{") + 1 : match_block_text.rindex("}")]

    base_line = match_start + 1  # 1-indexed line of `match head {`
    offset_to_line = build_offset_to_line(body_inner, base_line)
    arms = parse_arms(body_inner, offset_to_line)

    end_line = base_line + body_inner.count("\n")
    return preamble_text, match_start + 1, end_line, arms


def find_nested_dispatch(body):
    """Return list of (inner_arms) for every `match head {` found inside an arm's body."""
    out = []
    for m in re.finditer(r"match\s+head\s*\{", body):
        sc = Scanner(body)
        sc.i = m.end() - 1
        block = sc.consume_balanced("{", "}")
        inner = parse_arms(block[1:-1], None)
        out.append(inner)
    return out


PREAMBLE_SITES = [
    (
        r'if head == ":wat::rete::insert" \{',
        ':wat::rete::insert',
        "pre-match short-circuit: 2-ary routes to eval_insert_public; 3+-ary handled by the "
        "':wat::rete::insert-all' arm inside the match. A line-anchored grep over the match "
        "body cannot see this — it fires BEFORE the match even starts.",
    ),
    (
        r"if head\.starts_with\(crate::rete::vocabulary::RETE_PREFIX\)",
        None,
        "prefix gate + RETE_OPS table lookup (rete_op_for) — routes `where`-clause VOCABULARY "
        "operators (':wat::rete::i64::>'-shaped), a DIFFERENT population from the 28 engine-verb "
        "arms in the match body (fire-rules, insert-all, export, lower, ...). Verified: zero "
        "string overlap between RETE_OPS's rete_name column and the match's own FQDN set.",
    ),
    (
        r"if let Some\(handler\) = crate::intrinsic::registry\(\)\.lookup\(head\)",
        None,
        "the registry-first door, consulted a second time here (dispatch_keyword_head already "
        "consults it before ever calling this function) — any FQDN already homed here wins "
        "before the match is reached at all, silently making its own match arm (if not yet "
        "retired) dead code.",
    ),
]


def scan_preamble(preamble_text):
    hits = []
    for regex, fqdn, note in PREAMBLE_SITES:
        m = re.search(regex, preamble_text)
        hits.append({"found": bool(m), "fqdn": fqdn, "note": note, "pattern": regex})
    return hits


def multi_site_grep(fqdn_literal, home_line):
    """Grep the whole src/ tree for this FQDN literal appearing in a dispatch-shaped context
    OUTSIDE its home line in runtime.rs. Returns list of "path:line: text" hits."""
    try:
        out = subprocess.run(
            ["grep", "-rn", "-F", fqdn_literal, str(REPO_ROOT / "src")],
            capture_output=True,
            text=True,
            timeout=30,
        ).stdout
    except Exception:
        out = ""
    hits = []
    dispatch_shape = re.compile(r"=>|==|starts_with|matches!")
    for line in out.splitlines():
        try:
            path, lineno, rest = line.split(":", 2)
        except ValueError:
            continue
        if path.endswith("runtime.rs") and int(lineno) == home_line:
            continue
        if dispatch_shape.search(rest):
            hits.append(line)
    return hits


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--control", help="comma-separated FQDN literals to restrict the report to")
    ap.add_argument("--json", help="also dump the full structured table to this path")
    ap.add_argument("--no-multisite", action="store_true", help="skip the (slower) multi-site grep pass")
    args = ap.parse_args()

    # arc 255 Stone P6-c-3 — a corrupted ledger (missing/empty/placeholder reason, or a malformed
    # row) FATALs before a single line of the report is printed.
    validate_ledger()

    preamble_text, match_start_line, match_end_line, arms = load_giant_match()

    print(f"# giant match: src/runtime.rs:{match_start_line}-{match_end_line}")
    print(f"# top-level arms parsed: {len(arms)}\n")

    print("## preamble dispatch sites (before the match even starts)")
    for hit in scan_preamble(preamble_text):
        status = "FOUND" if hit["found"] else "NOT FOUND (drifted? read by hand)"
        print(f"  [{status}] {hit['pattern']}")
        print(f"      {hit['note']}")
    print()

    control_set = None
    if args.control:
        control_set = {f.strip() for f in args.control.split(",")}

    rows = []
    total_fqdns = 0
    for arm in arms:
        label, reason, hint = classify_arm(arm["pattern"], arm["body"], arm.get("leading", ""))
        fq = extract_fqdns(arm["pattern"])
        line_desc = f"{arm['pattern_start_line']}-{arm['arrow_line']}"
        if label in ("PREFIX-GUARD", "CATCH-ALL"):
            rows.append(
                {
                    "fqdn": None,
                    "lines": line_desc,
                    "shape": label,
                    "reason": reason,
                    "hint": hint,
                    "pattern": arm["pattern"].strip()[:80],
                }
            )
            continue
        total_fqdns += len(fq)
        nested_clusters = find_nested_dispatch(arm["body"])
        if nested_clusters:
            for inner_arms in nested_clusters:
                for ia in inner_arms:
                    ifq = extract_fqdns(ia["pattern"])
                    ilabel, ireason, ihint = classify_arm(
                        ia["pattern"], ia["body"], ia.get("leading", "")
                    )
                    for f in ifq:
                        rows.append(
                            {
                                "fqdn": f,
                                "lines": line_desc + " (nested cluster)",
                                "shape": ilabel,
                                "reason": ireason,
                                "hint": ihint,
                                "pattern": arm["pattern"].strip()[:80],
                            }
                        )
            continue
        for f in fq:
            rows.append(
                {
                    "fqdn": f,
                    "lines": line_desc,
                    "shape": label,
                    "reason": reason,
                    "hint": hint,
                    "pattern": None,
                }
            )

    # ★ STOP trigger #1 — the ledger must go red BEFORE anything else is printed or counted,
    # and it must be checked against the FULL FQDN population, not a `--control`-filtered one.
    all_fqdn_set = {r["fqdn"] for r in rows if r["fqdn"] is not None}
    check_ledger_freshness(all_fqdn_set)

    # DESTINATION is read per-FQDN from the frozen ledger — never derived from SHAPE.
    # arc 255 Stone P6-c-3 — DEFAULT-DENY: three DISJOINT-BY-CONSTRUCTION buckets, each keyed off
    # DESTINATION, not SHAPE alone:
    #   HOMEABLE          shape fits  AND a human RULED this fqdn INTRINSIC (never the default —
    #                     DESTINATION_DEFAULT is UNRULED, so this can only ever come from an
    #                     explicit DESTINATION_LEDGER entry).
    #   AWAITING A RULING shape fits  AND nobody has ruled on it at all (dest == UNRULED) — the
    #                     worklist; this is the number that used to be silently "homeable".
    #   RULED OUT         a human ruled this fqdn to something OTHER than INTRINSIC — counted
    #                     regardless of SHAPE, because the ruling (not the shape) is what took it
    #                     out of contention (three of the 13 — def, defclause, let — don't even
    #                     have SHAPE=fits, so they were invisible to the old
    #                     shape_fits_total-minus-homeable arithmetic; they are still rulings).
    homeable_count = 0
    awaiting_count = 0
    ruled_out_count = 0
    for r in rows:
        if r["fqdn"] is None:
            r["dest"], r["dest_reason"] = None, None
            r["homeable"] = False
            continue
        r["dest"], r["dest_reason"] = destination_for(r["fqdn"])
        r["homeable"] = r["shape"] == "INTRINSIC-READY" and r["dest"] == HOMEABLE_DESTINATION
        if r["homeable"]:
            homeable_count += 1
        elif r["dest"] == DESTINATION_DEFAULT:
            if r["shape"] == "INTRINSIC-READY":
                awaiting_count += 1
        else:
            ruled_out_count += 1

    if control_set:
        rows = [r for r in rows if r["fqdn"] in control_set]

    print(f"## per-FQDN candidate disposition ({len(rows)} rows)")
    for r in rows:
        fqdn_disp = r["fqdn"] if r["fqdn"] else f"<{r['shape']} arm: {r['pattern']}>"
        if r["dest"] is None:
            print(f"  {fqdn_disp:55s} [{r['lines']:>14s}]  SHAPE={r['shape']:16s}  {r['reason']}")
            continue
        homeable_tag = "HOMEABLE" if r["homeable"] else "--------"
        print(
            f"  {fqdn_disp:55s} [{r['lines']:>14s}]  {homeable_tag}  "
            f"SHAPE={r['shape']:16s} DESTINATION={r['dest']:22s}  shape:{r['reason']}"
        )
        print(f"      destination: {r['dest_reason']}")
        if r["hint"] and r["dest"] != "SPECIAL-FORM":
            print(
                "      ⚠ comment-hint: a 'special form' comment appears near this arm but it is "
                "NOT in the frozen ledger as SPECIAL-FORM — SUGGESTION ONLY, add to human review, "
                "does not change SHAPE or DESTINATION above"
            )

    # arc 255 Stone P6-c-3 — three counts, unmistakably. HOMEABLE is the only one anyone may act
    # on; AWAITING A RULING is the worklist every later wave draws from; RULED OUT is a human
    # decision already made and carried forward, not recomputed here.
    print(f"\n## HOMEABLE:          {homeable_count} of {len(all_fqdn_set)} FQDNs "
          f"(ruled {HOMEABLE_DESTINATION} AND SHAPE=INTRINSIC-READY — NEVER the default)")
    print(f"## AWAITING A RULING: {awaiting_count} of {len(all_fqdn_set)} FQDNs "
          f"(SHAPE=INTRINSIC-READY AND DESTINATION={DESTINATION_DEFAULT} — the worklist)")
    print(f"## RULED OUT:         {ruled_out_count} of {len(all_fqdn_set)} FQDNs "
          f"(DESTINATION ruled to something other than {HOMEABLE_DESTINATION}, any SHAPE)")
    shape_fits_count = sum(1 for r in rows if r.get("shape") == "INTRINSIC-READY" and r["fqdn"] is not None)
    if not control_set:
        print(
            f"      SHAPE=fits total: {shape_fits_count}  =  HOMEABLE {homeable_count}  +  "
            f"AWAITING-A-RULING {awaiting_count}  +  (ruled-out rows that also happen to fit: "
            f"{shape_fits_count - homeable_count - awaiting_count})"
        )

    if not args.no_multisite:
        print("\n## multi-site grep (candidate — every hit needs a human read)")
        home_line_map = {}
        for arm in arms:
            for f in extract_fqdns(arm["pattern"]):
                home_line_map.setdefault(f, arm["arrow_line"])
        for f, home_line in home_line_map.items():
            if control_set and f not in control_set:
                continue
            hits = multi_site_grep(f, home_line)
            if hits:
                print(f"  {f} (home: runtime.rs:{home_line}) — {len(hits)} other candidate site(s):")
                for h in hits:
                    print(f"      {h}")

    if args.json:
        with open(args.json, "w") as fh:
            json.dump(rows, fh, indent=2)
        print(f"\n(structured table written to {args.json})")


if __name__ == "__main__":
    main()
