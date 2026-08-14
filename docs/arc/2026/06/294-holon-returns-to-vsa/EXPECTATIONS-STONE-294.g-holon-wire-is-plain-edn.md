# EXPECTATIONS — STONE 294.g: the holon record's wire is PLAIN EDN

Written **before** the strike, against HEAD `0c4cd415`
(floor **4408 run / 4407 passed / 1 failed** — the 1 is this stone's probe row, red by design; clippy 0).

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | **★ THE STONE** | `cargo nextest run --release -E 'binary_id(wat::comms) and test(probe_arc294_holon_wire)'` | **4 passed, 0 failed.** At HEAD: 3 passed / 1 failed. The delta IS the stone |
| 2 | **★ the non-vacuity rows never stopped passing** | same run, rows 3-4 | green **before and after**. A red here means the hologram was DELETED, not derived — the opposite of the stone, and the one way row 1 can be bought dishonestly |
| 3 | the control never moved | same run, row 1 | `#t/Plain {:x 1 :y 2}` still. The plain record is the reference shape; if it changed, the fix reached the wrong arm |
| 4 | the probe was not edited | `git diff --stat tests/comms/probe_arc294_holon_wire_is_plain_edn.*` | **empty** (STOP-4) |
| 5 | **★ the encode dispatch is GONE** | `grep -n "HolonForm::Hologram" src/edn_shim.rs` | **0 hits in the encode arm.** One arm, not two — if the match survives with both branches emitting maps, the duplication survived |
| 6 | the discriminator MOVED, not vanished | read the decode diff | the holon/base decision reads the **type registry's `Nature`**; NOT the body shape, NOT a marker key injected into the map (STOP-1) |
| 7 | the derive side is reused | `grep -c "fn build_holon_hologram" src/runtime.rs` | **1** — the receiver calls the existing one; no second implementation minted |
| 8 | arc-093's round-trip still lives | `grep -c "edn_holon_tag_to_ast" src/edn_shim.rs` | **non-zero** — the HolonAST vocabulary serves a second master and must keep working (STOP-2) |
| 9 | blast radius | `git diff --stat` | `src/edn_shim.rs` plus whatever the decode path needs; goldens/tests as surfaced. **Not** `src/runtime.rs`'s derive logic |
| 10 | clippy | `cargo clippy --release --all-targets` | zero warnings (`-D warnings` is the wall) |
| 11 | **floor** | `scripts/floor.sh`, Summary line read whole | **4408 / 4408**. Any other number either way is a finding, not a rounding |

**Row 1 is the stone; row 2 is what keeps it honest.** A green row 1 with a red row 2 is the hologram
thrown away — the failure mode this probe was built to make unbuyable. **Row 5 is the extirpare check:**
the goal is not "both branches produce the same thing," it is "there is one branch."

**Row 6 is the design's load-bearing claim.** Today the decoder distinguishes holon from base by BODY
SHAPE (`edn_shim.rs:3789`'s own comment says so). That signal disappears with this change. If it
reappears as a sniff or an injected marker key, the coupling was renamed rather than removed.

## Runtime prediction

**45–75 minutes.** The encode side is a deletion (two arms → one, and the surviving one is already
written). The decode side is real work: move a discriminator and rebuild fields from a map. The cost is
the fallout — this changes an observable wire form.

Time-box: 150 minutes.

**Predicted overrun: STOP-3.** Some `.edn` golden or test almost certainly pins the hologram wire form.
That is a ruling, not an edit, and it comes back to the orchestrator.

## Trap doors — named in advance

- **★ Editing the probe to match the output.** The one way to a meaningless green. Row 4 catches it;
  STOP-4 says it in the brief.
- **★ Buying row 1 by dropping the hologram.** If `HolonForm` stops being populated at construction,
  the wire gets clean and `cosine` goes `Degenerate`. Rows 2 exist solely for this and they face the
  outcome rather than assuming `Similarity`.
- **Sniffing the body, or adding a `:holon true` key to the map.** Both keep the encode/decode coupling
  alive under a new name. STOP-1 and row 6.
- **Deleting `holon_ast_to_edn`/`edn_holon_tag_to_ast` as newly-orphaned.** They serve arc 093's
  substrate-internal HolonAST round-trip too — a second master this stone does not touch. Row 8, STOP-2.
- **Deciding which goldens are defects.** Not the rider's call. The identical STOP on stone 279.2 came
  back "real contract, revert" — the tests were right and the design was wrong.
- **Trusting a narrow filter.** The gate is the whole floor on purpose; see the brief's rationale and
  R7's `MVRVS AVCTOREM NON NOVIT`.

## What this stone does NOT claim

It does not rename `#wat-edn.*` → `#wat.*` (~118 sites, deliberately after — renaming what we delete is
wasted motion). It does not annihilate `HolonRepresentable` (flaw #4). It does not touch HolonAST's
code-AST duty (flaw #5 / task #91). It does not teach `#holon` to the wire — that is the **anonymous**
case and this stone is the **declared** one.

The honest claim, and only this: **a declared holon record crosses the wire as its class tag and its
fields, exactly as a plain record does; the hologram is derived on arrival and never serialized.**
