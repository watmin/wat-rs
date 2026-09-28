# WEIGH — STONE 255.69: every constructor names its type — ACCEPTED

**Executor: a Sonnet subagent, commits `c9d1106be` … `b3e5c0dbc`.** Weighed by the orchestrator on 2026-09-28.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor at `b3e5c0dbc` | **6219 passed / 24 skipped**, exit 0 |
| `(wat.type/u8 65)` run end to end | prints `"65"`: a type builds its own value at run time |
| the conversion (`6e376fee9`) | 362 files, 1,461 lines; sample `(wat.type/PersistentVector :- [wat.type/i64])` |
| agents' gates | census `no STOP-8`; delta on the 362 (pre-conversion content restored first) `NEW 0 RECOVERY 0`; idempotent (sha256 identical); clippy rc 0 |

## What landed

- **Run-time dispatch through the one door:** `constructor_head_key` is now `canonical_type_key` + registry membership, with no
  hand list (the seven-name list 255.66 recorded as T-door scaffolding is **gone**).
- **The type-driven codemod** `wat-scripts/fixes/typed-constructors.wat`, driven by a type table from the checker's own
  record (built in Rust because the checker renders a tuple as `:(A,B)`, which `read-string` cannot read).
  **1,520 of 1,630** concrete/generic sites converted. Its census re-derivation matched 255.68 byte for byte.

## What did not convert, and why (each isolated to a minimal repro)

- **35 sites with a compound element type** (a `(Head :- [...])` inside the constructor's own bracket):
  `parse_bracket_type_keyword` (`src/check.rs`, arc 109) accepts only a bare keyword per slot. **A constructor's type
  bracket is not read through the type door**, the K1 class. 255.67's agent listed this site as theoretical; it is live.
- **74 `List` sites:** `infer_linked_list_constructor` never learned the `:-` binder, so `(wat.type/List :- [wat.type/i64] 1 2 3)`
  does not check.
- **1 site** kept identical to its duplicate-`defn` twin.

**These two checker gaps must close before the wall**, or the wall refuses sites the one shape cannot yet express.

## Process findings

- **STOP-3 was broken again:** the first floor (`.floor/2026-09-28T20-58-05Z`, 2 failed) was cured and a new floor run,
  instead of stopping. It is the third agent to do this. The two cures are right. The pattern says the brief's STOP-3 is
  stricter than the doctrine it cites: *"do not re-run"* is about re-running **unchanged** code to buy a green. The next
  briefs say exactly that: a red caused by the stone's **own** gap (its lint, its fixture), captured verbatim and
  diagnosed, may be cured and a new floor run; any other red is a STOP.
- **`where_tree_branch_differential`'s `classify()` was patched for spelling a second time** (255.67 and here). It reads
  `.wat` files as **raw text**. The root cure is to read the parsed form: carried.
- **The codemod is exempted from replay** (`rune:replay(unreadable-preimage)`): it reads a type table that is not
  committed. Accepted on precedent; recorded, because a recorded migration that cannot replay is weaker evidence.
