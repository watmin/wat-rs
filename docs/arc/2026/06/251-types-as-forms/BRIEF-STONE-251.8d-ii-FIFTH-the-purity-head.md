# BRIEF — STONE 251.8d-ii (FIFTH DRAW): the purity head

**Drawn 2026-09-22 against `main` @ `60bae1176`.** Floor 5993/5993, clippy 0, census `no STOP-8`.
Delta **3**, RECOVERY **0**. ⭐ **Heresy ledger: 232** (`tests/lint/keyword_heresy_ledger.rs`).

## ⭐ THE ADDRESS — AND THIS TIME THE ORCHESTRATOR VERIFIED IT

⛔ **The FOURTH draw's brief convicted `collection/transform.rs:304-340`. That was WRONG** — measured:
`awk 'NR>=290 && NR<=360' … | grep -c '":wat::'` → **0**. It is `sort$native`'s *call site*, not the
gate. ⭐ **255.13's lint found the real address by DATA FLOW, and the orchestrator has now read the
code itself:**

| site | shape |
|---|---|
| `src/rete/purity.rs:1232` — `classify_expr`'s head extraction | ⛔ `Some(WatAST::Keyword(k, s)) => …, _ => "<structural form>"` — **a SYMBOL head becomes the literal string `"<structural form>"`** |
| `src/rete/purity.rs:260` — `intrinsic_meta(head: &str)` | keyword-keyed table (`rete_op_for`) |
| `src/rete/purity.rs:2116` — `effectful_by_prefix(head: &str)` | `head.starts_with(":wat::kernel::")` · `":wat::io::"` · `":wat::holon::"` · `":wat::eval-"` |

⭐ **12 of the crate's 14 shape-B sites are in this ONE file.** In the converted floor this surfaced
as **468 log occurrences naming exactly ONE callee** — `wat.core/<`:

```
comparator `:wat::core::Fn` is not pure: `wat.core/<` is not proven pure
  … :head ":wat::core::sort$native"   wat/core.wat:1555
```

The converted stdlib passes the **Symbol** `wat.core/<`; the table knows the **keyword**. **`sort`
refuses before any comparison runs.**

## ⛔⛔ THIS CURE RUNS IN THE PERMISSIVE DIRECTION — AGAIN

**A purity gate is DEFAULT-DENY.** Teaching it the symbol spelling makes it **accept more**.
⛔ **That is the red → false-green direction, and 255.12 is the precedent: its adversarial row caught
its own first draft.**

⭐ **NON-VACUITY, MANDATORY, AND IT IS THE STONE:**

1. `wat.core/<` (a genuinely pure comparator) → **accepted**, same as `:wat::core::<`.
2. ⛔ **A genuinely IMPURE comparator is STILL REFUSED — in BOTH spellings**, with the same located
   reason. `effectful_by_prefix` exists to catch `:wat::kernel::`/`:wat::io::` heads;
   ⭐ **`wat.kernel/println` in a comparator must still be refused.**
3. ⛔ **A head that is genuinely unknown is still unknown** — the cure must not make
   `"<structural form>"` into "anything I can't parse is fine."

⚠ **`intrinsic_meta` and `effectful_by_prefix` take `head: &str`.** If you normalize at extraction,
they are fixed for free — ⛔ **but CENSUS THEIR OTHER CALLERS.** A second caller passing a raw head
leaves the hole open and the ledger will still count it.

## The work

1. **Cure the head read through the identity door**, one door. ⛔ **No second spelling test per table.**
2. Census the other callers of both helpers. **Report what you find.**
3. The three non-vacuity rows, in one test.
4. ⭐ **Re-measure the conversion:** convert live `wat/` (64 paths, explicit list), build, run the
   floor. **The 416 is the number to move.**

## The gate

- ⭐ **THE LEDGER MUST DROP.** `LEDGER_TOTAL = 232` → lower, ⛔ **and RE-FROZEN in the same commit**
  (the lint fails in both directions by design and tells you so). ⭐ **Report the new shape mix** —
  B should fall by ~12.
- ⭐ **The converted floor: 416 → ?** ⛔ **Report the number even if it is 416.** ⚠ **The 416 are
  CLASSIFIED, NOT DIAGNOSED** — 255.13 and the fourth draw both say so, and the third draw's
  *"323 tests bought by two sites"* warns the classes interact. **DO NOT promise 416 − 131.**
- The three non-vacuity rows.
- `scripts/floor.sh` green **on the landable (unconverted) state**; clippy 0; census `no STOP-8`;
  `scripts/replay/delta.sh` — baseline **3**, ⛔ **RECOVERY non-zero is a STOP**.
- ⭐ **LAND THE CONVERSION ONLY IF THE CONVERTED FLOOR IS GREEN.** Otherwise `git checkout -- wat/`,
  rebuild, report. **Three draws have stopped correctly and all three were accepted.**

## ⛔ Operational traps — every one has cost this arc time, two of them TODAY

- ⛔⛔ **NEVER `git add -A`, and NEVER invoke cargo, while a conversion is writing `wat/`.** The
  orchestrator pushed 4 half-converted stdlib files to `main` today (`d9b8a82fa` → `3af0e5d9d`); the
  fourth-draw rider rebuilt the binary around a half-converted stdlib. **Explicit paths, and wait.**
- ⛔ `git ls-files 'wat/**/*.wat'` returns **31 of 64**. Use `git ls-files | grep -E '^wat/.*\.wat$'`.
- ⛔ The stdlib is `include_str!`'d — **rebuild between every step and say you did.**
- The conversion takes **~22 min**; Bash caps at 600 s — **background it.**
- ⚠ **A fast floor is a symptom, not a speedup** (24 s = the stdlib did not load; ~330 s = healthy).
- ⚠ **`cargo clippy` can report rc 0 from CACHE in 0.07 s.** Touch a file or check the elapsed time.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole, never re-run to green, name the arm.
  ⚠ One bookkeeping exception: `harvest_wrap_split` is diagnosed-unsound
  (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`). **If it fires, CITE and REPORT** — do
  not treat as a pass, do not re-run to clear. ⭐ **Name it either way.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Sixteen stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Twenty-two corrections across twenty
  stones** — the last one proved this orchestrator convicted **a file with no keyword comparison in
  it at all.** **Assume a twenty-third.**
- ⚠ **Reformat in its own commit, or not at all.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

Shape **E** (125 sites, 78 in `check.rs`) — ⭐ **its own campaign; the terminal cut needs A+E at zero,
not just heads** · `arm.rs`'s 2 shape-B sites (255.9 dispositioned them *"already both — fine"*;
⭐ **reading both spellings is HALF the cure**) · `is_type_equatable`/`is_type_orderable` parametric
residues · 8d-iii · `is_quasiquote_form` · `Ngram` · variant tags in declarations.
