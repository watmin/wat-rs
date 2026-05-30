# BRIEF — argspec WARD — Pattern A retrofit + annihilate all vigilia findings (one stone)

You are sonnet. Ward the `src/argspec/` home. The 8-spell vigilia found the home's CONCEPT-duplication already annihilated (arc 241 unified the 4 parsers — confirmed), but surfaced live failure domains, the chief being: **`ArgSpecError` is a flat enum, NOT Pattern A** — the very hand-disciplined-span precedent that INSPIRED Pattern A has never taken the cure. This stone cures it. ONE stone: retrofit Pattern A + annihilate every finding → the home earns `vigilatum`.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## ‼️ COMMIT DISCIPLINE ‼️
ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits. `git status`/`git diff`/`git grep` READ-ONLY only. Orchestrator commits atomically after a vigilia re-cast. If you think a commit is needed, STOP and say so. (#1 rejection trigger — a prior sweep breached it.)

## ‼️ ANCHOR AGAINST THE LIVE FILE ‼️
The line numbers in this brief are approximate (the orchestrator's read returned some garbled line numbers). For every edit, `grep`/read the LIVE file to confirm the exact site before editing. The WHAT and the target code shapes below are exact; the line cites are hints.

## Read first
1. `docs/CONFORMARE.md` — Pattern A doctrine (outer struct { span, kind } + kind enum; location structurally imposed)
2. `docs/VIGILATUM.md` — the ward marker you're earning
3. `src/argspec/error.rs` (the error type — retrofit target) + `src/argspec/parse.rs` (the emitters) in full

## Pre-spawn state
HEAD `bbf670d8`. Working tree clean. Gates baseline: lib 890/0; function 8/0; clippy ≤894; workspace clean. The home is mod.rs (65) + parse.rs (~181) + error.rs (~108).

CONTEXT — the conformare probe already established (do NOT re-investigate):
- The 4-parser concept-duplication (A1-A4) is ANNIHILATED; `parse_argspec_triples` is the sole parser. Confirmed by all spells.
- `From<ArgSpecError> for CheckError` is LIVE (triggered at infer.rs:71 via the A3 diagnostic parser). NOT dead. Do NOT touch/delete it (purgare's "dead" finding was probe-disconfirmed).
- The `.map_err(|_| ())` classifier-probe in function/parse.rs already got its earned `rune:sequi(reclassified-by-caller)` (commit bbf670d8). Out of scope here.

## PHASE A — Pattern A retrofit (the spine)

`ArgSpecError` is currently a flat enum where EVERY variant carries `span: Span` AND `head: String`. Both fields are universal (every variant has them; every emitter passes them). Pattern A moves the universal fields to an outer struct; the kind enum carries only what VARIES.

### A1 — reshape error.rs

Current: `pub enum ArgSpecError { NameNotSymbol { span, head }, ... 7 variants ... }`

Target:
```rust
/// Canonical argspec parse error (Pattern A): outer struct carries the
/// universal span + head; the kind enum carries only variant-specific data.
/// span is structurally imposed — a spanless argspec error is uncompilable.
#[derive(Debug, Clone)]
pub struct ArgSpecError {
    pub span: Span,
    pub head: String,
    pub kind: ArgSpecErrorKind,
}

#[derive(Debug, Clone)]
pub enum ArgSpecErrorKind {
    NameNotSymbol,
    MissingArrow,
    TypeNotKeyword,
    MalformedTypeKeyword { inner: Box<TypeError> },
    TrailingItems { count: usize },
    IncompleteTriple,
    RestBinderNotSupported,
}
```
(NameNotSymbol/MissingArrow/TypeNotKeyword/IncompleteTriple/RestBinderNotSupported become UNIT variants — they carried only span+head, which are now outer. Only MalformedTypeKeyword + TrailingItems keep variant-specific data.)

Update the module doc (lines 1-9): drop the "every variant carries span (per AUDIT.md line 161)" convention-language — span is now STRUCTURAL, not conventional. State the Pattern A shape + cite Stone 243.3 / CONFORMARE.md. The whole point: the convention this home documented is now elevated to structure (this home was Pattern A's founding precedent; it now takes the cure).

### A2 — `kind.reason()` replaces the reason-half of into_parts

The current `into_parts(self) -> (Span, String, String)` maps each variant to its reason string. Replace with a method on the KIND:
```rust
impl ArgSpecErrorKind {
    /// The human-readable reason for this failure shape.
    fn reason(&self) -> String {
        match self {
            ArgSpecErrorKind::NameNotSymbol => "argument name must be a bare symbol".into(),
            ArgSpecErrorKind::MissingArrow => "missing `<-` arrow in argument triple".into(),
            ArgSpecErrorKind::TypeNotKeyword => "argument type must be a keyword".into(),
            ArgSpecErrorKind::MalformedTypeKeyword { inner } => format!("malformed type keyword: {}", inner),
            ArgSpecErrorKind::TrailingItems { count } => format!("{} trailing item(s) after rest binder", count),
            ArgSpecErrorKind::IncompleteTriple => "incomplete argument triple (expected `name <- :Type`)".into(),
            ArgSpecErrorKind::RestBinderNotSupported => "rest binder `&` not supported here".into(),
        }
    }
}
```
Delete the old `into_parts()` (its span/head extraction collapses to direct field access on the outer struct).

### A3 — the From impls (now FOUR consumers; all ride the new shape)

All become direct field access — no match needed:
```rust
impl From<ArgSpecError> for crate::runtime::RuntimeError {
    fn from(e: ArgSpecError) -> Self {
        Self::MalformedForm { head: e.head, reason: e.kind.reason(), span: e.span }
    }
}
impl From<ArgSpecError> for crate::types::TypeError {
    fn from(e: ArgSpecError) -> Self {
        crate::types::TypeError {
            span: e.span,
            kind: crate::types::TypeErrorKind::MalformedDecl { head: e.head, reason: e.kind.reason() },
        }
    }
}
impl From<ArgSpecError> for crate::check::CheckError {
    fn from(e: ArgSpecError) -> Self {
        Self::MalformedForm { head: e.head, reason: e.kind.reason(), span: e.span, remedies: vec![] }
    }
}
```
NOTE the ordering: compute `e.kind.reason()` BEFORE moving `e.head`/`e.span` out, OR bind reason first (`let reason = e.kind.reason();`) — `reason()` borrows `&self.kind`, then the struct fields move. Let the borrow-checker guide; if it objects, bind reason first.

### A4 — `From<ArgSpecError> for MacroError` (solvere L1 — the missing impl)

Currently macros.rs (~458-487) hand-matches every ArgSpecError variant to build `MacroError::MalformedDefmacro`, DUPLICATING the reason strings (and its comment cites the ghost `classify()`). Add the canonical impl in error.rs:
```rust
impl From<ArgSpecError> for crate::macros::MacroError {
    fn from(e: ArgSpecError) -> Self {
        // verify the exact MacroError variant + field names against macros.rs
        crate::macros::MacroError::MalformedDefmacro { reason: e.kind.reason(), span: e.span }
    }
}
```
Then in macros.rs: replace the ~30-line `.map_err(|e| { match e { ... } })` block at the `parse_argspec_triples(...)` call (~453-487) with `.map_err(crate::macros::MacroError::from)?` (or just `?` if the surrounding fn returns `Result<_, MacroError>`). The `classify()` ghost-comment dies with the block. VERIFY the exact MacroError variant/fields by reading macros.rs first.

### A5 — emitter cascade in parse.rs

Every `ArgSpecError::Variant { span: X, head: head.to_string() }` becomes `ArgSpecError { span: X, head: head.to_string(), kind: ArgSpecErrorKind::Variant {...} }`. The ~7 emitter sites:
- RestBinderNotSupported (~48): `ArgSpecError { span: args_vec[idx].span().clone(), head: head.to_string(), kind: ArgSpecErrorKind::RestBinderNotSupported }`
- IncompleteTriple (~60, ~103): `... kind: ArgSpecErrorKind::IncompleteTriple`
- MissingArrow (~70, ~137): `... kind: ArgSpecErrorKind::MissingArrow`
- NameNotSymbol (~78, ~130): `... kind: ArgSpecErrorKind::NameNotSymbol`
- TrailingItems (~91): `... kind: ArgSpecErrorKind::TrailingItems { count: ... }`
- the `parse_keyword_type` closure (`|span, head| ArgSpecError::TypeNotKeyword { span, head }`): becomes `|span, head| ArgSpecError { span, head, kind: ArgSpecErrorKind::TypeNotKeyword }`
- MalformedTypeKeyword (~160 in parse_keyword_type): `ArgSpecError { span: kw_span.clone(), head: head.to_string(), kind: ArgSpecErrorKind::MalformedTypeKeyword { inner: Box::new(e) } }`
Substrate-as-teacher: the compiler names every site. VERIFY no `Span::unknown()` appears (the probe confirmed every emitter supplies a concrete span — keep it that way; NO spanless-by-domain rune is needed, unlike TypeError's CyclicSubtype).

## PHASE B — the remaining findings

### B1 (intueri L2) — promote `is_bare_symbol`, kill the arrow-detection duplication
`is_bare_symbol` (parse.rs ~178) is private; two callers inline its pattern instead of using it:
- `function/parse.rs:~82`: `WatAST::Symbol(s, _) if s.as_str() == "->" => {}`
- `macros.rs:~431`: `WatAST::Symbol(s, _) if s.as_str() == "->" => {}`
Promote `is_bare_symbol` to `pub(crate)`, re-export from `mod.rs` (`pub use parse::is_bare_symbol;` or via the existing `pub use`). Replace both inline patterns with `is_bare_symbol(node, "->")`. VERIFY each call site's surrounding match still compiles (the inline was a match-arm guard; the replacement is an `if is_bare_symbol(...)` — adapt the control flow honestly, don't force a match arm into an if).

### B2 (intueri/solvere/purgare L2) — `is_bare_symbol` doc drop "->"
parse.rs ~176: doc says `"<-"`, `"->"`, and `"&"`. Within argspec, `is_bare_symbol` is only called with `"<-"` and `"&"` (the `"->"` use is the caller's, post-B1 promotion). After B1, `"->"` IS used via the promoted callers — so KEEP "->" in the doc if B1 makes external callers use it; DROP it only if it remains unused. Resolve by checking post-B1 usage. (If B1 promotes and the function/macros callers now call `is_bare_symbol(node, "->")`, the doc's "->" is correct — keep it.)

### B3 (exigere L1) — stale status
mod.rs ~59: `**Stone 241.5** — ... PENDING.` But 241.5 SHIPPED. Change `PENDING.` → `DONE.` (one word; matches the other entries).

### B4 (intueri L2) — variable renames in parse.rs
- `idx` (~42) → `cursor` (or `triple_start`) — wide scope, two paths; name the role.
- if a `post_rest` / boundary index exists, → `trailing_start`. (Verify against live file — orchestrator's read showed `let consumed = idx + 4` not `post_rest`; rename whatever the trailing-bound variable actually is, or skip if already clear.)

## Gates (all hold)
```
cargo test --release --lib -p wat 2>&1 | tail -1            # 890/0
cargo test --release --test function 2>&1 | tail -1         # 8/0
cargo build --release --tests --workspace                   # clean
cargo clippy --release 2>&1 | grep -cE "^warning:"          # <= 894
cargo test --release --lib -p wat argspec 2>&1 | tail -1    # argspec tests pass
```
Plus any probe that exercises argspec error paths (grep tests/ for argspec).

## STOP triggers (REJECTION)
1. Any gate regresses · 2. ANY git mutation · 3. CONFUSING compile error (not the verbose cascade) — pivot+surface · 4. unsafe/leak/clone-to-satisfy-borrow · 5. touching `From<ArgSpecError> for CheckError` as "dead" (it's LIVE — infer.rs:71) · 6. introducing `Span::unknown()` at any emitter (all have concrete spans) · 7. a spanless-by-domain rune (none needed — argspec has no spanless variant) · 8. holon-rs touched · 9. scope creep beyond argspec home + the B1 caller-edits (function/parse.rs + macros.rs arrow-inlines + the macros.rs From-consolidation) · 10. INTERSTITIAL touched · 11. 90 min elapsed

## Return paragraph (≤ 250 words)
- A: ArgSpecError → Pattern A (outer struct span+head+kind; 5 unit variants + 2 data variants); into_parts → kind.reason(); the 3 From impls direct-field; emitter cascade count
- A4: From<ArgSpecError> for MacroError added; macros.rs hand-match collapsed to ?; classify() ghost gone
- B1: is_bare_symbol promoted pub(crate); 2 inline arrow-detections replaced
- B2/B3/B4: doc + stale-status + renames
- VERIFY: no Span::unknown() at any emitter; From<CheckError> untouched (live)
- all gates; CONFIRM no commits/git-mutations/scratch-files
- any confusing-error pivots

## Predicted band
**60-90 min Mode A.** Pattern A retrofit (the spine — 7 variants, 7 emitters, 4 From impls) + the MacroError consolidation + arrow-promotion + doc/renames. Contained to argspec + 2 caller files. Borrow-checker-guided cascade; verbose-not-confusing.
