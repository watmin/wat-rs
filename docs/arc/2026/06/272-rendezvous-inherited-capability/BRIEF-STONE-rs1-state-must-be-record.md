# BRIEF — Stone rs-1: defservice MINTS the state record (`:state [fields]` + `:record-parent`)

> Single-hop sonnet Shadowdancer. Do NOT spawn sub-agents. Do NOT use git worktrees. Work ONLY in
> `/home/watmin/work/holon/wat-rs`. FIRST run `pwd`; if not there, `cd`. Use `git -C <that path>` for git.
> Commit NOTHING — the orchestrator weighs + re-runs the gate. Grounded against HEAD `bff542f5`.
> Full design + reasoning: `DESIGN-STONE-record-state-final-return.md` § rs-1 (READ IT — esp. the
> reasoning chain + the emit-mint PIVOT). This SUPERSEDES the assert-record! approach (do NOT build a check).

## The work (one paragraph)
A service's `:state` must be a record, enforced BY CONSTRUCTION: defservice takes the state's FIELDS inline
and MINTS the state record `:<fqdn>::State` (so a non-record state is unexpressible — there's no slot for a
type keyword). An optional trailing `:record-parent` keyword selects the minted record's parent: omitted →
`:wat::Record` (base); `:wat::holon::Record` → a real holon record. Then migrate the 12 existing
`:state <type-keyword>` services to the new `:state [fields]` form (substrate-as-teacher cascade: the macro
change reds them; the failures are the worklist).

## Build — `wat/service.wat` (the defservice defmacro)
1. **Make defservice VARIADIC + change the `:state` slot to a field vector.** Current params (service.wat:52-57):
   `[fqdn _state-kw state-ty _ops-kw ops]`. New:
   ```
   [fqdn <- :wat::WatAST  _state-kw <- :wat::WatAST  state-fields <- :wat::WatAST
    _ops-kw <- :wat::WatAST  ops <- :wat::WatAST
    & opts <- :wat::core::Vector<wat::WatAST>]
   ```
   (`& rest` is the variadic syntax — see `wat/core.wat:254` `cond`, and the `+`/`-` arith macros. `opts` is
   the trailing tail: empty, or `[:record-parent <parent>]`.)
2. **Parse the optional `:record-parent`** in the `let` head (fenced macro-eval — `cond`/`if` + keyword
   equality are pure-total, allowed). `state-parent` = if `opts` empty → `:wat::Record`; else the SECOND
   element of `opts` (after the `:record-parent` marker). (Defensively: if `opts` non-empty and its first
   element is not `:record-parent`, `macro-error` with a clear message.)
3. **Mint `:<fqdn>::State` and REBIND `state-ty` to it.** Add a binder `state-ty` (keep the NAME — every
   downstream use of `~state-ty` then keeps working unchanged: serve param :406, `StopResponse[state <- ~state-ty]`
   :362, stop method `-> ~state-ty` :556/562, start params :650, self-peer `~state-ty` :623):
   `state-ty = (keyword/from-string (concat fqdn-str "::State"))`.
4. **Emit the State record def, branching on `state-parent`:**
   `state-record = (cond ((= state-parent :wat::holon::Record) \`(:wat::holon::Record::def ~state-ty ~state-fields))
                         (:else \`(:wat::Record::def ~state-ty ~state-fields)))`.
   Both macros exist (`wat/Record.wat`: `:wat::Record::def` → base; `:wat::holon::Record::def` → holon via
   `:wat::holon::Record::of` with `holon_form`).
5. **Splice `state-record` into BOTH emission sites** so the process/remote child also gets the type:
   - the final top-level `do` (service.wat ~597-606), and
   - `service-forms-def` (~555-564, where `~@request-records`/`~@response-records` ride to the child).
   Place it BEFORE the records/enums that may reference it (it's a type-decl; order among type-decls is fine).
6. **`:State` alias:** unchanged — defservice already DROPS the leading `s <- :State` triple from each op
   clause (service.wat:251, :129) and binds `s` to the serve `state` param; `:State` is never resolved as a
   type. No work needed there.

## Migrate the 12 services (cascade) — `:state <kw>` → `:state [fields]`
Pattern (mirror it): `:state :wat::core::i64` → `:state [count <- :wat::core::i64]`; in handlers,
`s` (the state) is now the `:<fqdn>::State` record — read with `(:<fqdn>::State/count s)`, build with
`(:<fqdn>::State <v>)`; call-site `state0` `0` → `(:<fqdn>::State 0)`. Files: `tests/probe_arc209_c1`/`c2`/
`c3`/`locus_agnostic_start`/`naming_conversion`, `tests/probe_arc265_acronym_registry`,
`tests/probe_arc272_6b_defservice_on_process`, `tests/probe_arc272_rs2_{thread,process}_stop_returns_final_state`,
`tests/probe_arc272_rs2_crash_surfaces_to_client`, `wat-tests/service-locus-parity.wat`. Keep each test's
INTENT + assertions (only the state SHAPE changes; e.g. rs-2 stop probes now extract `(:<fqdn>::State/count final)`).
The rs-2 stop probes already expect the final state — adjust to extract the count field from the State record.

## The gate probe — `tests/probe_arc272_rs1_state_must_be_record.rs` (already committed, RED at HEAD)
REMOVE all three `#[ignore]`s; all three must go GREEN:
`field_vector_state_mints_base_record_and_round_trips`, `record_parent_holon_mints_a_real_holon_record`,
`bare_type_keyword_state_is_rejected`. Do NOT weaken them.

## Rooms (read in order)
1. `DESIGN-STONE-record-state-final-return.md` § rs-1 (contract + reasoning + naming).
2. `wat/core.wat:254` (`cond` — the `& rest` variadic syntax) + the `+`/`-` arith macros (`& rest`).
3. `wat/Record.wat:8-110` (`:wat::Record::def`) + `:186+` (`:wat::holon::Record::def` — the holon mint path).
4. `wat/service.wat:52-112` (defmacro head + binders) + `:355-365`/`:540-565` (StopResponse + stop method) +
   `:555-606` (service-forms-def + final `do` — the two emit sites) + `:251`/`:129` (the `s <- :State` drop).
5. `src/macros/expand.rs:161-214` (how `rest_param` binds — confirms the `& opts` mechanism).
6. `tests/probe_arc272_rs1_state_must_be_record.rs` (the gate) + one migrated probe as the pattern.

## STOP triggers (halt + report — rejection criteria, not permission to ship less)
1. STOP if the fenced macro-eval engine rejects the `cond`/keyword-equality you need to branch on
   `state-parent` (report what it rejected — it should be pure-total/allowed).
2. STOP if `:wat::holon::Record::def` can't be emitted/spliced from defservice the way `:wat::Record::def`
   already is (report the divergence; the holon path is load-bearing for the builder's holon requirement).
3. STOP if minting `:<fqdn>::State` collides with any existing generated name, or if the State record must
   ride to the process child by a path other than `service-forms-def` (report).
4. STOP if a service migration would change what a test fundamentally PROVES (beyond the scalar→record state
   shape) — report that file; do not weaken a test.

## Gate (orchestrator re-runs)
- `cargo build --release -p wat` → clean.
- `cargo test --release -p wat --test probe_arc272_rs1_state_must_be_record -- --include-ignored --test-threads=1` → 3 GREEN (`#[ignore]`s removed).
- `cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum --test probe_arc209_c2_defservice_dispatch --test probe_arc209_c3_defservice_client_face --test probe_arc209_locus_agnostic_start --test probe_arc209_naming_conversion --test probe_arc265_acronym_registry --test probe_arc272_6b_defservice_on_process -- --test-threads=1` → all GREEN.
- `cargo test --release -p wat --test probe_arc272_rs2_thread_stop_returns_final_state --test probe_arc272_rs2_process_stop_returns_final_state --test probe_arc272_rs2_crash_surfaces_to_client -- --include-ignored --test-threads=1` → all GREEN.
- `cargo test --release -p wat --test test -- counter 2>&1 | grep "test result"` → locus-parity deftests GREEN.
- `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → 929/36 (zero new).
- `cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"` → ~893/4 baseline.

Report: exact files+lines changed; how `:record-parent` is parsed + how the def-macro branch is chosen; the
two splice sites for the State record; the per-file migration pattern; the pasted gate results from YOUR OWN
runs; any STOP hit. Do not commit.
