# BRIEF — Arc 218 Stone 218.6e — IMPECCABLE polish (6 items)

**Stone scope (sonnet portion):** 6 items closing the 1 L1 + 5 L2 from VIGILIA-REPORT-2026-05-22-IMPECCABLE.md. Two cascade-renames from 218.6d; two USER-GUIDE/README doc fixes; two rune rewrites (one closes the L1, one re-categorizes per purgare SKILL).
**Type:** Sonnet Mode A.
**Time budget:** 5-10 min target; 20 min STOP.
**Depends on:** Stone 218.6d (`9cc804e`) + VIGILIA-REPORT-2026-05-22-IMPECCABLE.md (`1c8a9d0`).
**Calibration:** 10 stones in series below lower band. Smallest stone yet — 6 surgical edits across 5 files.
**Unblocks:** Vigilia FINAL recast — if CONVERGED across all 7 spells, IMPECCABLE achieved within wat-edn scope.

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Section A — intueri cascade renames (2 items)

**A.1 — `lexer.rs` local `open_pos` → `quote_start`**

Stone 218.6d renamed `lex_string_escaped`'s parameter `open_pos` → `quote_start`, but the CALLER's local variable in `lex_string` kept the old name. Three occurrences need rename:

- `lexer.rs:191` — `let open_pos = self.pos;`
- `lexer.rs:198` — `Error::at(open_pos, ErrorKind::UnclosedString)`
- `lexer.rs:206` — `self.lex_string_escaped(open_pos, body_start)`

After rename: caller's local + callee's parameter share the name `quote_start` — same concept, same name across the call boundary.

**A.2 — `writer.rs` `all_scalar` → `all_inline`**

Stone 218.6d renamed `is_scalar` → `is_inline_value` (since the function includes Inst + Uuid which aren't scalar in any conventional sense). The wrapper `all_scalar` was not updated. Two occurrences:

- `writer.rs:71` — `fn all_scalar(items: &[Value]) -> bool { items.iter().all(is_inline_value) }`
- `writer.rs:87` — `} else if items.len() <= 8 && all_scalar(items) {`

Rename `all_scalar` → `all_inline` (or `all_inline_values`; sonnet's call) to match the renamed delegate. Doc comment may need an update too.

### Section B — cernere doc fixes (2 items)

**B.3 — USER-GUIDE §7 code example uses demoted functions**

`docs/USER-GUIDE.md:404-422` — the §7 API code example imports + uses `edn_to_json` and `json_to_edn` as if they were public:

```rust
use wat_edn::{to_json_string, to_json_string_pretty,
              from_json_string, edn_to_json, json_to_edn};
...
// Or work with serde_json::Value directly
let jv: serde_json::Value = edn_to_json(&v);
let back: OwnedValue = json_to_edn(&jv)?;
```

Stone 218.6c demoted both to `pub(crate)`. **This example would FAIL TO COMPILE today.** Fix:
- Line 405: remove `, edn_to_json, json_to_edn` from the import list
- Lines 419-421: remove the entire "Or work with serde_json::Value directly" comment + 2-line block (it's the only place those two functions appear in the example)

After fix, the §7 example shows 3 public JSON functions (`to_json_string`, `to_json_string_pretty`, `from_json_string`) + their normal use. The verbatim JSON output comment at line 411 stays.

**B.4 — README + USER-GUIDE test counts stale**

Three sites carry stale test counts; current truth is **344** (verified post-218.6d ship):

- `crates/wat-edn/README.md:45` — `"313 Rust tests + 39 Clojure tests, all green"` → `"344 Rust tests + 39 Clojure tests, all green"` (or whatever the current Clojure count is; verify via `clojure -M clj/test-runner.clj` if a tool exists, else preserve `39 Clojure` as written)
- `crates/wat-edn/README.md:97` — `"Self round-trip is \`cargo test -p wat-edn\` (342/342 passing)."` → `"(344/344 passing)"`
- `crates/wat-edn/docs/USER-GUIDE.md:792` — `"Tests:  313 Rust + 39 Clojure (96 assertions)"` → `"Tests:  344 Rust + 39 Clojure (96 assertions)"`

Sonnet may verify the actual count via `cargo test --release -p wat-edn 2>&1 | grep "^test result:" | awk '{sum+=$4} END {print sum}'`. If the count differs from 344 (e.g. has shifted since), use the actual current count consistently across all 3 sites.

### Section C — purgare rune rewrites (2 items)

**C.5 — `to_json_string_pretty` rune (L1) — drop false consumer claim**

`crates/wat-edn/src/json.rs:179-184` currently reads:

```rust
// rune:purgare(public-api) — symmetric pretty variant of to_json_string
// (consumed by src/edn_shim.rs for WAT_TEST_OUTPUT cargo integration per
// arc 116). Impressive JSON bridges ship both compact and pretty forms;
// removing this would leave an asymmetric surface. The pretty variant
// is the natural API for human-readable JSON output (debug logs, error
// envelopes, REPL inspection).
```

**The 2nd-line claim is FALSE.** `edn_shim.rs:92` calls `write_pretty` (EDN pretty), NOT `to_json_string_pretty` (JSON pretty). Vigilia caught the lie. Rewrite to be honest:

```rust
// rune:purgare(public-api) — symmetric pretty variant paired with
// to_json_string (which IS actively consumed by src/edn_shim.rs:105,166
// for WAT_TEST_OUTPUT cargo integration per arc 116). to_json_string_pretty
// itself has no current direct caller; justification is symmetric-
// completeness with the live compact variant + future Clojure-IPC bridge
// surface (crates/wat-edn/docs/IPC-BRIDGE.md). Impressive JSON bridges
// ship both compact and pretty forms; the pretty variant is the natural
// API for human-readable output (debug logs, error envelopes, REPL).
```

Or sonnet's cleaner wording — the constraint is: NO false "consumed by edn_shim.rs" claim; the symmetry + vision justification stands on its own without inventing consumers.

**C.6 — `write_to` rune category → `future-fixture`**

`crates/wat-edn/src/writer.rs:195-200` currently uses `purgare(public-api)` but the justification is forward-looking (IPC-BRIDGE.md vision). Per purgare SKILL.md, `future-fixture` is the more honest category — "rune retires when the downstream lands." Sonnet rewrites:

```rust
// rune:purgare(future-fixture) — buffer-reuse ergonomic retained for
// the future Clojure-IPC bridge per crates/wat-edn/docs/IPC-BRIDGE.md:95;
// no current external caller. Symmetric with the actively-consumed
// `write` fn. The append-to-existing-buffer shape is the canonical
// Rust pattern for output composition. This rune retires when the IPC
// bridge ships and write_to gains a real caller (per purgare SKILL:
// "rune retires when the downstream lands").
```

Or sonnet's wording — the constraints: category is `future-fixture`; retirement criterion is explicitly stated.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- No new runes — 218.6e refines 2 existing runes only; doesn't add any
- 2 temperare runes on json.rs PRESERVED intact (CLEAR per all 7 spell casts)

## Your scope (sonnet)

Execute Sections A, B, C in any order — items are independent. 6 surgical edits.

### Verification (must run before SCORE)

1. `cargo build --release -p wat-edn` — 0 warnings
2. `cargo test --release -p wat-edn` — expected 344 PASS (no count change; all fixes are renames/doc/rune wording)
3. `cargo test --release --lib -p wat` — 824/0 PASS (renames are internal; no external API touched)
4. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings
5. From `crates/wat-edn/interop-tests/`:
   - `cargo build --release` — 0 warnings
   - `cargo clippy --release --all-targets -- -D warnings` — 0 warnings
6. **Interop-tests 4 handshakes** (mandatory):
   - `cd crates/wat-edn/interop-tests`
   - `cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj`
   - `clojure -M clj/produce.clj | cargo run --release --bin reader`
   - `cargo run --release --bin shape_matrix | clojure -M clj/consume_shapes.clj`
   - `clojure -M clj/produce_shapes.clj | cargo run --release --bin shape_matrix_reader`

**NOTE on handshakes:** if sub-agent permissions deny the piped form (precedent: 218.6b + 218.6c + 218.6d), ship the rest cleanly + write SCORE marking row 8 as "pending orchestrator-side verification". Orchestrator runs during scoring. Do NOT block.

**Write `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.6e.md`** mirroring 218.6d shape.

## STOP triggers

- **STOP-1 (A.1 surfaces additional `open_pos` site):** vigilia named 3 occurrences; if a 4th surfaces, report.
- **STOP-2 (A.2 surfaces additional `all_scalar` caller):** vigilia named 1 caller; if more surface, report.
- **STOP-3 (B.3 surfaces additional demoted-function usage in USER-GUIDE):** the 2 demoted functions should appear ONLY at lines 405, 420, 421; if other USER-GUIDE sites surface, report.
- **STOP-4 (B.4 test count is not 344):** verify via cargo + use the actual current count consistently across all 3 sites.
- **STOP-5 (rune wording rejected by clippy or breaks doc-test):** unlikely (these are comments), but if cargo build fails on the rewritten runes, report.
- **STOP-6 (20 min elapsed):** wall-clock STOP.

## Out-of-scope

- Adding new runes — discipline forbids; 218.6e only refines existing runes
- Touching the 2 temperare runes (CLEAR per all spells) — they stay
- Touching `to_json_string` / `from_json_string` / `write` / `write_pretty` live APIs
- Any new public surface
- Encoding doctrine / syntax changes
