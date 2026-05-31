# BRIEF — argspec WARD R2 — annihilate live-cast findings (earn the vigilatum)

You are sonnet. The `src/argspec/` home was prematurely stamped `vigilatum`; the
stamp was RETRACTED (commit `d81941f2`) because a live 8-spell vigilia, cast
after the Pattern A retrofit, did NOT converge. This sweep annihilates every
must-fix finding so the home can earn the stamp on a clean re-cast.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside the
named edits. `git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator
commits atomically after re-casting the vigilia. If you believe a commit is
needed, STOP and say so. (#1 rejection trigger.)

Work only in `/home/watmin/work/holon/wat-rs/`. Files in scope: `src/types.rs`,
`src/argspec/error.rs`, `src/argspec/parse.rs`, and the probe
`tests/probe_arc241_stone1_argspec_canonical.rs` if its assertions need updating
to match improved messages.

---

## FIX 1 (L1 — the blocker) — double-span in MalformedTypeKeyword

`src/argspec/error.rs:62` builds `format!("type keyword is malformed: {inner}")`
where `inner: Box<TypeError>`. `TypeError`'s Display (`src/types.rs:1565`) prepends
`span_prefix(&self.span)` — so the rendered diagnostic stamps the location TWICE
(once from the outer `ArgSpecError.span`, once from `{inner}`), and this `reason()`
arm is inconsistent with its six span-free siblings.

The honest fix needs a span-free render of `TypeErrorKind`, which does not exist.
Build it (trap-door), then use it:

**1a. `src/types.rs` — add a span-free `Display for TypeErrorKind`, delegate `TypeError`'s Display to it.**
- Add `impl fmt::Display for TypeErrorKind` whose body is EXACTLY the current 13
  match arms of `TypeError`'s Display (lines ~1568–1679) **with every `prefix`
  removed** (and the `prefix` parameter dropped from each `write!`).
- Reduce `impl fmt::Display for TypeError` to:
  ```rust
  let prefix = span_prefix(&self.span);
  write!(f, "{}{}", prefix, self.kind)
  ```
- CRITICAL: every reason string must stay **byte-identical** (minus the prefix).
  This is a pure extract-and-delegate refactor. The ONLY behavior change anywhere
  is that argspec (FIX 1b) stops double-printing the span. Any `TypeError` rendered
  on its own prints exactly as before. Verify by reading both Displays side by side.
- If a `remedies`-rendering arm or any arm has post-`write!` logic, preserve it
  verbatim in the `TypeErrorKind` Display (move the whole arm body, drop only the
  prefix token).

**1a-NOTE — the CyclicSubtype delta (read carefully).** Of the 16 `TypeError`
Display arms, EXACTLY ONE — `CyclicSubtype` (types.rs:~1666) — currently renders
WITHOUT a span prefix (it's the lone anomaly; the other 15 all use `prefix`). When
you move it verbatim into `TypeErrorKind`'s Display (no prefix) and `TypeError`
delegates via `write!("{}{}", prefix, self.kind)`, `CyclicSubtype` will GAIN a
prefix. This is the ONE intentional behavior change and it is an IMPROVEMENT (a
span-bearing error showing its location). VERIFIED no test asserts CyclicSubtype's
message text (grep: only src/check.rs + src/types.rs reference it). Make this
change; flag it explicitly in your report as the single accepted delta. All other
15 arms must be byte-identical (minus prefix).

**1b. `src/argspec/error.rs:62`** — change to:
```rust
ArgSpecErrorKind::MalformedTypeKeyword { inner } =>
    format!("type keyword is malformed: {}", inner.kind),
```
(`inner.kind` now Displays span-free; outer `ArgSpecError.span` carries the single
location.)

---

## FIX 2 (L2) — stale doc counts / caller lists in argspec

- `src/argspec/parse.rs:59` — says "the three `From<>` impls in `error.rs`"; there
  are FOUR (`RuntimeError`, `CheckError`, `TypeError`, `MacroError`). Change
  "three" → "four".
- `src/argspec/error.rs:76` (the comment beginning "241.2/241.3 callers convert
  at their boundary") — the enumeration is stale (defclause/defmacro/types
  consumers now also route through). Replace the stone-specific list with a
  maintenance-free sentence, e.g.:
  `// Callers convert at their site boundary; the parser itself emits only ArgSpecError.`

---

## FIX 3 (L2) — extract the duplicated triple-extraction block

`src/argspec/parse.rs` has the same guard + `try_into().expect()` block twice
(the rest-binder branch ~91–100 and the fixed-params loop ~116–126). Extract one
private helper and call it from both sites:
```rust
fn extract_triple<'a>(
    args_vec: &'a [WatAST],
    start: usize,
    span: Span,          // the span to attribute IncompleteTriple to (see FIX 5)
    head: &str,
) -> Result<&'a [WatAST; 3], ArgSpecError> {
    if args_vec.len().saturating_sub(start) < 3 {
        return Err(ArgSpecError { span, head: head.to_string(),
            kind: ArgSpecErrorKind::IncompleteTriple });
    }
    Ok(args_vec[start..start + 3].try_into().expect("len gated by the `< 3` check above"))
}
```
Both call sites reduce to `let triple = extract_triple(args_vec, start, <span>, head)?;`
The `.expect()` invariant message now lives in one place.

---

## FIX 4 (L2) — drop the gratuitous generic on parse_keyword_type

`parse_keyword_type` is generic over a closure `F: FnOnce(Span, String) -> ArgSpecError`
but has exactly ONE call site, and the closure always produces
`ArgSpecErrorKind::TypeNotKeyword`. Remove the `F` parameter; hardcode the
`TypeNotKeyword` kind in the non-keyword arm. The single caller (`parse_triple`)
drops the closure argument.

---

## FIX 5 (L2 — cernere messages + sequi span-precision, folded together)

Make these `reason()` strings name the real failure, AND make the errors point at
the offending element rather than the whole-vector `form_span` (the span-precision
makes the messages resolvable):

- `IncompleteTriple` — attribute to the offending element's span where one exists:
  - fixed-params path: `args_vec[cursor].span().clone()` (loop guard proves valid).
  - rest-binder path: `if rest_start < args_vec.len() { args_vec[rest_start].span().clone() } else { form_span.clone() }`.
  - Plumb the chosen span into `extract_triple` (FIX 3's `span` param).
- `TrailingItems` (parse.rs ~105) — attribute to `args_vec[trailing_start].span().clone()`
  (branch condition proves valid). Reword `reason()` (error.rs:64):
  `format!("{count} trailing item(s) after the rest-binder triple `& name <- :T`; nothing may follow it")`.
- `MissingArrow` (error.rs:58) — drop the internal "slot 1" jargon:
  `"triple must be `name <- :T`; expected `<-` as the second element"`.

Keep all other reason strings as-is. If
`tests/probe_arc241_stone1_argspec_canonical.rs` asserts on any changed message
text, UPDATE the probe assertion to the new wording (keep its structural intent —
it must still prove the same failure fires; only the message text changes).

---

## FIX 6 (L3 → comment only) — document the MacroError head redundancy

`src/argspec/error.rs` `From<ArgSpecError> for MacroError` drops `e.head`. This is
CORRECT (the only caller passes head `":wat::core::defmacro"`, identical to what
`MacroError::MalformedDefmacro`'s Display hardcodes). Add ONE comment on that impl
so the asymmetry is not silent:
```rust
// e.head ("defmacro") is redundant with MalformedDefmacro's own Display text; not threaded.
```
No structural change.

---

## DO NOT TOUCH (L3 — leave per let-need-reveal)

- temperare allocations (`head.to_string()` per arm, `reason()→String` on statics,
  `ident.name.clone()`, missing `Vec::with_capacity`) — all parse-time, no proven
  cost. LEAVE.
- `ParseOptions` rename, `is_bare_symbol` placement — LEAVE.

---

## VERIFY before returning

Run and report exact numbers:
- `cargo test --release --lib -p wat` (expect 890+ / 0)
- `cargo test --release --test probe_arc241_stone1_argspec_canonical` (expect all pass)
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat` (must compile — the types.rs Display refactor)

Report: every file touched, the before/after of the types.rs Display split (paste
the new `TypeError` Display + the new `TypeErrorKind` Display so the orchestrator
can confirm reason strings are byte-identical), and the four gate numbers. Your
final message IS the report — raw, no human-facing summary fluff.
