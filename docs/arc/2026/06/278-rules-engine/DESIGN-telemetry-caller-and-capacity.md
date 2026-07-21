# DESIGN — telemetry `caller` as call-site, arith at expansion, and derived log capacity

**Arc 278, post-Stone-16.1.** Three linked sub-threads on the telemetry `Log`, discovered together
while exploring "can we evaluate a code form during macro expansion to derive a byte budget from a
log's required params?" All grounded against the disk this session; names cast + ratified by intueri.

## Ordering (tractability: each step makes the next more tractable)

**caller stone → arith label → capacity-derive.**

The capacity-derive REFLECTS over the `Log` field set (framing overhead = the fixed field-name keys +
tag). The caller stone CHANGES that field set (`caller <- keyword` → `emitted-from <- Frame`; a `Frame`
serializes to a different fixed size than a keyword). So: settle the schema first (caller), then derive
capacity once against the final shape — no rework. Arith is capacity's numeric prerequisite. Hence the
order above.

---

## 1. `caller` → a captured call-site `Frame` (ratified #1)

**Problem.** `Log.caller <- :wat::core::keyword` (wat/telemetry.wat:94) is a FREE-FORM, forgeable tag
the producer hand-types (`:probe`). We want the EXACT source site the log was emitted from — checked,
un-forgeable — like Ruby's `file:line:in 'label'` / Rust's `#[track_caller]` / `Location::caller()`.

**Ratified names (intueri cast, weighed + concurred, collision-checked clean):**
- **`:wat::kernel::call-site`** — the native nullary verb; returns the caller's `Frame`. (`caller` was
  rejected: names the agent "who" but returns a location "where", and Ruby's `caller` is a whole array
  — an arity lie. `here`/`current-frame` mumble — ego-framed / plumbing-framed.)
- **`emitted-from`** — the `Log` field (replaces `caller`). Speaks the telemetry domain (logs are
  *emitted*); refusing to keep `caller` refuses to hide the free-form→checked upgrade.
- **`:wat::kernel::Frame`** — reused directly as the field type. No added concept → a new noun would be
  ceremony. And it is `Nature::Record` (arc 293.W.2b — pure EDN, `value_from_frame_info`
  runtime.rs:23088), so `emitted-from <- Frame` serializes + crosses the wire cleanly. Reuse is
  required-shaped, not just clean.

**Mechanism — grounded, offset 0.** `:wat::kernel::call-site` is NATIVE and mirrors `assertion-failed!`
(src/assertion.rs:134): `snapshot_call_stack()` → `.first()` → `value_from_frame_info` →
`:wat::kernel::Frame {file, line, symbol}`. The precedent's own comment: *"Top frame = innermost user
call (where the author wrote the assert)."* A native verb pushes NO wat frame of its own (only wat
fn-calls push, via `FrameGuard`, src/value/frame.rs) — so from inside the native verb the top frame is
already the true caller. **Offset 0**, deterministic, proven by the shipped `assertion-failed!` behavior
(no fresh mechanism needed). The stack read is a cheap in-memory `thread_local` `Vec<FrameInfo>`
snapshot — NOT `std::backtrace` symbol resolution.

- `snapshot_call_stack` / `FrameInfo` / `FrameGuard`: `src/value/frame.rs` (Stone 251.2a).
- `FrameInfo` → `:wat::kernel::Frame` Record: `src/runtime.rs:23088` (`value_from_frame_info`).
- `Frame` fields: `{file, line, symbol}`, each `Option` (symbol = the enclosing-defn label).

**NOT on the macro-expand allow-list** — `call-site` reflects the runtime stack (a runtime reflection),
so it is a runtime verb, never an expansion-time pure-total combinator (see §2).

**#1 vs #3 (four-questions verdict → #1).** #1 = runtime `Frame` off the arc-016 call-stack.
#3 = static `Frame` baked at expansion via a NEW enclosing-defn seam. #3 fails **Simple** (needs a new
expander seam with edges: nested/macro-generated defns, closures) and is weak on **Honest** (captures
the LEXICAL parent, not the dynamic caller — a helper's name, not the origin). #1 wins Obvious + Simple
+ Honest; #1's only cost (runtime capture) is negligible + grounded-cheap, so its Good-UX holds too.
#3 stays closed unless the capture cost ever bites (it doesn't — cheap in-memory read).

**Strike (a shadowdancer's build; orchestrator draws the RED probe + briefs + weighs — R20):**
1. `:wat::kernel::call-site` native verb (dispatch entry + return-type registration = `:wat::kernel::Frame`).
2. `Log`: `caller <- :wat::core::keyword` → `emitted-from <- :wat::kernel::Frame` (wat/telemetry.wat:94).
3. Migrate the `caller` writers (fixtures pass `:caller :probe`) → the emission path fills
   `emitted-from` via `(:wat::kernel::call-site)`; codemod if multi-site (`wat-scripts/fixes/`).
4. **RED gate:** a wat test calls `(:wat::kernel::call-site)` from a fn and asserts the returned
   `Frame`'s `file`/`line`/`symbol` match that fn's site → RED now (verb absent) → GREEN after.

**Open sub-decision:** does the field hold ONE `Frame` (the immediate emitter — lean, zero-waste; the
recommendation) or `Vector<Frame>` (a full backtrace)? Default: one `Frame`; a full-backtrace verb
(`:wat::kernel::...` → `Vector<Frame>`) is a separate future add.

---

## 2. Arith at macro-expand — RESOLVED (no substrate change): use fully-qualified TYPED arith

**Corrected finding (grounded 2026-07-21 — the earlier "add polymorphic heads" premise was STALE and
wrong).** The gate is `is_pure_total(head)` (`src/macros/eval.rs:351`). It ALREADY blesses the
fully-qualified **TYPED** arith — `:wat::core::i64::+/-/*//`, `i64::mod/rem/quot`, `i64::>/</>=/<=`,
`f64::…` — plus `:wat::core::=`/`not=`/`and`/`or`/`not`. And those **fold at macro-expand today** (PROVEN:
a `defmacro` computing `(:wat::core::i64::+ …)` through `(:wat::core::i64::/ …)` + the `i64::` comparisons
`--check`s CLEAN and splices the computed literal `56`).

**Only the POLYMORPHIC un-suffixed heads (`:wat::core::+`, `-`, `*`, `/`, `<`, `>`, `<=`, `>=`) are
refused — and that refusal is CORRECT.** Polymorphic arith is **NOT viable at macro-expand** (builder,
2026-07-21): it needs runtime type dispatch, not a macro-time capability — `(wat.core/+ …)` can't be used
at expand, but `(wat.core.i64/+ …)` can. Blessing the polymorphic heads would label something unusable —
leave them refused. (The RED I proved was the F5 *refusal*; I never confirmed a polymorphic *fold*, and
typed already covered the need — the wrong-premise strike is retired.)

**Resolution: NO `is_pure_total` change. arith is DONE.** Capacity (§3) and any expand-time arith use the
fully-qualified TYPED forms (`:wat::core::i64::+`/`i64::-`/… — capacity math is i64 `+`/`-`), already
blessed and folding. `/`,`mod`,`rem`,`quot` are moot here (typed-only, already on; div-by-zero = a
deterministic located abort, the gate's real "no-panic" contract).

**Macro-expand engine facts (grounded):**
- A `defmacro` body is pure code run AT EXPANSION; `~value` splices the computed result as a baked
  literal (precedent: `:wat::core::keyword/of`, wat/core.wat:1245 — `` `~(keyword/from-string full) ``).
- Field reflection runs at expansion: `:wat::runtime::field-names-of` / `field-types-of` (wat/bracket.wat:272)
  — a shipped, green macro reflecting a type's fields to synthesize code.
- `is_pure_total` default-denies effectful heads (`eval-ast!`, `macroexpand-1`) — the F5 wall stays.

---

## 3. `LOG-MSG-CAPACITY` — derive the byte budget from the required params

**Goal.** Declare the exact message-byte limit a caller has, zero waste — server read ceiling ~10 MiB
per req; a single log record must fit just under it; the dynamic `message` gets the rest after the
required params.

**Honest boundary (grounded — the real `Log` has variable-length required params).** `Log` = `Scope`
splice (`namespace <- String`, `uuid <- Uuid`, `tags <- Tags`, `time-ns <- i64`) + `emitted-from`/`level`/
`message` (wat/telemetry.wat:83-99; `caller <- keyword` → `emitted-from <- :wat::kernel::Frame` as of
caller.2). `namespace`/`tags`/`emitted-from`/`message` are VARIABLE-length → a truly-exact, zero-waste
per-record limit is inherently RUNTIME. What IS compile-time-derivable is the **framing floor**: the record
tag + the field-name keys (all literals via `field-names-of`) + the fixed-width fields.

**The fixed-vs-variable rule (ratified 2026-07-21, builder).** A field contributes to the fixed framing
floor ONLY if its type is an **explicitly-defined-known-size** wat type — *"we can only assert what we
explicitly define to have a known size; any field that is not a symbol of a wat-provided [known-size] field
must be assumed variable width."* FIXED: the sized primitives (`i64`, `f64`, `Uuid` = 16B, `bool`, …) and an
**enum** — sized to its **LONGEST variant** so every variant fits (`level <- Level` is fixed at its longest).
Everything else is **VARIABLE** and contributes ZERO to the fixed floor (runtime-only bytes): any `String`,
`Tags`/map, record, user type, or `:wat::kernel::Frame` (it carries a path `String`, so `emitted-from` is
variable — confirmed by caller.2). So for the current `Log`: FIXED = the tag + all field-name keys + `uuid`
(16B) + `time-ns` (i64) + `level` (Level @ longest variant); VARIABLE = `namespace` + `tags` + `emitted-from`
+ `message`. The framing floor sums only the FIXED set; the per-caller remainder is the runtime layer below.

**Design — a RUNTIME adaptive derive (the type IS the schema, R26 derive-is-the-wall).** The floor is
computed AT RUNTIME (once, at the sink's `:init` / first use), reflecting the LIVE `Log` type registry — so
a field added/removed/retyped tomorrow re-derives the capacity (builder: *"i do not know what we'll do
tomorrow… all i know is that we need tooling who'll adapt"*; `caller`→`emitted-from` was one such swap).
Runtime is where capacity is ENFORCED (the server rejects over-budget requests), so the derive lives where
it's used — no baked-constant staleness, no macro-expand gap.

**Why runtime, not compile-time (grounded 2026-07-21 — the compile premise was DISPROVEN by a probe).**
`field-types-of` at MACRO-EXPAND errors *"unknown type"* even for baked `:wat::telemetry::Log` — the
macro-expand `sym` only holds `bracket.wat`'s locally-generated `::Kwargs`, not record types. But at RUNTIME
`field-types-of :wat::telemetry::Log` RESOLVES (proven): it returns the 7 field type-nodes, and `ast-name`
gives a string per field — `"wat.type/String"`, `"wat.type/Uuid"`, `"wat.telemetry/Tags"`, `"wat.type/i64"`,
`"wat.kernel/Frame"`, `"wat.telemetry/Level"`, `"wat.type/String"`. So reflect→classify→sum runs at RUNTIME.

- **The adaptive floor (runtime, at `:init`).** `(framing-floor-of :Log)` = reflect `field-names-of` +
  `field-types-of` → per field, `ast-name` → classify against the explicitly-known-size set → sum the FIXED
  (via `i64::+`) + the ASCII field-name-key byte costs (`string::length`) + the tag. Static per schema;
  computed once, cached. `LOG-MSG-CAPACITY = BUDGET − floor` — the advisory "your message can be ~N bytes."
- **The exact per-caller remainder (runtime).** `budget − serialized-bytes(the filled-in required params)`,
  from the same reflected set (`edn::write` + real byte sizing are available at runtime). **The floor is a
  conservative hint; the remainder is the exact gate.**

**Ratified build order (four-questions, 2026-07-21 — lead with the mechanism, not the plumbing).**
1. **Stone 1 — the runtime adaptive derive (the deliverable; buildable now, NO substrate gap).**
   `(framing-floor-of <record-type>)` — reflect → classify (match each field's `ast-name` `"wat.type/…"`
   string) → sum the *unambiguously* fixed (`i64`/`f64`/`Uuid`/`bool`) + the ASCII field-name keys + the
   tag → the adaptive floor; then `LOG-MSG-CAPACITY = BUDGET − (framing-floor-of :Log)`. Enums/strings/
   records/`Frame` sit in the VARIABLE part for now (the remainder sizes them exactly; an under-counting
   floor is a safe conservative hint *because* the remainder is the real gate). **RED gate: a record with an
   extra fixed field yields a LARGER floor** — adaptivity proven on a *general* record (the test needn't
   mutate stdlib `Log`). This is "tooling that adapts."
2. **Refinements (sharpen the floor, NOT prerequisites), each a small substrate prim:**
   - **`variants-of`** — RUNTIME enum-variant reflection → pull enums from VARIABLE into the fixed floor,
     sized to their longest variant BY REFLECTION. **Ratified option (b): reflection, NEVER a hand table**
     — a table drifts on a new variant (fails Honest; no check catches it), reflection cannot.
     (`enum_def.variants` exists internally in `runtime.rs`; expose a `:wat::runtime::variants-of`.)
   - **A UTF-8 byte-length prim** — only `string::length` (char-length) today → exact non-ASCII keys.

**Grounded substrate facts (2026-07-21):** at RUNTIME `field-names-of`/`field-types-of` resolve record types
(`:wat::telemetry::Log` → 7 type-nodes; `ast-name` → `"wat.type/i64"`-style strings; typed `i64::` arith
works). At MACRO-EXPAND they do NOT (the macro-expand `sym` lacks non-local record types — the
compile-time-derive path is DEAD). GAPS (refinement-only): no `variants-of` runtime reflection; no UTF-8
byte-length prim.

---

## Status
- §1 caller: **DONE** — caller.1 (the `:wat::kernel::call-site` verb, `60fbef21`) + caller.2 (the
  `emitted-from <- :wat::kernel::Frame` field flip + structural codemod + gate, `31b3d1ac`), both weighed
  green by own re-run (4194→4195/0). `:caller'` arena service verified untouched.
- §2 arith: **DONE (no substrate change)** — the fully-qualified TYPED arith (`:wat::core::i64::+`/`-`/…)
  is already blessed on `is_pure_total` and folds at macro-expand (proven, `--check` clean → literal 56);
  polymorphic arith is correctly refused (not macro-time-viable per the builder — needs runtime dispatch).
  Capacity uses the typed forms. The earlier "add 8 polymorphic heads" strike was retired (wrong premise).
- §3 capacity: designed + RATIFIED (fixed-vs-variable rule; four-questions build order; option (b)
  enum-by-reflection; **RUNTIME reframe** — compile-time reflection of `Log` was DISPROVEN by a probe, the
  derive runs at runtime, proven + fully adaptive). Stone 1 = the runtime `(framing-floor-of :Log)` (reflect
  → classify each field's `ast-name` → sum the unambiguously-fixed + ASCII keys → `LOG-MSG-CAPACITY = BUDGET
  − floor`; RED gate = a record with an extra fixed field yields a larger floor) — buildable now, no
  substrate gap — **BUILDING**. Refinements (follow-ons): `variants-of` (enum-in-floor by reflection) + a
  UTF-8 byte-length prim.
  **Also owed (surfaced by caller.2):** a `log` MACRO that captures `(:wat::kernel::call-site)` AT the
  user's log-call boundary — caller.2 fills *constructions*, which capture the enclosing-fn's caller (offset
  0), so a Log built in a Rust-invoked fn gets the Rust site (absolute path); the precise per-log-line
  capture is the macro's job (capture-at-expansion, like `assertion-failed!` fires at the author's line).
- Stone 16.1 (the ruling-A / RequestTooLarge lock) is DONE + committed (9ca2e88d, 25ca431b).
