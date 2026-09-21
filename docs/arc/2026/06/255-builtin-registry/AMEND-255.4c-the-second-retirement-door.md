# AMEND — STONE 255.4c: the teach-fix added a SECOND retirement door. 3 → 32.

**Orchestrator's verification row, `37b8d7b43` (not pushed).**

## ⛔ FLOOR RED — 32 of 5939, UP from 3. One cause.

```
     Summary [ 287.206s] 5939 tests run: 5907 passed (8 slow), 32 failed, 22 skipped
```

**All 32 are `arc241` retirement/remedy tests** — `stone9_defenum` · `stone10_remedy` ·
`stone11_define_hard_cut` · `stone12_defalias` · `stone13` · `stone14_restricted_absorbed` ·
`stone15_zombie_purge` · `stone16_define_eval_residue` · `probe_def_not_special` ·
`wat_arc143_define_alias`.

**The arm:**

```
actual  : MalformedForm "malformed :wat::core::struct form:
          ':wat::core::struct' is retired; use ':wat::core::defstruct'"
expected: … 'did you mean: :wat::core::defstruct [replaces a retired form]'
```

## ⭐ THE DIAGNOSIS — and it is this arc's own defect, committed by the cure

Fixing *"17 rows are diagnosed only at runtime"* was right. ⛔ **But the fix added a SECOND
retirement door, and there was already one.**

**Arc 241 built the teaching path**, and it is still there:

- `src/remedy/mod.rs:235` — `RemedyKind::Retirement => "replaces a retired form"`
- `src/remedy/mod.rs:210` — the rendered remedy line
- `src/remedy/mod.rs:54` — `retirement_table_names()`

Your new `infer_list` arm consults the table **first** and emits a **plain `MalformedForm`** —
preempting the arc-241 path and **dropping the remedy**. The new message is *less* teaching than
the one it displaced: an author now gets a bare sentence where they used to get
`did you mean: … [replaces a retired form]`.

⛔⛔ **A SECOND DOOR IS THE DEFECT THIS WHOLE ARC EXISTS TO REMOVE.** 255.2 refused to rune
`one_param_spec` and moved `is_binder_marker` into `wat-reader` **precisely so there would not be a
second `:-` recogniser.** This is the same shape, in the same arc, three stones later.

## The fold — route through the EXISTING door, do not add one

⛔ **Fold into `37b8d7b43`.** Not a repair commit.

**The requirement is unchanged and still right:** the 17 rows must be diagnosed **at check time**,
not at runtime. **What must change is HOW:**

1. ⭐ **The 17 rows reach arc 241's remedy path**, producing the *same* diagnostic shape the other
   retired forms produce — remedy, marker, note. **Not a new plainer message.**
2. ⛔ **The early `is_resolvable_call_head` / `covers` pass-through can stay** — letting a retired
   head survive resolution *so that check can teach* is correct and is not a second door. **The
   second door is the new `infer_list` arm that answers instead of delegating.**
3. **Prove the teaching is equal.** `:wat::kernel::HandlePool::new` and `:wat::core::struct` should
   produce the **same kind** of diagnostic — one is not a lesser citizen because it arrived through
   a different resolution path.

⚠ **If arc 241's path genuinely cannot serve these 17** — because they are resolution-stage and it
is check-stage — **that is a finding, and say it.** Do not fork the diagnostic to work around it.
`[[feedback_a_rejected_option_returns_in_new_clothes]]`.

## Verified good, unchanged by this red

| row | result |
|---|---|
| the 3 previous reds | ✅ all gone (`non_vacuity`, `recorded_migration_fixtured`, `retirement_table_reachable`) |
| the non-vacuity floor | ✅ **2459 paths driven, floor > 1500**, marker present — a real number, read from a real run |
| the replay fixture | ✅ synthetic pre-image with **near-misses** (`Bytes/to-hex` already-slash, `Cache::GetRequest` nested type name) — the near-misses are what make it a test |
| crate clippy | ✅ 0 |

⭐ **The replay fixture's near-misses deserve naming:** a pre-image that only contains what the
codemod *should* change proves nothing about what it should *leave alone*. Including both is the
difference between a fixture and a demo.

## After the fold

State crate clippy + the arc241 suite + the three walls from 255.4b. I re-run floor, workspace
clippy, census and the delta. **Do not push. Do not start 255.5.**
