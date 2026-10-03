# BRIEF — shape strike B3: the last interim shapes, a fabricated location, and the serve loop's panics

Excursus 003. This strike handles the remaining items on the Strike B worklist in
`AUDIT-the-shape-of-an-error.md`: B1's findings 3–5, T3's two notes, and one new finding. It builds on
`a7749e5a1`.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

**Land one item per commit, each on a green floor**, in the order below. No item can be half-built.

## Items

1. **`MacroExpansionFailed.cause` holds the real `MacroError`.**
   - Today it holds an interim `Fault` (`single_cause_fault`, `src/value/runtime_records.rs:~366`).
     B1 said a typed cause "needs a `TypeEnv` that `to_record` lacks on the peer-death paths".
   - Measure two candidate fixes, and choose the one with no second copy of the shape:
     - **(a)** decode `MacroError`'s `error_edn()` against the process-wide builtins registry. A
       `OnceLock<TypeEnv>` exists at `src/runtime.rs:~10731`; confirm what it holds. Every error
       record is a builtin since the sweep.
     - **(b)** build the `MacroError` value directly, the way `hash_error_value` does for B2's
       `HashError`.
   - Then retire `single_cause_fault` if nothing else uses it.
2. **`read-json` / `read-foreign` failures carry structure.** The failure is flattened at the
   source:
   - `read_json_outcome_malformed(&e.to_string(), …)` at `src/edn/render.rs:~298,~300,~348`;
   - `format!("EDN parse error: {e}")` at `:~354`.

   Then `tagged_read_outcome_malformed` builds a `Fault` from prose, under a synthetic tag that is
   never declared.
   - Declare the JSON and foreign read errors as records (the sweep pattern: mirror the wire, Rust
     derived from `.wat`), or route them through the `:wat::edn::*` read-error records T2 declared,
     if they are the same kinds.
   - `:Malformed`'s `cause` then holds the record.
   - Measure which error types reach these sites first.
3. **No fabricated location.** `fault_value(message, None)` synthesizes a `<runtime>` file at line 0
   (`src/runtime.rs:~11963`). It has one `None` caller, at `:~12196`.
   - That caller supplies a real span: the Rust raise site via `rust_caller_span!` at worst, or the
     wat span it has in hand.
   - `fault_value`'s parameter becomes a mandatory `Span`, so a fabricated location cannot be
     written.
4. **Goldens stop churning on Rust edits.** `assert_edn_matches_file!` compares after
   `normalize_rust_source_span_lines` (every `.rs` span's `:line` becomes 0), but the `UPDATE_EDN`
   path writes the RAW value (`src/lib.rs:~431`). So a recapture writes whatever line the build
   produced, and goldens churn for no semantic reason (T3's `0 → 93`).
   - Normalize before writing.
   - Recapture once. Only `.rs` `:line` values may move, toward 0. Read every diff.
5. **The serve loop's panics become values.**
   - **(a)** T3's fallback arm in the `ServiceEvent.RequestMalformed` dispatch,
     `[_ (assertion-failed! …)]`. It is unreachable by construction (the op name comes from the
     registered enum), but it is still a panic. Make it a value: for example, reply the generic
     `Reply::Failed` with a `Fault` naming the impossible op, and keep serving.
   - **(b) New finding.** The admin dispatch panics on protocol errors:
     `(:wat::kernel::assertion-failed! :message "defservice dispatch-admin: Stop received before Init/Resume …")`,
     and the same for `Hibernate` (`wat/service.wat:~1240,~1242`).
     - **Measure** whether a client (or a peer that is not the owner) can send an admin message.
     - **If one can,** a client can crash the service, which is a denial of service: STOP and report
       it before changing anything. That is the builder's call.
     - **If only the owner can,** make the refusals values, in the same move as (a).
   - List any other `assertion-failed!` in service codegen with the same reachability question.

## Gates (each mutation-proven in RELEASE, one per item)

1. `MacroExpansionFailed.cause` decodes typed as `:wat::macro::<Kind>`. Mutation: restore the
   interim `Fault`. RED.
2. A malformed JSON/foreign read's `:Malformed` cause is the declared record. Mutation: restore
   `to_string()`. RED.
3. No `<runtime>` file appears in any golden, and `fault_value` cannot take `None` (show the compile
   error).
4. Recapturing twice is byte-identical, even after an unrelated Rust edit that shifts a raise site's
   line. Mutation: write raw again. RED.
5. The unreachable-op arm, and any owner-only admin protocol errors, reply a value and keep serving.
   Mutation: restore the `assertion-failed!`. RED.

## Scope fence

- **IN:** items 1–5.
- **OUT:**
  - the builder's calls on `char` declarability and `LoadFetchError`'s tag rename;
  - a raw-socket test door;
  - C (`provenance`), D (`Frame`), E (`EvalError`), F (the domain `Fault`s);
  - the stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build, recapture or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- Iterate with targeted runs and `cargo wat` dry-runs. Run the full floor only at each commit.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - **Never commit a red floor.** "Unrelated to my change" is not a disposition.
- Stage new files BEFORE the floor. Never use `git add -A`. No `cargo fmt`.
- Multi-site structural `.wat` rewrites go through the wat-fix codemod. Authoring new declarations
  in one file may be done by hand: name it.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings` after each item.
- If your budget runs short, stop after the last fully landed item and report where.

## Report

- per item: the decision, each gate's mutation RED, the floor `Summary` line verbatim, and the SHA;
- item 5(b)'s reachability answer;
- any golden that changed beyond the stated shapes.
