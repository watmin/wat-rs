# SCORE 6 — replay batch 3: grok-rete #126 → #152

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent brief: `BRIEF-6-replay-batch-3.md`. Start census `.census/2026-09-15T06-53-02Z.txt` files=2077.
HEAD: `55969600a` (#152). #126 folded golden recapture `ef8ded525`.

```
#139 first floor (captured, not re-run)  .floor/2026-09-15T19-06-20Z
     Summary [223.564s] 5542 tests run: 5541 passed, 1 failed, 22 skipped   exit=100
     FAIL probe_supervisor_select_lost::select_prime_yields_lost_when_process_child_crashes
     golden :line 1525 vs actual 1526 (freeze.rs rust_caller_span!)
     FOLDED into #126

#139 fold tip  scripts/floor.sh   .floor/2026-09-15T19-13-39Z
     Summary [226.425s] 5542 tests run: 5542 passed, 22 skipped   exit=0
     clippy 0

#152  scripts/floor.sh   .floor/2026-09-15T19-36-41Z
     Summary [228.990s] 5546 tests run: 5546 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

## EXPECTATIONS

| # | result |
|---|---|
| E1 | **PASS.** 27 `REPLAY(grok-rete #` commits, #126=`ef8ded525` … #152=`55969600a`, contiguous. Each has `(cherry picked from commit <C>)`. |
| E2 | **PASS.** The 14 docs-only steps (#127 #130 #132 #133 #135 #136 #138 #141 #145 #146 #147 #148 #150 #152) are cherry-pick -x of `docs/`/`.md` only. |
| E3 | **PASS.** Every produced `.wat` `--check` rc 0. convert.sh UNREADABLE era docs expected, not STOP. No UNREGISTERABLE `wat/`. |
| E4 | **PASS.** #126 re-expressed on `src/edn/render.rs` + `src/string/mod.rs`. No `src/edn_shim.rs` / `src/string_ops.rs` resurrected. Named test `an_unencodable_holon_raises_instead_of_panicking` green. |
| E5 | **PASS.** Shared-step named tests green (#126 diagnostic+row7; #139/#140 fixpoint_round_cap; #151 insert-door+fire-door). |
| E6 | **PASS** after one captured red. #139 green 5542/5542 clippy 0. #152 green 5546/5546 clippy 0. |
| E7 | **PASS.** Census+stone-3 gate on qualifying steps; lint subset + kind(lib) + doctests on `.rs` steps. No STOP-8/10/11 remaining. |
| E8 | **PASS.** No step touched `wat/`, `wat-scripts/fixes/`, or a file main deleted. #126's two moved homes re-expressed, not resurrected. |
| E9 | **PASS.** Composition repairs folded into #126 (flattening-helper rewrite; golden `:line 1526`). No repair commit after #152. |

## Re-expression that mattered

- **#126 trap door.** `value_to_edn_with` returns `Result<OwnedValue, RuntimeError>` on main's `src/edn/render.rs` (H-2 tagged maps kept). Lossy door for infallible sites. try-send encode hoisted in `src/kernel/message.rs`. `render_str_total` → lossy on `src/string/mod.rs`.
- **#126 composition 1.** Grok's new test `fn call -> Result<Value, String>` met main's `no_error_flattening_helper`. Rewritten to return `RuntimeError`. Captured lint FAIL not re-run.
- **#126 composition 2.** freeze.rs lossy-door comments moved `rust_caller_span!` 1525→1526. Golden recaptured. Folded into #126 (4b). Captured floor red `.floor/2026-09-15T19-06-20Z` not re-run.
- **#131.** HEAD diagnostic-snapshot comment kept (chase CLOSED by `assert_edn_eq!` rust-span normalize). Grok's item 10→9 was on the pre-normalize trail.
- **#151.** merge-file on `probe_arc278_session_memory_ceiling.wat`: grok rewrote the fixture (sm::step → fd::cross). Took converted C.

## Captured reds (not re-run)

1. lint `tests_carry_no_error_flattening_helper` at #126 first gate — 1 site `probe_edn_write_unencodable_is_a_diagnostic.rs:19 fn call -> Result<Value, String>`.
2. `.floor/2026-09-15T19-06-20Z` ARM verbatim: `select_prime_yields_lost_when_process_child_crashes` at `tests/process/probe_supervisor_select_lost.rs:202` — expected `:line 1525`, actual `1526`.

## STOP

None remaining. Do not push. Main untouched.
