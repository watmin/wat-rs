# ward `purgare` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

## `purgare` — wat-rs @ `21530efab` (`grok-rete`)

Every claim below was grepped this session across `src/`, `tests/`, `crates/`, `benches/`, `examples/`, `tools/` **and** the `.wat` corpus (`wat/`, `wat-scripts/`, `wat-tests/`, `tests/**/*.wat`). Sweeps run: all 234 `pub struct/enum` names, all 591 `pub fn` names (call-position regex, not bare-name count), all 853 enum variants, 350 `const`/`static`, 912 struct fields, 160 collection-typed fields, all 586 `wat/` top-level def-form names, and every `wat/*.wat` file against `include_str!`.

---

### L1

**1. `src/process/handle.rs:34` — `ChildHandle` is never constructed. The whole file is dead, and two live doc comments claim it is the substrate's zombie reaper.**

- `ChildHandle::new` (`handle.rs:61`) has **zero call sites**. `Value::wat__kernel__ChildHandle` (`src/value/value.rs:171`) is **never constructed** — 40 whole-repo hits for the identifier, every one a match arm, a type-name string, or prose. Dead members: `new:61`, `child_pid:74`, `mark_reaped:83`, `wait_or_cached_exit:98`, `impl Drop:128`, plus the `#[allow(dead_code)]` `lifeline_w:52`.
- **Reachability evidence including the `.wat` grep:** `grep -rn "ChildHandle" wat wat-scripts wat-tests --include='*.wat'` returns exactly two hits, both `;;` comment prose (`wat-scripts/scratch-pad/probe-arc278-fnforms-reaches-program-types.wat:20`, `…-57-persistentmap-contains-key.wat:32`). No `.wat` code position names `:wat::kernel::ChildHandle`, and no `#[wat_dispatch]`/primitive registration exists for any of these methods (unlike `src/rust_deps/sqlite.rs`, where `begin`/`pragma`/`execute_ddl`/`open_readonly` show the same zero-Rust-caller shape but are live via `#[wat_dispatch(path = ":rust::sqlite::Connection")]` and called from `wat/sqlite.wat:106,124,132,134`).
- **Mechanism:** the OS-spawn path now returns `Process { input, output, pidfd }` (`src/kernel/spawn.rs:1066-1071`); nothing builds a `ChildHandle`.
- **Consequence — this is why it is L1, not just clutter.** `handle.rs:19-22` states *"`Drop` sends `SIGKILL` and blocks on `wait_status` if the caller never called `wait_or_cached_exit` — keeps zombies out of the process table."* That guarantee is inoperative: the Drop impl can never run. `src/runtime.rs:32387` repeats it — *"`ChildHandle::Drop`/`close'` remain the only paths that reap"* — and is contradicted **ten lines below in the same comment block**, `src/runtime.rs:32397`: *"nothing in this substrate reaps a `Process` peer's pidfd except `close'`"*. A reader auditing reap coverage is told there are two paths when there is one.
- **Fix:** delete `src/process/handle.rs`, `src/process/mod.rs:63`, the `wat__kernel__ChildHandle` variant, and correct `runtime.rs:32387` to name `close'` alone. **Cascade, ~12 sites:** `value.rs:171,592,692,907,1220,1471-1477,1751`, `runtime.rs:9346,12544,18522,32387`, `check.rs:13782,13895`, `closure_extract.rs:2255`, `edn_shim.rs:4053`, `value/observe.rs:420`, `process/clone.rs:244,299,300`, `clippy.toml:17`. Note `value.rs:1474-1478` registers a `KeyEligibility::NeverAKey` gate keyed on `TypeExpr::Path(":wat::kernel::ChildHandle")` — a map-key rule for a type no value can inhabit.

**2. `wat/bracket.wat:289` — `:wat::bracket::dotpath->colonpath` is a stdlib `defn` with zero callers anywhere.**

- **Reachability evidence including the `.wat` grep:** `grep -rn "dotpath" . --exclude-dir=target --exclude-dir=.git` returns exactly **two** lines, both in `wat/bracket.wat`: the comment header at `:275` and the `defn` at `:289`. No `.wat` file calls it; no `src/` site names the string (so it is not resolved dynamically from Rust either). `wat/bracket.wat` *is* baked in (`src/stdlib.rs:188`), so it type-checks — and per this repo's own CLAUDE.md warning, type-checking an unforced `def` body is not resolution, which is exactly why nothing went red.
- Its 15-line header (`:275-288`) asserts it exists to re-punctuate `field-types-of`'s output for the angle-bracket keyword strings "built below" — but the code below (`:295+`) does the `Peer`→`Address` head-swap with inline `split`/`join` instead. The comment vouches for a consumer that was never wired.
- **Fix:** delete the `defn` and its header. Leaf deletion, 1 site.

---

### L2

**3. `src/value/value.rs:438` — `ClauseSet.shared_return` is write-only, and its doc claims a reader.**

Whole-repo grep for `shared_return`: constructed at `src/runtime.rs:8322` (production) and `src/check/env.rs:498` (a `#[cfg(test)]` fixture); **never read** — no `.shared_return` access, no `ClauseSet { shared_return, .. }` destructure (only 3 `ClauseSet {` sites exist: `value.rs:434` decl, `runtime.rs:8319`, `check/env.rs:495`). `value.rs:431` says *"`shared_return` is retained only for diagnostics"* — no diagnostic reads it. **Fix:** delete the field (Option A sugar is already resolved into per-clause returns at `runtime.rs:7949`), or wire the diagnostic. Cascade: 3 sites.

**4. `src/io.rs:398,405` — `StringIoWriter::snapshot_bytes` / `snapshot_string` are a dead inherent pair superseded by the trait method, and the struct doc points at the dead one.**

`snapshot_string:405` has zero callers (whole-repo `grep -rIF snapshot_string` = 1 hit, the definition). `snapshot_bytes:398`'s only caller is `snapshot_string` at `:406`. The live snapshot path is the `WatWriter::snapshot()` override at `io.rs:431`, reached from `snapshot_writer` (`io.rs:1387-1406`) which serves `IOWriter/to-bytes` and `IOWriter/to-string`. `io.rs:377` tells readers *"Readable via [`StringIoWriter::snapshot_bytes`] — intended for test…"*, aiming them at the corpse. **Fix:** delete both, repoint the doc at `WatWriter::snapshot`. Cascade: 3 sites.

**5. `src/check/env.rs:457` — `get_extend_methods` has zero callers, which makes the `Vec<String>` half of `extend_registrations` write-only.**

`extend_registrations` (`check/env.rs:110`) is written at `:189` and `:450`; the map is otherwise only key-probed. `get_extend_methods` — the sole reader of the values — is called from nowhere (whole-repo grep: 3 hits, all in `check/env.rs`, none a call). `check/env.rs:459-461` says it plainly: *"The KEY's existence is the satisfaction signal."* The `Vec<String>` payload is carried, cloned, and stored for no consumer. **Fix:** either delete `get_extend_methods` and narrow the map to `HashSet<(String,String)>`, or rune it. Cascade: 4 sites.

**6. `src/runtime.rs:8745` — `is_defclause_form` has zero callers, and the predicate it encodes is duplicated inline where it is actually needed.**

Whole-repo grep: 3 hits, none a call. The same match is written inline at `src/check.rs:8515` (`Some(WatAST::Keyword(k, _)) if k.as_str() == ":wat::core::defclause"`). **Fix:** delete, or route `check.rs:8515` through it. Leaf deletion.

**7. `src/runtime.rs:144` — `reset_user_signals` is a `#[cfg(test)]` helper whose last caller was deliberately removed, and deletion never followed.**

Zero callers. Its three siblings (`set_kernel_sigusr1:127`, `_sigusr2:132`, `_sighup:137`) are all live from `src/compose.rs:58,61,64` and `src/process/child.rs:42,46,50` — this one is the odd one out. It escaped `dead_code` only because it is `pub` in the lib crate. **Fix:** delete it, or `// rune:purgare(future-fixture)`. Leaf deletion.

**8. `src/check.rs:321` — `CheckResult::merge_errors_from<U>` has zero call sites.**

Whole-repo grep: 15 hits, 14 in `docs/`, 1 the definition. Its sibling combinator `drain_errors_into` (`check.rs:~340`) is the one that took over. **Fix:** delete or `rune:purgare(public-api)`. Leaf deletion.

**9. `src/value/symbol_table.rs:342` — `remove_def_value` has zero call sites.**

Whole-repo `grep -rIF remove_def_value` = 1 hit, the definition. Its sibling `remove_function:330` is live (`src/runtime.rs:894`), so this is not a symmetric-API pair that a rune already covers. **Fix:** delete or rune.

**10. `wat/telemetry.wat:329` — `:wat::telemetry::LOG-MSG-CAPACITY` is computed at every freeze and read by nothing.**

Whole-repo grep for the name: the `def` at `:329`, plus one `;;` comment mention in `tests/types/probe_arc278_capacity_derive.wat:9`. No code position reads it. Because top-level `def` RHS is evaluated eagerly at freeze, this runs `framing-floor-of` over `:wat::telemetry::Log` on every stdlib freeze for nothing. Its own header (`:326-327`) admits the reader does not exist: *"the exact per-caller gate is the runtime remainder … (§3, deferred wiring)."* `LOG-JOURNAL-BUDGET-BYTES:328` is alive only through this dead value. **Fix:** wire §3, or delete both. Cascade: 2 sites.

**11. `crates/wat-edn/src/json.rs:91` — `JsonError::InvalidMap(String)` is never constructed.**

Whole-tree grep for `InvalidMap`: 1 hit, the declaration. Its neighbour `InvalidSet:94` *is* constructed (`json.rs:418`), so this is not a symmetric pair. Coverage is `thiserror`'s derived `Display`, not a hand-written exhaustive match, so the spell's variant exemption does not apply — nothing is kept honest by its presence. `JsonError` is public API of a workspace crate, so it also widens a published surface with an unreachable case. **Fix:** delete, or rune `public-api`.

**12. `src/process/handle.rs:4` and `src/process/mod.rs:26` — both module docs name `ForkedProgramHandles`, a type that does not exist anywhere in the tree.**

`grep -rIn "ForkedProgramHandles" src tests crates --include='*.rs'` returns exactly 3 hits: these two doc lines and `src/distribution/mod.rs:378` (also prose). No declaration. `handle.rs:4` states *"`ForkedProgramHandles` — bundle returned by the OS-process spawn paths"*; the real bundle is `ProcessPeerBundle` (`src/kernel/spawn.rs:1078`). These are comments *in code files*, not `docs/`. **Fix:** correct both lines to `ProcessPeerBundle` — or they vanish with finding 1.

---

### L3

**13. Two error variants that no code path can produce.** `src/harness.rs:72` `HarnessError::StdioSnapshot(String)` and `src/wat_edn_bridge.rs:309` `WatEdnBridgeError::KeywordDecode { raw }` are each referenced exactly twice: the declaration and one hand-written `Display` arm (`harness.rs:81`, `wat_edn_bridge.rs:338`). Never constructed. The spell exempts variants held honest by exhaustive matches, and a manual `Display` arm is one — so this is taste, not defect. But `StdioSnapshot` is the third member of the same retired stdio-snapshot cluster as finding 4; if that goes, this goes with it.

---

### What I could not check, and why

- **I did not build, run the floor, or execute anything.** Every deadness claim rests on grep over source, not on `cargo check` after deletion. The spell's "Simple?" test — *delete it and see if the build still passes* — is unexecuted for all 13. Findings 1, 2, 4, 6, 7, 8, 9, 11 are leaf/small-cascade and I am confident; **finding 1's 12-site cascade is the one most likely to surface a compile edge I cannot see** (the `KeyEligibility` table at `value.rs:1471` is macro-generated and I did not read the macro).
- **My field sweep cannot see destructuring.** I extracted 912 struct fields and tested only `.field` access, which produced ~14 false positives I hand-cleared (`RoundScratch`'s `packed_full`/`seen_ids`/`seen_rest`/`leaf_aids`, `ClauseCtx.rule_name`, `AssertionPayload.raised_error`, `MainInput`/`TestInput.deps`/`loader`, `clone_args`'s syscall fields, `ExecPlan._argv`/`_envp`). Fields read **only** via destructuring in a file I did not open would read as alive to my sweep and I would have missed them — so the write-only-state category is **under**-reported, not over-reported. `ClauseSet.shared_return` (finding 3) survived because I checked all 3 `ClauseSet {` sites by hand.
- **The `wat/` sweep counts substrings.** `grep -F` means a short name that is a prefix of a longer one inflates its count and reads as alive. So dead `.wat` defns whose names are prefixes of live ones are invisible to me. Only two candidates surfaced from 586 names, which is suspiciously clean; I would not claim `wat/` is otherwise dead-free.
- **I checked reachability only for the `:wat::rete::` and `:rust::sqlite::` families against the `.wat` corpus by hand.** For every other `src/` function I relied on Rust call sites plus a whole-repo string grep. A Rust fn registered as a wat primitive under a name that does not textually contain its Rust identifier would be invisible to that method. I found no such registration form, but I did not enumerate every `#[wat_*]` attribute macro to prove none exists.
- **Untaken branches are almost entirely unexamined.** I sampled the bool-parameter surface (44 params) and verified variance at call sites for `exec_ops`, `harvest_class_scan_filter`, `register_extend_type_surface_impls`, `image_source` — all genuinely two-valued. I did **not** attempt the general "this `if` always goes one way" analysis; that needs execution, not grep, and it is the spell's category I have the weakest coverage on.
- **`tests/` I treated as a consumer, not a target.** Dead helpers *inside* `tests/` are caught by `dead_code` (separate crates), so I did not sweep them — except where a `tests/` file was the sole reference keeping a `src/` item alive, which I flagged inline.
