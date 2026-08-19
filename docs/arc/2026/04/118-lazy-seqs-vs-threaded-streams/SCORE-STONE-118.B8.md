# SCORE — STONE 118.B8 · the arc's tail. All three discharged.

Rider: sonnet, one flight, Parts 1–2, no commit. Part 3 run by the orchestrator. Every row below
re-verified against the disk by the orchestrator's own invocation.

| # | what | expected | RESULT |
|---|---|---|---|
| 1 | ★ `dorun`'s peak RSS FLAT in n | slope ≈ 0, with a LINEAR before | **PASS**, re-run independently — see below |
| 2 | `dorun` still forces every element | n in → n forces | **PASS** — 5/5; and the retention probe's own non-vacuity line fired at both sizes |
| 3 | recursion in TAIL position | the arm's whole body | **PASS**, read the diff |
| 4 | `doall` UNCHANGED | still `(into [] coll)` | **PASS** |
| 5 | `extract_lazyable_elem` SURVIVES | function + 6 call sites intact, order gone | **PASS** — and see the correction below |
| 6 | the class census, from a FORM TREE | inventory, every sibling dispositioned | **PASS** — `MEASURED-118.B8-the-class-has-exactly-one-member.md` |
| 7 | floor | ≥4772/0, 19 skipped | **PASS** — `4772 tests run: 4772 passed, 19 skipped` |
| 8 | clippy | 0 | **PASS** — 0 |
| 9 | ignores | 13 | **PASS** — 13 |

## Row 1, re-run by the orchestrator on its own build

```
n=100,000   maxRSS 44,232 KB   printed 99999
n=800,000   maxRSS 44,400 KB   printed 799999
```

**8× the input, +168 KB — 0.4%.** The printed value is the probe's non-vacuity guard: it fires only
on the LAST element, so the walk provably reached the far end at both sizes. The rider's four-point
BEFORE column is linear and internally consistent with an n-element Vector (+5,084 → +9,292 →
+18,648, doubling with n), which is mechanically what `(into [] coll)` must allocate.

**O(n) live → O(1) live.** Same class as B3's memo deletion, not a speed-up.

## ★ THE CORRECTION I MADE TO PART 2 — a doc that pinned line numbers into the file it lives in

The rider's rewrite was right in substance and struck the stale deletion order exactly as briefed.
But it wrote the six call sites as **line numbers** — `infer.rs:734, 810, 887, 1016, 1079, 1142` —
copied from the brief, which had read them *before* the edit. **The same edit inserted 14 lines
above them.** Measured after: `748, 824, 901, 1030, 1093, 1156`. Every number was wrong on arrival.

Not fixed by renumbering — **replaced with names**: `infer_map`, `infer_filter`, `infer_foldl`,
`infer_take`, `infer_drop`, `infer_seqable_to_stream`, plus the grep that checks them. A doc that
pins line numbers into its own file is invalidated by its own next revision; names are not. That is
the rung above renumbering, and it is the same class as the goldens carve-out — except here the
artifact and its target are the same file, so the staleness was instantaneous and self-inflicted.

(The two unnamed "siblings" the rider could not name are now named: `infer_foldl` and
`infer_seqable_to_stream`.)

## Part 3 — the census, and what it actually found

```
44 .wat files · 373 defns · 12 lazy-cell walkers · 4 growth hits · 1 in both
```

`distinct-walk` is the class, alone. Full write-up and the reproduce command:
`MEASURED-118.B8-the-class-has-exactly-one-member.md`. **The finding is not the count — it is that
only one verb in the surface needs unbounded history**, which is why the count is one and why a
future `frequencies`/lazy-`group-by` would join it.

Two instrument defects caught mid-census and written into the record: a discriminator that did not
discriminate (`stream::next` is consumption, common to the harmful walker and the benign drain), and
a population one file short (`wat/**/*.wat` descends one level; `find` caught it).

## STOP-1 was investigated and correctly did NOT fire

The rider built a real expand-time trigger — a `defmacro` whose own program body calls `dorun` — got
`UnknownFunction`, then **isolated it properly**: stashed B8's change, rebuilt, reran against the
ORIGINAL `into`-based `dorun` (byte-identical failure), and tested untouched `:wat::core::into`
(same error class). The restriction is general and pre-existing. STOP-1's wording was *"if
expand-time `dorun` now fails where it previously worked"* — it did not previously work either.

**That is the differential this project asks for, and the rider ran it unprompted.** Evidence
transcribed into **task #107**, which now carries all three arms.

## Honest deltas

- **The rider caused a real RED and reported it in full.** It left the expand-time probe in
  `wat-scripts/scratch-pad/`, where `every_wat_scripts_file_loads` runs `startup_from_source` and
  went red. Caught and fixed by the rider itself, then disclosed rather than quietly cleaned up.
  ★ **A `wat-scripts/` file must LOAD by construction, so a probe proving a LOAD-TIME failure can
  never live there.** Its home is `tests/` — arc 255 recorded this exact pattern. Noted in #107:
  rebuild the control there before that task is struck, because a finding that lives only in prose
  is one revision from unfalsifiable.
- The rider's independent golden census (`grep -rl ':file "src/'` → 14 files, 8 pointing at real
  `.rs` paths) matches the orchestrator's exactly. Nothing to ratify — no `infer.rs` fixture exists.
- Wall-clock ~50 min against a 35–55 prediction, the overage entirely in the STOP-1 differential.
- Line counts: `wat/seq.wat` +14/−3, `src/collection/infer.rs` +31/−9 (doc only), two new probes,
  one new census.

## ⚠ WHAT THIS STONE UNBLOCKS

**Arc 118's INSCRIPTION.** All three owed items are discharged, and every affirmative cut in the
stone names a home that exists. The pre-INSCRIPTION grep has NOT been run — that is the next act,
and it is the builder's call whether to inscribe now.
