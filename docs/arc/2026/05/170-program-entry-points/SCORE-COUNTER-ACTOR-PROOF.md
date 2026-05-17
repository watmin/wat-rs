# SCORE — Counter actor pattern proofs (thread + process tiers)

**Phase:** Pre-D3 verification artifact. Proves the Counter actor pattern
inscribed in INTERSTITIAL-REALIZATIONS.md § 2026-05-16 (Kay-OOP + control-channels
entries) at both thread and process tiers.

**Commit baseline:** `c581c73` (workspace: 2328 passed / 4 failed)

**Post-proof result:** 2334 passed / 3 failed (lifeline flake absent this run; ±1 per BRIEF)

---

## SCORE rows

| Row | What | Evidence | Result |
|-----|------|----------|--------|
| A | Thread-tier deftest passes end-to-end | `cargo test -p wat --test test deftest_counter_actor_thread_proof` → `ok` | YES |
| B | Process-tier deftest passes end-to-end | `cargo test -p wat --test test deftest_counter_actor_process_proof` → `ok` | YES |
| C | Both tests use identical body shape (same operations + same assertions; only transport/verb differs) | `counter-actor-proof-thread.wat` and `counter-actor-proof-process.wat` body sections are line-for-line identical except `counter/` → `counter-proc/` prefix, `Thread/println` → `Process/println`, `Thread/readln` → `Process/readln`, `Thread/drain-and-join` → `Process/drain-and-join` | YES |
| D | Workspace failure count ≤ baseline (≤ 4) | 3 failures: `deftest_wat_tests_tmp_totally_bogus` + `startup_error_bubbles_up_as_exit_3` + `t6_spawn_process_factory_with_capture_round_trips` — all pre-existing per BRIEF § baseline | YES |
| E | All inscribed pattern claims verified | enum shapes ✓, four handler shapes (read/mutate-computed/mutate-literal/terminal) ✓, Shutdown/Final ✓, state recovery ✓, lockstep ✓, drain-and-join ✓ | YES |

**All 5 rows: YES.**

---

## Honest deltas from inscribed pattern

### Delta 1 — Enum unit variant syntax

**Inscribed:** `Get`, `Reset`, `Shutdown` (bare symbols) as unit variants.

**Actual substrate:** unit variants must be written as `(VariantName)` (list form with
no payload fields) OR as `:VariantName` keywords. This artifact uses the list form
`(Get)`, `(Reset)`, `(Shutdown)` throughout, which the parser registers as
`EnumVariant::Tagged { name, fields: [] }` (zero-field tagged).

**Impact discovered:** The serializer (`value_to_edn_with`) checks `fields.is_empty()`
at runtime and emits `nil` body. The coercer (`coerce_enum_path`) checked the TypeDef —
found `Tagged` — and expected a Vector body. This produced a round-trip failure:
`edn coerce mismatch: expected Shutdown (tagged), got Tagged-body Nil`.

**Fix applied:** `src/edn_shim.rs` `coerce_enum_path` — zero-field Tagged variants
now accept `Nil` as equivalent to an empty vector. This makes `(VariantName)` a valid
unit-equivalent syntax for EDN round-trip.

**File:** `src/edn_shim.rs` around line 869 — the Tagged arm in `coerce_enum_path`.

### Delta 2 — Enum payload variant syntax

**Inscribed:** `(Increment :wat::core::i64)` (positional, no field name).

**Actual substrate:** payload variants use named fields: `(Increment (n :wat::core::i64))`.

### Delta 3 — Enum variant constructor separator

**Inscribed:** `:counter::Request/Get` (slash separator).

**Actual substrate:** `::` separator — `:counter::Request::Get`.

`register_enum_methods` synthesizes constructors as `format!("{}::{}", enum_path, variant_name)`.

### Delta 4 — ThreadPeer orientation

**Inscribed:** `counter/dispatch` takes raw `server-rx!` + `server-tx!` (Receiver + Sender directly).

**Actual substrate:** Thread/readln and Thread/println operate on a ThreadPeer struct.
The dispatch fn takes `ThreadPeer<counter::Request, counter::Response>` (server-side peer:
reads requests, sends responses). The constructor wraps `server-rx!` + `server-tx!` via
`ThreadPeer/new(server-rx!, server-tx!)` inside the spawn-thread fn body.

### Delta 5 — spawn-process cannot capture parent types

**Inscribed:** implied that subprocess could reference parent-side types.

**Actual substrate:** subprocess declares its own independent copy of the counter enums.
EDN round-trip works because:
- parent serializes `counter::Request::Increment` → `#counter.Request/Increment [5]`
- subprocess deserializes `#counter.Request/Increment [5]` against its own `counter::Request` TypeDef

Same EDN tag format (namespace derived via `::` → `.` replacement) means types are
interoperable across the process boundary without any shared type registry.

### Delta 6 — ProcessPeer construction argument order

**Inscribed:** `ProcessPeer/new(Process/stdout proc, Process/stdin proc)` (direct).

**Actual substrate (verbose-is-honest form):**
```
rx    = Receiver/from-pipe(Process/stdout proc)   ← reads subprocess stdout
tx    = Sender/from-pipe(Process/stdin proc)       ← writes subprocess stdin
peer! = ProcessPeer/new(rx, tx)
```
No constructor verb minted. Three explicit steps expose the composition.

### Delta 7 — Subprocess entry point name

**Inscribed:** uses `:user::main-process` (speculated).

**Actual substrate:** always `:user::main` — per user 2026-05-16:
"processes must always define `:user::main` ... there is no `:user::main-process`".

### Delta 8 — Client-side ProcessPeer type orientation

**Inscribed:** not explicitly stated in the INTERSTITIAL for process tier.

**Actual substrate (per Stone C2):**
`ProcessPeer<counter::Response, counter::Request>` — client reads responses (from
subprocess stdout), writes requests (to subprocess stdin). Same asymmetry as thread
tier's `ThreadPeer<counter::Response, counter::Request>`.

---

## Substrate fix shipped

**File:** `/home/watmin/work/holon/wat-rs/src/edn_shim.rs`
**Function:** `coerce_enum_path` (line ~869)
**Change:** Zero-field `EnumVariant::Tagged` variants now accept `Nil` body in the
coercer, matching what the serializer produces for `EnumValue { fields: [] }`.

This is a bug fix, not a new primitive. The fix is minimal (one extra match arm).
The underlying issue: `(VariantName)` list form creates `Tagged { fields: [] }` in the
TypeDef but the runtime `EnumValue` has `fields: []` which the serializer treats as
unit-equivalent (emitting `Nil` body). The coercer now agrees.

Diagnostic path: a Rust probe test (`tests/probe_counter_actor_process_diag.rs`)
was used to surface the actual subprocess stderr error. The probe is left in the
test suite as additional coverage for the round-trip pattern.

---

## Test files

- `/home/watmin/work/holon/wat-rs/wat-tests/counter-actor-proof-thread.wat`
  Thread-tier Counter actor proof. 214 lines.

- `/home/watmin/work/holon/wat-rs/wat-tests/counter-actor-proof-process.wat`
  Process-tier Counter actor proof. 197 lines.

- `/home/watmin/work/holon/wat-rs/tests/probe_counter_actor_process_diag.rs`
  Diagnostic Rust probe (3 tests) used during investigation. Left as regression coverage.

---

## Suggested INTERSTITIAL corrections

If updating (user has authority; these are findings for the record):

1. **Enum unit variant syntax:** Change `Get`, `Reset`, `Shutdown` (bare symbols) to
   `(Get)`, `(Reset)`, `(Shutdown)` (list-form) or `:Get`, `:Reset`, `:Shutdown` (keyword-form).

2. **Enum payload variant syntax:** Change `(Increment :wat::core::i64)` to
   `(Increment (n :wat::core::i64))` (named field).

3. **Enum variant constructor separator:** Change `/` to `::` in all constructor
   call-sites: `:counter::Request::Get` not `:counter::Request/Get`.

4. **counter/dispatch parameter shape:** The server-side dispatch fn takes a `ThreadPeer`
   (not raw `server-rx!` + `server-tx!`). Raw channels are only visible inside the
   spawn-thread closure; dispatch receives the peer.

5. **Process-tier EDN round-trip:** Mark explicitly that `(VariantName)` enum variants
   with no fields are serialized as `#ns.TypeName/VariantName nil` and this round-trips
   correctly through the subprocess boundary.
