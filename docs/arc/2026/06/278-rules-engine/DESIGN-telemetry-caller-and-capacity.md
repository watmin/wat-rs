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

## 2. Arith → the F5 macro-expand allow-list ("label it")

**Finding (proven by a probe that RED-failed).** `:wat::core::+` is REFUSED at macro-expand time:
*"not on the pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only pure-total
heads are permitted."* The gate is `is_pure_total(head)` in `src/macros/eval.rs:169` — the pure-total
subset of `dispatch_keyword_head`. `length` (returns i64) is allowed; `+`/`-`/`*` are not — an
oversight, not a principle. Arith IS pure-total; it just needs labeling.

**The label (strike):** add `:wat::core::+` / `-` / `*` and the comparisons (`<`/`>`/`<=`/`>=`/`=`/
`not=`) to `is_pure_total`. **Keep `/`, `mod`, `rem`, `quot` OFF** — they are pure but *partial*
(div-by-zero raises), and the gate's contract is *total*. Capacity math is only `+`/`-`, so the total
subset suffices. Strike = add heads + a probe proving expansion-time arith (the removed
`log-msg-capacity-probe.wat` returns) + `--release` weigh.

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
splice (`namespace <- String`, `uuid <- Uuid`, `tags <- Tags`, `time-ns <- i64`) + `caller`/`level`/
`message` (wat/telemetry.wat:83-99). `namespace`/`tags`/`caller` are VARIABLE-length → a truly-exact,
zero-waste per-record limit is inherently RUNTIME. What IS compile-time-derivable is the **framing
floor**: the record tag + the field-name keys (all literals via `field-names-of`) + fixed-width fields.

**Design (two layers, one reflected field set):**
- **Compile-time (a `defmacro`, needs §2 arith):** reflect `field-names-of :…::Log` at expansion, sum
  the framing bytes (tag + per-field key cost), emit `(def :…::LOG-MSG-CEILING (- BUDGET framing))` as a
  baked literal — the max message bytes if all other required values were empty. The type IS the schema
  (R26 derive) — change the required fields, the ceiling moves with them, checked.
- **Runtime (the exact, zero-waste-per-caller remainder):** `budget − serialized-bytes(the filled-in
  required params)`, generated from the same reflected field set.

**Caveat to resolve at build:** the exact per-field wire cost needs grounding the EDN encoder (or an
expansion-time `edn::write` of a zeroed template + a byte-length prim — only `string::length`
(char-length) was found; UTF-8 byte-length TBD). The illustrative probe used +3/field, +2/tag — the
MECHANISM is proven; the exact costs are the build's job. Depends on §1 (final schema) + §2 (arith).

---

## Status
- §1 caller: designed + names ratified; RED probe + shadowdancer strike NEXT (per the ordering).
- §2 arith: finding proven (RED); one-spot label strike, queued after §1.
- §3 capacity: designed; needs §1 (schema) + §2 (arith); the payoff.
- Stone 16.1 (the ruling-A / RequestTooLarge lock) is DONE + committed (9ca2e88d, 25ca431b).
