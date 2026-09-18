# ward `partire` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I have what I need. Verdict summary below.

---

# `partire` — vigilia cast, wat-rs @ `21530efab` (read-only)

Four files carry more than one reason to change. Every `file:line` below was opened this session.

---

## L1 — SPLIT · `src/freeze.rs` (2646 lines, ~5 domains)

The header states the file's job at `:1` — *"The freeze pass — step 11 of the startup pipeline"* — and then at `:30–32` states, under **"What freeze is NOT"**: *"It doesn't invoke `:user::main` — that's the wat binary's job."* `pub fn invoke_user_main` is at `src/freeze.rs:1388`, in this file. The file's own doc is the first witness against it.

**Module 1 — `process_runtime` (the stop protocol).** Moves: `src/freeze.rs:78–434` — the banner *"Runtime bootstrap — substrate-owned process startup"* (`:78`), `BootstrapArgs` (`:85`), `StopTarget` (`:115`), `ProcessRuntime` (`:122`), `ask_stop_and_collect_failures` (`:167`), `Drop` (`:232`), `bootstrap_wat_vm_process` (`:263`). **Reason to change:** the arc-170 stdio-as-defservice lifecycle and the STOP protocol — not what gets frozen. **Independent-test evidence, already realized:** `tests/kernel/bootstrap_wat_vm_process.rs:79` drives the whole region with `startup_bare()` + `BootstrapArgs` and no `:user::main`; `src/distribution/mod.rs:501` is a production consumer that calls bootstrap without going near `invoke_user_main`.

★ **Its other half is in `src/runtime.rs:195–750`** — *"Arc 170 Slice A — process-wide shutdown signal infrastructure"* (`:195`) through *"End arc 170 Slice A"* (`:750`): `SHUTDOWN_RX_PTR`/`SHUTDOWN_TX_PTR`/`SHUTDOWN_WAKE_WRITE_FD`/`SHUTDOWN_BROADCAST_READ_FD` (`:227`,`:266`,`:274`,`:287`), `init_shutdown_signal` (`:301`), `trigger_shutdown` (`:517`), the stop-failure publish slot (`:694`), `STDIO_BOOTSTRAPPED` (`:732`). Zero of it is evaluation. `src/runtime.rs:539–546` points at `ProcessRuntime::ask_stop_and_collect_failures` *in `src/freeze.rs`* as the place the ask now runs. One domain, two unrelated hosts, five external consumers reaching into `crate::runtime::` for it (`src/freeze.rs:272,1546,1568`, `src/io.rs:656,1048`, `src/comms/thread.rs:306`, `src/comms/process.rs:56`, `src/distribution/spawned_runtime.rs:50`). Independent test exists: `tests/comms/probe_arc278_send_poll_arm.rs:86–90` calls `wat::runtime::init_shutdown_signal()` and asserts the broadcast fd is armed — no world, no eval. **The minimum cut is one module (`src/kernel/shutdown.rs`, beside the existing `src/kernel/`) taking both regions.**

⚠ **The invariant that holds today only because the code is together:** `ask_stop_and_collect_failures` must run on the *same OS thread* that ran `bootstrap_wat_vm_process`, while the `ProcessRuntime` is still alive. Nothing in the type system says so — it is `ThreadOwnedCell` at runtime plus prose at `src/freeze.rs:147–155` and `src/runtime.rs:539–546`. Today construction (`:1459`) and ask (`:1544`) sit 85 lines apart inside one function in one file. After the split that becomes a cross-module comment. The cut must keep `ask_stop_and_collect_failures` at `pub(crate)` (it is, `:167`) and must not grow a second caller.

**Module 2 — `user_main`.** Moves: `src/freeze.rs:1350–1758` — invocation (`:1350` banner, `invoke_user_main` `:1388`, `invoke_user_main_orchestrated` `:1453`) and signature enforcement (`:1585` banner, `validate_user_main_signature` `:1623`, `format_type_expr` `:1701`). **Reason to change:** the `:user::main` calling convention (4-arg signature, ExitCode return, UselessMain). **Independent test:** `tests/program/wat_arc170_program_contracts.rs:5–8` is exactly this contract end-to-end. **Note the one back-edge:** `src/freeze.rs:949–950` — `startup_from_source` calls `validate_user_main_signature`. That is a use, not a shared secret (the comment at `:940–947` explains why the wall sits at that chokepoint), but it makes the seam bidirectional at module level; it is the one place a practitioner must decide whether the wall call moves with the validator.

**Module 3 — `deftest`.** Moves: `src/freeze.rs:1005–1191` — `is_deftest_fn` (`:1005`), `DeftestOutcome` (`:1032`), `expect_passed`, `call_beside` (`:1084`), `deftest_verdict` (`:1103`), `call_beside_value` (`:1163`). **Reason to change:** how a Rust gate reads a wat `deftest` verdict (the arc-278 vacuous-gate wall). It is *already* a cross-module contract: `src/test_runner.rs:614` defines `is_test_function` as a one-line delegation to `crate::freeze::is_deftest_fn` precisely so the two answers cannot drift (`src/test_runner.rs:610–613`). Its natural neighbour is `test_runner`, not the freeze pass. **Independent test:** any probe calling `call_beside(file!(), ":user::…")` — it needs a `.wat` fixture and nothing else from freeze.rs.

**Remainder stays `freeze`:** `FrozenWorld` (`:439`), `StartupError` + its ten `From` impls (`:680–922`), `startup_from_*` (`:924–1004`, `:1192–1349`), and the constrained-eval block.

**Refused cut — "Constrained eval" (`src/freeze.rs:1759–1974`).** It has its own banner and looks available. Withdrawn: `refuse_mutation_forms` (`:1882`) / `is_mutation_form` (`:1907`) enumerate *the set of forms freeze makes illegal*. That is the same secret `FrozenWorld` hides, restated for the runtime door. Splitting it puts the freeze contract in two files that must change together — the `encode.rs`/`decode.rs` mistake. Leave it in.

**Severity: Level 1.** A maintainer changing the entry-point signature currently opens a file that also holds the shutdown handshake, the test-harness API, and the eval denylist.

---

## L1 — SPLIT · `src/string_ops.rs` (1254 lines, 5 domains, one verb in the wrong crate module)

Header at `:1–2`: *"`:wat::core::string::*` + `:wat::core::regex::*` + `:wat::core::Uuid/*`"* — the file names three of its domains in its first line, and holds two more it does not name. Every caller is `runtime.rs` dispatch plus three sites in `src/types.rs`, so this is a leaf: no inbound coupling to unpick.

**Module 1 — `string_ops` (keep the name).** Stays: `:30–125` and `:363–791` (the plain verbs), helpers `one_string` (`:1187`) / `two_strings` (`:1215`), `render_str_total` (`:543`). **Reason to change:** the `:wat::core::string::*` surface and its char-oriented UTF-8 semantics.

**Module 2 — `name_case`.** Moves: `src/string_ops.rs:126–362` — `eval_string_pascal_to_kebab` (`:130`), `keyword_value_to_registry_key` (`:158`), `eval_string_pascal_to_kebab_in` (`:197`), `pascal_to_kebab_with_acronyms` (`:233`), `eval_string_kebab_to_pascal_in` (`:300`), `kebab_to_pascal_with_acronyms` (`:336`). **Reason to change, distinct:** the naming doctrine — it reads `sym.acronym_registry` (`:222`) and answers "how is a wat *name* spelled across cases", not "what does a String verb do". Its consumers are the `defservice` macro at expand time (`:126–128`) and `src/types.rs` (three call sites), not string users. **Independent test:** a fixture that registers an acronym set for a namespace and asserts `"CreateWebACL"` → `"create-web-acl"` — needs a SymbolTable with an acronym registry and none of the string verbs.

**Module 3 — `uuid`** (`src/intrinsic/uuid.rs` — `src/intrinsic/` is the established home for per-type intrinsics). Moves: `src/string_ops.rs:792–1058` — banner *"typed uuid (arc 207 slice 2)"* (`:792`), `is_canonical_uuid_string` (`:805`), `eval_uuid_typed_v4` (`:824`), `v5` (`:847`), `from_string` (`:895`), `to_string` (`:935`), `version` (`:971`), `rfc4122_variant` (`:1007`), `nil` (`:1042`). **Reason to change:** RFC-4122 canonical form and the `uuid` crate. **Independent-test evidence, already realized:** `tests/types/uuid.rs` plus nine `tests/types/uuid_*.wat` fixtures — none of which touches a string verb. Verified structurally: not one of these nine functions calls `one_string`/`two_strings` (grep of every call site, `:36–1177`); they hand-roll their own arity/type checks. The seam is clean in both directions.

**Relocation (not a new module) — `:wat::core::List/of`.** `src/string_ops.rs:1148–1160` is a `LinkedList` constructor sitting in the string module. Every other `List/*` verb body lives in `src/collection/eval.rs` (`:43`, `:165`, `:251`, `:389`, `:478`, `:568`). Move it there. There is no reason-to-change under which `List/of` and `string::trim` move together.

**Practitioner's call — `char/of` (`:1060–1146`) and `regex::matches?` (`:1162–1184`).** Both have distinct reasons to change (BMP-only scalar policy; the `regex` crate), and the header at `:13–16` argues the regex case itself — *"a wat-rs deployment that didn't want the regex dep could feature-gate this module separately."* But each is a single verb, and `regex` uses `two_strings`. Cutting one-verb modules is over-shredding unless the feature gate is actually wanted. **Named, not decided.**

**Severity: Level 1.** Five reasons to change; the misplacement is demonstrable against the tree's own placement rule for `List/*`.

---

## L1 — SPLIT · `src/load.rs` (1894 lines, 2 domains, one of them a security boundary)

**Module 1 — `load` (keep the name).** Stays: `:1–992` — the four `load!` forms' grammar (`match_load_form` `:653` through `variant_name` `:989`), the resolution pipeline (`resolve_loads` `:451`, `process_single_load` `:489`), verification dispatch, `LoadError` (`:255`). **Reason to change:** the load-form grammar and verification semantics.

**Module 2 — `source_loader`.** Moves: `src/load.rs:171–190` (`pub trait SourceLoader`, `:171`), `:191–254` (`LoadFetchError`, `:191` — the loaders' error type, incl. `OutOfScope`), and the whole `// ─── Loaders ───` block `:993–1279` — `InMemoryLoader` (`:998`), `span_display_path` (`:1081`), `FsLoader` (`:1100`), `ScopedLoader` (`:1157`), `resolve_within_scope` (`:1198`), the containment refusal (`:1229`), `resolve_relative` (`:1268`). **Reason to change:** how a path resolves to bytes — canonicalization, `../` traversal, symlink escape, scope containment. **The seam is one-directional and I checked it, not assumed it:** the region `:993–1279` contains **zero** references to `LoadError`, `LoadSpec`, or `WatAST`. Grammar → loaders, never back.

**Independent-test evidence, already realized:** `src/load.rs:1787–1898` — seven `scoped_loader_*` unit tests (`:1787`, `:1803`, `:1814`, `:1826`, `:1840`, `:1862`, `:1878`, `:1889`) construct a `ScopedLoader` over a temp dir and assert absolute-path escape, `../` escape, and symlink escape all return `OutOfScope`. No `load!` form, no parser, no `LoadSpec`. That is the fixture that reaches this module without the other.

⚠ **The invariant that holds today only because the code is together — and this is the `hash_join` lesson repeating.** `LoadedSource::canonical_path` (`:163`) is the *identity* on which cycle detection (`:500`) and commit-once (`:513`, `:520`) key. Canonicalization is performed by the loader (`FsLoader`/`ScopedLoader` call `std::fs::canonicalize`), while `InMemoryLoader` returns the raw map key (`:1043`) and `synthetic_string_path` fabricates one (`:565`). The contract *"two spellings of one file yield one `canonical_path`"* is per-loader convention, documented in prose at `:1075–1080`, enforced nowhere. Today both ends sit in one file. Split it, and a future loader that forgets to canonicalize silently disables cycle detection — a correctness failure in the module that no longer contains the detector. **Any execution of this cut must land a boundary test on `LoadedSource::canonical_path`, not just move the code.**

**Severity: Level 1**, and the "actively misleads" is measurable: see L2-a.

---

## L2 — findings

**a) `tests/kernel/wat_run_sandboxed.rs:144–165` — two tests named for scope containment that cannot observe scope containment.** `scoped_file_eval_inside_scope_dies_on_terminal_eprintln` (`:145`) and `scoped_file_eval_outside_scope_dies_on_terminal_eprintln` (`:158`) both assert the *empty-loader* `Err` arm. The file's own comment says so (`:151–154`): *"the original test asserted stdout='ok' under a ScopedLoader; canonical spawn-program' :process hardcodes an empty InMemoryLoader for the child … The ScopedLoader CONTAINMENT surface needs separate coverage."* Confirmed at `src/process/verbs.rs:403` — `let loader = Arc::new(InMemoryLoader::new());`. So the "outside scope" test's claim at `:161` — *"a stronger no-leak proof than the old targeted-absence check"* — is false: the read fails because the loader is empty, not because the path is out of scope. Containment *is* covered, but only by the in-file unit tests at `src/load.rs:1826–1877`, i.e. in the last place someone auditing the sandbox would look. **Remedy:** rename the two tests to what they now prove (empty-child-loader Err routing), and let the `source_loader` cut carry the containment tests into a file whose name says "containment". `[[a-negative-fixture-can-fail-for-the-wrong-reason]]`.

**b) SPLIT · `src/io.rs` (1991 lines) — byte-transport backends vs. verb bodies, plus one stray verb.**
- **`io::backend`** ← `:36–780`: the traits (`:36`), `RealStdin`/`RealStdout`/`RealStderr` (`:113`), `StringIo*` (`:281`), `PipeReader`/`PipeWriter` (`:438`). **Reason to change:** fd ownership, Drop/close semantics, buffering. **Independent test already realized:** `tests/types/uuid.rs:20–33` builds `PipeReader`/`PipeWriter` directly and never calls an `eval_ioreader_*`. Direction verified: the region `:36–780` contains no call into the primitive half (one doc mention at `:191`); the primitive half references backend types 24 times.
- **`io`** ← `:781–1836`: the `:wat::io::` verb bodies. I deliberately do **not** cut reader/writer from fs here: `src/intrinsic/io/mod.rs:4–17` rules that `:wat::io::` *is* one family — *"ONE subject — bytes crossing the process boundary — asked three ways"* — so those three are one reason to change. Minimum cuts respects that ruling.
- **Stray, move it:** `eval_stdlib_sources` (`src/io.rs:1837`) — its own doc says *"Zero args; pure (no I/O)"*, it calls `crate::stdlib::stdlib_files()`, and `src/intrinsic/io/fs.rs:11–16` states outright that `:wat::stdlib::sources` *"belongs to a different family (arc 275 Stone 275.1's baked stdlib load order) … it was never part of this family."* Home: `src/stdlib.rs`.
- ⚠ **Invariant at risk:** `eval_ioreader_read_frame` (`:1003`) does raw `libc::poll` on `reader.as_raw_fd_for_poll()` (`:1046`) against `SHUTDOWN_BROADCAST_READ_FD`, and its correctness depends on *which impls return `Some`* — the comment at `:1043–1047` reasons about `RealStdin` reporting `Some(0)` and can say *"this module"* only because they share a file. The shutdown lockstep is a secret spanning the proposed seam; the trait method (`:57–64`) is its declared contract, and after the cut that contract needs a test, not a comment. **Level 2** — the entanglement is real but rarely bites.

---

## L3 — judgement

- **`src/rete/validate/mod.rs` — LEAVE, and my prior cuts landed.** `:40` and `:44` credit two earlier `partire` cuts (`typing.rs`, `error.rs`), both executed. What remains — `:when` validation, the `:not` bind wall, `:then` validation *and* the kwargs reorder — is one decision: the arc-294 9a defrule freeze-time wall (`:1–17`). The reorder is not a second concern; the wall's stated purpose (`:14–16`) is *validate and REORDER*, because the corruption class it closes was positional-kwargs drift. Splitting "validate" from "reorder" would cut through the middle of one secret. No sub-region has an independent test surface — a wall test feeds a `make-rule` form and asserts a located `#wat.rete/*` error or a reordered residue.
- **`src/rete/export.rs` — likely LEAVE, not certified.** It is a pack/unpack codec (`:6` "The three laws of this codec"), `pack_*`/`unpack_*` pairs from `:630` to `:1652`. Cutting `pack` from `unpack` is the archetypal accidental seam — they share the wire shape. I skimmed the item list only; I did not read the bodies, so this is a judgement, not a verdict.
- **`src/runtime.rs` (40,883 lines) — the tree's largest braid, and re-proposing "split it" would add nothing.** 50 section banners spanning shutdown infra, `defclause` parse+eval, scalar conversions, holon holograms, timers, peer verbs, config accessors. But arc 255's `#[wat_intrinsic]` registry (`src/intrinsic/mod.rs:1–5`, `src/intrinsic/io/mod.rs:18–21`) is the declared, in-flight decomposition mechanism for exactly this, and it is closing families one at a time. The one region that mechanism will *never* reach, because it is not a verb, is the shutdown block at `:195–750` — which is why it is folded into L1 above instead of standing alone.

---

## What I could not check, and why

- **I ran nothing.** Read-only order: no build, no floor, no clippy. So I have not proven any proposed module compiles after the move, that `pub(crate)` visibilities survive (`ask_stop_and_collect_failures`, `render_str_total`, `kebab_to_pascal_with_acronyms` all cross the proposed boundaries), or that `#[cfg(test)]` blocks travel with their subjects. Every "independent-test evidence" claim above is *that a test exists and reaches the region through the named door* — read, not executed.
- **I did not survey `wat/` at all.** It is in the stated scope and I spent the cast on `src/`. `wat/service.wat` (2952), `wat/core.wat` (2152), `wat/fix.wat` (1200), `wat/rete/compile.wat` (1163), `wat/bracket.wat` (1026), `wat/gen.wat` (992) are unassessed. `service.wat` at ~3k lines is the largest single file in the corpus and nobody has cast on it this round as far as I can tell. That is the biggest hole in this report.
- **I did not re-derive my own two open rows from 2026-08-30.** `src/rete/compiled_cond.rs` still carries `// ─── The compiler ───` (`:280`) and `// ─── The executor ───` (`:869`), and `src/rete/expr_ir/eval.rs` still holds `apply_core_kind` (`:937`) — so neither of those cuts has landed. I confirmed the *banners* are still there; I did **not** re-read the bodies this session, so I am not restating those seams as verified. Treat them as open and unconfirmed, not as findings.
- **`src/check.rs` (22,509), `src/closure_extract.rs` (3191), `src/rete/purity.rs` (2633), `src/comms/process.rs` (2149), `src/collection/eval.rs` (2384), `src/macros/expand.rs` (1971)** — headers/banner lists only. `purity.rs` in particular looks like a defensible LEAVE (four classifiers over one shared walk) but I read its module doc, not its walk.
- **The `src/intrinsic/kernel/` tier.** `src/intrinsic/io/mod.rs:4–6` quotes that tier's own `mod.rs` as saying *"`:wat::kernel::` is not a family. It is a TIER — nine homes braiding independent concerns."* I did not open it. If that sentence is accurate the tier is already decomposed; if it is a claim nobody rechecked, it is a `partire` target and I have not touched it.
