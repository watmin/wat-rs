# BRIEF — STONE 255.9: the keyword-only head, and the greens it forges

**Drawn 2026-09-22 against `main` @ `342ddd051`.** Floor 5959/5959, clippy 0, census `no STOP-8`.
Delta baseline **18**, and **the sample is now a committed file** —
`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt` (sha `33ede76c…`).
⛔ **USE THAT FILE. NEVER REBUILD THE SAMPLE BY INDEX AGAIN** — the recipe is an index over a growing
corpus, and two reconstructions of "the same" 179 files shared **4 entries**.

## ⛔⛔ THIS STONE EXISTS BECAUSE A CONVERSION CAN FORGE A GREEN

Measured, in the weigh, on the codemod's real output — not a hand-written guess:

```
(:wat::load-file! "missing.wat")   → rc=1   "load: file not found: missing.wat"
(wat/load-file!   "missing.wat")   → rc=0
```

`src/load/loader.rs::match_load_form` matches **Keyword heads only**:

```rust
let head = match items.first() {
    Some(WatAST::Keyword(k, _)) => k.as_str(),
    _ => return Ok(None),          // ← a SYMBOL head is silently "not a load form"
};
```

The codemod rewrites `(:wat::load-file! …)` → `(wat/load-file! …)`. The converted form is **never
flagged** — not malformed, not unresolved, not anything. ⛔ **A file whose load target is missing goes
from an honest failure to a PASS.** On a file that uses what it loaded, the only surviving symptom is
an `UnresolvedReference` for the missing content — the load itself is never named.

**26 tracked `.wat` files use the six load forms; ZERO are in `wat/`.** ⛔ **8d-iii converts all 26.**

⛔⛔ **AND THE DELTA GATE CANNOT SEE IT.** All 4 load-form files in the 179 sample are
`CLEAN → CLEAN` — **green before, green after, loads silently gone.** No count run in eight stones
could have caught this. **This is why this stone goes before 8d-iii and before the other three.**

## ⭐ THE STONE IS THE CLASS, NOT THE FORM

⚠ **A crude 3-line text probe** finds **83** `Some(WatAST::Keyword` head matches under `src/`, of
which **~14** sit within three lines of a silent `_ => None` / `_ => return Ok(None)` —
`runtime.rs` 4, `macros/expand.rs` 4, `rete/purity.rs` 2, `rete/kernel/stratify.rs` 2,
`load/loader.rs` 2.

⛔ **THAT IS A LEAD, NOT A CENSUS, AND IT IS MINE, SO DISTRUST IT.** `[[GREP IS NOT A CENSUS]]` — a
three-line window is not a parse, the counts double-count, and a matcher that declines a symbol head
**four** lines down is invisible to it. ⭐ **THE FIRST JOB OF THIS STONE IS THE REAL CENSUS:**

> **Every site that dispatches on a form's head and accepts ONLY the keyword spelling, where a
> SYMBOL spelling can legally arrive after 8d-iii.**

For each, say what it does with a symbol head today: **silently declines** (the dangerous one),
**errors** (visible, fine), or **cannot be reached** by a converted form (say WHY).

## ⛔ THE DOOR ALREADY EXISTS — do not write a recogniser per site

`ns_to_wat_path("wat", "load-file!")` → `:wat::load-file!` — an exact round-trip of what the codemod
produced. `canonical_identity` is the documented identity door and 255.1/255.6/255.7 routed three
subsystems through it. ⛔ **Route the head through the door. Do not add a second spelling test at
each site** — that shape has cost this arc four stones, and 255.4's version cost 32 reds.

## ⛔ NON-VACUITY — the cure must not open the gate

A cure that makes every symbol-headed list a candidate load form is worse than the defect. **Prove
all three, in one test:**

1. ⭐ `(wat/load-file! "missing.wat")` → **rc=1, and the message names the missing FILE** — the same
   failure the keyword form gives. **Not merely non-zero.**
2. ⭐ `(:wat::load-file! "missing.wat")` → **unchanged**, byte-for-byte the same diagnostic.
3. ⛔ `(some.other/thing "x")` → **still NOT a load form**, still declined exactly as today.

## The work

1. **The census** above. Report it whole, including the sites you decided were unreachable.
2. **Cure the load family** (all six forms) through the door.
3. **Cure, or REPORT with a reason, every other site the census finds.** ⚠ If a site needs a ruling
   rather than a fix, **name it and stop there** — do not guess the builder's intent.
4. ⭐ **Prove it on a REAL corpus file:** convert a copy of one of the 26, and show its loads still
   resolve. `examples/with-loader/wat/main.wat` is the smallest.

## The gate

- The three non-vacuity rows above, **in one test**.
- ⭐ **A converted real file's loads WORK** — step 4, shown, not asserted.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
- Delta **≤ 18**, measured **on the committed list file**.
- ⛔ **NOT ONE `.wat` CONVERTED** in the live tree. This is a `src/` stone.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole, never re-run to green, name the arm.
  ⭐ The last two executors each disclosed a red floor and **both stones were accepted because of it.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Ten stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** **Seventeen corrections across fourteen
  stones.** The last stone's most valuable finding was **out of scope and unasked**. ⭐ **If you find
  the real defect somewhere this brief never looks, go there and say so.**
- ⚠ **Reformat in its own commit, or not at all.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope — drawn, queued, not this stone

`type_denotation` at the two runtime string-table sites · the 416-test rete remainder ·
255.8's wrong-join acceptance · 8d-ii · 8d-iii.
