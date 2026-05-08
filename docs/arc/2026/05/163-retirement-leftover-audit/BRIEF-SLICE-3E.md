# Arc 163 Slice 3e BRIEF — substrate-internal container heads to FQDN

**Drafted 2026-05-07.** User direction: *"wat internals are fully
qualified - no exceptions... if there's a short form - its illegal
... if the internal code is mapping to a rust primitive then we use
the rust form... wat /must be/ fully qualified."*

This slice rewrites substrate-internal `Parametric.head` storage from
short form (`"Vec"`, `"Option"`, `"Result"`, `"HashMap"`, `"HashSet"`)
to FQDN (`"wat::core::Vector"`, `"wat::core::Option"`, etc.). Plus
deletes the `parse_type_inner` canonicalize arm that actively
DOWNGRADES source FQDN to short form.

## Context

Arc 163 has cleared 12 of 14 BareLegacy surfaces (slice 3a/3b/3d).
Pre-flight audit for slice 3e originally targeted `:Vec<T>` walker
firmness — that's verified hard-by-construction (no edit needed).
Audit surfaced a deeper issue: substrate-internal storage uses the
LEGACY short forms despite arc 109 slice 1c retiring the user-source
short forms. Substrate-as-teacher inconsistency: walker rejects user
`:Vec<T>` while substrate stores `head: "Vec"`. User flagged 2026-05-07.

The Vec rename arc 109 slice 1f handled the user-facing surface
(`:wat::core::Vector` minted) but kept `head: "Vec"` internally as a
"transitional impl detail" — that detail has now persisted and is
the discipline gap to close.

## Working directory

`/home/watmin/work/holon/wat-rs` on `main` branch at `334f61a`.

## Workspace state pre-spawn

- HEAD: `334f61a` (arc 163 slice 3d shipped)
- Working tree: SURVEY.md + DESIGN.md staged with slice 3e plan
- Workspace: 2041 passed / 0 failed
- Audit grep `head: "Vec"|head: "Option"|head: "Result"|head: "HashMap"|head: "HashSet"` returns ~118 substrate sites (sources)
- Audit grep `head == "Vec"|head == "Option"|...` returns ~9 (reads)
- Match arms `"Vec" =>|"Option" =>|...` returns 7 in src/runtime.rs + src/check.rs
- Canonicalize arm at `src/types.rs:60-72` actively DOWNGRADES FQDN → short form

## Verify Bash availability FIRST

Per memory `feedback_verify_sonnet_tool_claims.md`: if you hesitate
about Bash availability, run `which cargo` once. Expect `cargo 1.93.0`
(or similar). Do NOT claim Bash denied.

## The rule

**Every substrate-internal `Parametric.head` string for these 5
container types becomes the FQDN form (no leading colon — heads
follow no-colon convention).**

| Old | New |
|---|---|
| `head: "Vec"` | `head: "wat::core::Vector"` |
| `head: "Option"` | `head: "wat::core::Option"` |
| `head: "Result"` | `head: "wat::core::Result"` |
| `head: "HashMap"` | `head: "wat::core::HashMap"` |
| `head: "HashSet"` | `head: "wat::core::HashSet"` |

This rule is exhaustive — no exceptions, no "but this site is special."
The substrate's convention shifts wholesale.

## Edit plan

### Phase 1 — Substrate canonicalize arm reshape (`src/types.rs`)

The canonicalize step at `parse_type_inner` (around lines 60-72)
currently REWRITES source `wat::core::Option` → `"Option"`,
`wat::core::Vector` → `"Vec"`. Post-slice-3e, the FQDN form IS the
canonical storage — the arm becomes IDENTITY.

**Action:** Delete the canonicalize-true-path container-head match
arms (`"wat::core::Option" => "Option"`, ..., `"wat::core::Vector" =>
"Vec"`). The `else { raw_head }` flows through unchanged for FQDN
inputs. Source `wat::core::Vector` flows to `head: "wat::core::Vector"`
(was: `"Vec"`).

The PRIMITIVE-path canonicalize arms (`":wat::core::i64" =>
":i64"` etc., lines 103-109) are slice 3f scope. KEEP those
unchanged for now.

### Phase 2 — Sweep `head: "X".into()` writes (~118 sites)

Per the audit grep, ~118 substrate sites write the legacy short form.
Sweep mechanically, file by file (use `replace_all: true` per
file when the legacy literal is unambiguous):

- `src/types.rs` (typealias declarations + tests)
- `src/runtime.rs` (constructor sites + match arms)
- `src/check.rs` (type inference + walker scaffolding)
- `src/freeze.rs`, `src/lower.rs`, `src/macros.rs`, etc. as audit surfaces

Apply per-pair across all 5 short→FQDN substitutions:

```
"Vec".into()      → "wat::core::Vector".into()
"Option".into()   → "wat::core::Option".into()
"Result".into()   → "wat::core::Result".into()
"HashMap".into()  → "wat::core::HashMap".into()
"HashSet".into()  → "wat::core::HashSet".into()
```

**Sweep discipline:** be precise. The literal `"Option".into()`
(with quotes + `.into()`) is unambiguous; `Option` as a Rust
type/keyword (no quotes) is NOT in scope. Use anchored patterns
(quote-literal ON BOTH SIDES + `.into()` suffix or string-literal
context) to avoid touching Rust language `Option`/`Result` etc.

### Phase 3 — Sweep reads `head == "X"` (~9 sites)

Mechanical: `head == "Vec"` → `head == "wat::core::Vector"` etc.

### Phase 4 — Sweep match arms (~7 sites)

Update match arm keys in:
- `src/runtime.rs:3661-3665` (value_tag matching) — left side only
- `src/check.rs:5500` (`head == "Option"`)
- `src/check.rs:7631` (`head == "Vec"`)

The RIGHT side of the runtime.rs:3661-3665 match (the value_tag
strings like `value_tag == "Vec"`) is OUT OF SCOPE — that's a
separate Value::type_name() system, separate slice if needed.
**Only touch the match arm KEYS (left side, head string).**

### Phase 5 — Update typealias RHS doc comments

In `src/types.rs` around lines 439-442, 501-512, doc comments
describe the typealiases as `:wat::core::Option<T> = :Option<T>`
etc. — that's wat-syntax-shaped descriptive text. After slice 3e,
the RHS in those doc comments doesn't accurately reflect the new
internal storage (which is `head: "wat::core::Option"` — Rust
internal, not a wat type expression).

**Action:** rewrite those doc comments to describe the substrate
mechanism honestly. Example shift:

```
// (old)  typealias :wat::core::Option<T>     = :Option<T>
//        ...the substrate's existing special-case dispatch (which reads
//        against bare head names) keeps working...

// (new)  Internal canonical storage: Parametric { head:
//        "wat::core::Option", args: [...] }. Walker fires
//        BareLegacyContainerHead on source `:Option<T>` (bare head);
//        FQDN form `:wat::core::Option<T>` flows through to canonical
//        storage unchanged.
```

Keep the comments factual and reflect the post-slice reality. Don't
preserve "transitional" phrasing — the transition closed.

## Phase verify (after each phase)

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release 2>&1 | tail -5
```

Build MUST stay clean throughout. If it doesn't: STOP — you've
broken substrate-internal recognition somewhere. Re-read the
failing site, fix in-place, continue.

## Final verify

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pass:", passed, "Fail:", failed}'
```

Expect: 2041 / 0.

If any test fails: investigate, do not paper over. The most
likely failure mode is a test that hardcoded a legacy head string
(`assert_eq!(head, "Vec")` style); update the test's expected
value to FQDN form.

Audit grep post-fix:

```bash
# Bucket A: live legacy heads should be 0 (Bucket D variant scaffolding survives if any)
grep -rEn '"Vec"|"Option"|"Result"|"HashMap"|"HashSet"' src/ --include="*.rs" | grep -E '\.into\(\)|head ==|=> ' | head -20
```

Expect: zero live legacy short-form usage in Bucket A pattern. Any
hits should be either:
- (D) orphaned scaffolding (variants + Display preserved) — KEEP
- (C) historical retirement-context comments — KEEP
- Rust language usage (e.g., `Vec::new()`, `Option::Some`) — out of
  scope, KEEP

## Constraints

- DO NOT commit. Working tree dirty for orchestrator review.
- "STOP at unexpected red" — don't paper over breakage.
- Test count must stay 2041 (or higher).
- Slice 3e is HEAD STRINGS ONLY. Primitive paths (`":i64"` etc.) are
  slice 3f scope — DO NOT TOUCH.
- value_tag strings on the RIGHT of `value_tag == "Vec"` are out of
  scope. Only update the match arm KEYS.
- Rust standard-library `Vec`, `Option`, `Result`, `HashMap`, `HashSet`
  identifiers (no quotes, used as Rust types) are out of scope.
- Time-box: 60 min wall-clock.

## Reporting (~250 words)

1. Phase 1: canonicalize arm change — confirm deletion.
2. Phase 2-4: per-file sweep counts (file: N writes + M reads + K match arms updated).
3. Phase 5: doc comment updates — names of comment blocks revised.
4. Test pass: pre vs post (must stay 2041 or higher).
5. Path: Mode A / B / C.
6. Honest deltas:
   - Did Phase 1 break the build? Why?
   - Any sites you classified as Bucket C/D and KEPT (variant Display fixtures, retirement-context comments).
   - Any sites you weren't sure how to classify — list them for orchestrator decision.
   - Did `cargo test` reveal substrate-internal sites that still need legacy form (Phase enumeration missed)?
   - Were any test assertions on hardcoded legacy head strings — list them.

DO NOT commit. Orchestrator commits + scores after.
