# EXPECTATIONS — 294.j · `edn_shim` forgets the algebra

Written **before** the strike so the result cannot move the goalposts. Scored by the orchestrator
against its **own** re-run, never against the rider's report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the tag family is gone | `grep -rn 'wat-edn\.holon' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` | **0 lines** |
| 2 | the seven fns are gone | `grep -rnE 'fn (holon_ast_to_edn\|edn_to_holon_ast\|edn_to_holon_ast_natural\|edn_holon_tag_to_ast\|write_holon_ast_tagged\|read_holon_ast_tagged\|read_holon_ast_natural)' src/` | **0 lines** |
| 3 | the export line is gone | `grep -n 'read_holon_ast' src/lib.rs` | **0 lines** (or STOP-3 fired and is reported) |
| 4 | one encode arm | read `edn_shim.rs` at the `Value::holon__HolonAST` arm | routes through `holon_to_watast` + `watast_to_edn` |
| 5 | no mode selector | `grep -n 'tag.namespace() == "wat-edn' src/edn_shim.rs` | **0 lines** |
| 6 | the tag is refused | probe row 3 | `#wat-edn.holon/String "x"` → decode **error** |
| 7 | directives survive | probe row 4 | Thermometer renders `(:wat::holon::Thermometer …)` |
| 8 | goldens regenerated | `git diff --stat tests/value/*.edn` | **3 files changed**, every new value plain EDN |
| 9 | probe green, unignored | `cargo nextest run --release -E 'test(holon_bare_leaf)'` | **4 passed, 0 skipped** |
| 10 | floor | `scripts/floor.sh` → the **Summary line** | **0 failed, 0 timeout** |
| 11 | clippy | `cargo clippy --release --all-targets` | **0** |
| 12 | ★ waterline held | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` | **13** |

Rows **1**, **6**, and **12** are load-bearing.

- **Row 1** is the strike's whole claim. A non-zero count is a miss, not a rounding error.
- **Row 6** is the *dormancy* check. Rows 1–5 can all pass while the decoder still happily accepts
  the old spelling — that is the failure mode this stone exists to prevent, and only row 6 sees it.
  Row 7 is its non-vacuity partner: without row 7, a decoder broken *for everything* would score row
  6 green.
- **Row 12** is not bookkeeping. See the stone's §"Gate 11 is not bookkeeping".

## Independent prediction

**Runtime: 25–45 minutes.** Basis: 294.i (the `.opaque` strike) covered 24 sites across the same
file and ran inside that band. This is more sites (40 in `edn_shim.rs`) but the work is
*deletion plus one substitution* rather than 14 per-type destination decisions, and the substitution
is a composition that already exists. **2× cap = 90 minutes.**

**Expected diff shape:** heavily negative. Roughly −200 lines in `edn_shim.rs` (the 16-arm encoder,
the three readers, the tag switch), +5 to +15 (one arm, one visibility change, one collapsed reader),
plus the probe rewrite and 3 regenerated goldens.

## Trap-doors — named before the strike so a hit is data, not a surprise

1. **`holon_to_watast` is `runtime.rs`-private.** The rider must widen it. If `edn_shim` cannot
   reach `runtime` without a cycle, the honest move is to relocate the lowering, not to re-inline a
   copy of it into the shim. **A second copy is the exact defect this stone removes.**
2. **Tests asserting the old wire text.** 26 sites across 8 test files. Most should become plain-EDN
   assertions — that is the strike working. Watch specifically for
   `tests/types/probe_arc234_7b_holon_record_roundtrip.rs` (5 sites): it is a *round-trip*, so if it
   needs more than a text update, the round-trip genuinely changed and that is a finding.
3. **`tests/comms/probe_arc294_holon_wire_is_plain_edn.rs` (4 sites + 2 in its `.wat`).** This probe
   is the record-side sibling of this stone and its rows 3–4 are explicitly the non-vacuity guard
   *"this stone takes the hologram off the WIRE; it must not take it out of the VALUE."* **The same
   guard applies here.** If those rows move, the strike deleted the index rather than deriving it —
   the opposite of the cure.
4. **SlotMarker is documented non-round-trippable** (`runtime.rs:20727`, arc 073, "a substrate-internal
   sentinel"). Pre-existing. Row 7 pins the *rendering*; it does not claim a round-trip that never
   existed. Do not let a green row 7 be read as one.
5. **`reconstruct_holon_record` must not move.** Verified this session: it never touches
   `#wat-edn.holon` tags — 294.g already moved holon records off the hologram wire. If its behaviour
   changes, something outside this strike's scope was hit.

## How this gets scored

Each row re-run by the orchestrator, on its own floor, before any commit. The rider's report is a
**hypothesis**; a current `file:line` or a fresh Summary line is the only evidence that counts. Then
one commit — probe, strike, goldens, paperwork together, green, zero new ignores.
