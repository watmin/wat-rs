# BRIEF — kwargs-`start`/`resume`: defservice lifecycle goes all-kwargs (Form A) — "clojure expressivity, PROVEN"

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents. Do NOT use git worktrees. Do NOT commit.** If you hit a STOP trigger or the work exceeds the brief,
STOP and report — do not improvise.

Build/test: `cargo build --release -p wat`, `cargo test --release -p wat …`. After editing any `wat/*.wat`
**`touch tests/test.rs`** (wat-tests re-scan on `.rs` recompile). TRUST FORCED CLEAN BUILDS (`cargo clean -p wat
&& cargo build --release -p wat`) if results look stale.

## The mission, in one line

`defservice` emits its `start`/`resume` lifecycle fns as **positional** `defn`s (`service.wat`). Flip them to
**`& [argspec]` kwargs fns (Form A — ALL args named)** so a caller writes
`(worker/start :locus L :record R :recorder-addr A)` — order-independent, every arg strongly typed and
compile-checked. They inherit the 260.1b call-sugar automatically (the `defn` kwargs branch mints the companion
macro). This rides the just-landed **macros^unbounded** hoist (defservice is a macro emitting a kwargs `defn`
which emits its companion `defmacro` — registered at depth) and is guarded by the just-landed **hygiene Dredd
gate** (`HygieneScopeDivergence`). When green, **kwargs-`start` is the proof of "clojure expressivity in a
strongly typed language" on the real lifecycle API** — the thing arc 291 set out to demonstrate.

## Decision already made (do NOT re-litigate): Form A — ALL args named

`start` becomes `[& [locus <- :wat::spawn::Locus  <…init params…>]]` (no leading positional). Same for `resume`.
Four-questioned + settled; the call site is a clean keyword column.

## The macro change — `wat/service.wat` (re-ground line numbers before editing; service.wat was NOT touched by the hygiene strike, so HEAD anchors below should hold)

1. **`start-params` (`~1048`)** — today `` `[locus <- :wat::spawn::Locus  ~@init-param] ``. Change to
   `` `[& [locus <- :wat::spawn::Locus  ~@init-param]] `` (wrap the binders in the `& [...]` kwargs section).
2. **`resume-params` (`~1071`)** — identical flip: `` `[& [locus <- :wat::spawn::Locus  ~@init-param]] ``.
3. **The synth default-`:init` param `d`→`record` (`~187`)** — today `d-sym (:wat::core::symbol-node "d")`,
   used in the default `:init` `(fn [~d-sym <- ~record-ty] -> ~state-ty (~state-new-kw ~d-sym))`. Rename the
   symbol-node from `"d"` to `"record"`. This makes a **default**-init service's start kwarg `:record` (not
   `:d`). (The symbol-node feeds both the binder and the body `(State/new ~d-sym)`, so the one rename keeps
   them consistent — it's a single node reused, scope-safe.)

That's the whole macro change. `start-body`/`resume-body` are UNCHANGED — they reference `locus` +
`~@init-arg-names`, and the `defn` kwargs branch reshapes the `$impl` to bind those from the `::Kwargs` record.

### ⚠ ANTICIPATED — the hygiene gate may fire on `locus` (this is the gate PROTECTING you, not a bug)

The `defn` kwargs branch rebuilds the `$impl` as
`(fn [kw <- ::Kwargs] (let [locus (::Kwargs/locus kw)  record (…) …] <start-body>))`, **reusing the argspec
binder nodes** (the arc-291 hygiene fix in `core.wat`). The init params are SAFE: `~@init-param` (binders) and
`~@init-arg-names` (body refs) are the **same nodes** spliced from the user's `:init` fn → same scope. **But
`locus` is written bare in TWO separate quasiquotes** — `start-params` and `start-body` — so its binder scope
and body-ref scope MAY diverge. If they do, the live gate fires a precise compile-time
`HygieneScopeDivergence: reference 'locus' … unbound, but a binder 'locus' exists under a different scope`.

**If that fires, the fix is the standard hygiene pattern (mint once, reuse):** near the `lr-sym` hygiene
binder, mint `locus-sym (:wat::core::symbol-node "locus")` (or `fresh-symbol`), and use `~locus-sym` in BOTH
`start-params` and `start-body` (and the resume pair) so the binder and the body reference are the **same
node**. Do NOT suppress the gate; do NOT rebuild from a string. If the gate fires on anything OTHER than
`locus` (e.g. an init param), STOP and report — that would mean a deeper scope issue.

## PART B — migrate every positional `/start` + `/resume` call site to Form A kwargs

The kwarg KEY is the init param's binder NAME. **Convention: the first init param is the durable Record (the
4b-iv-a law) → name it `record` everywhere** (the synth default is now `record`; for explicit-`:init` services
rename the first `:init` param to `record` + update its body refs). Recipe per call:
`(svc/start LOCUS RECORD [EXTRA…])` → `(svc/start :locus LOCUS :record RECORD [:extra EXTRA …])`.

**Grep to confirm the full set** (`grep -rnE '::[a-z-]+/(start|resume)' wat-tests/ tests/`). Known sites:

**wat-tests (8 files):** `service-locus-parity.wat` (counter, default→`:record`), `service-init-parity.wat`
(seeded-counter, default→`:record`), `service-admin-facet.wat` (admin-counter→`:record`),
`service-stop-resp.wat` (resp-counter→`:record`), `service-hibernate-resume.wat` (hib-counter→`:record`, +
`/resume`), `timer-env-grab-parity.wat` (deadline→`:record`), `service-multiparam-init.wat` (offset-counter —
explicit 2-param `:init`: first→`:record`, second→its name), `service-telemetry-bridge.wat` (recorder→`:record`;
**worker** — explicit `:init [r recorder-addr]`: rename `r`→`record` in the `:init` fn + its body, then
`(worker/start :locus L :record R :recorder-addr A)`; + the `/resume` site).

**tests/*.rs (wat in string literals — hand-edit):** `probe_arc272_rs2_crash_surfaces_to_client.rs`,
`probe_arc209_locus_agnostic_start.rs`, `probe_arc272_6b_defservice_on_process.rs`,
`probe_arc209_c3_defservice_client_face.rs`, `probe_arc272_rs2_process_stop_returns_final_state.rs`,
`probe_arc272_rs2_thread_stop_returns_final_state.rs`, `probe_arc272_rs1_state_must_be_record.rs` — all
`:my::counter|svc|hcounter/start (locus) (Record …)` → `:my::…/start :locus (locus) :record (Record …)`.
(Let the compiler find any missed site: a positional `/start` will fail to type-check against the new kwargs
signature.)

## STOP triggers (halt + report; do NOT improvise)

1. **STOP if the hygiene gate fires on anything other than `locus`** — report the exact `HygieneScopeDivergence`
   message; an init-param divergence means a deeper issue the brief didn't predict.
2. **STOP if the `locus` shared-symbol fix doesn't clear the gate** — report; do not reach for a string rebuild
   or a suppression.
3. **STOP if a `/start` site needs a genuine design call** (e.g. a service whose `:init` shape doesn't fit the
   first-param-is-record convention) — report rather than guess.
4. **STOP if the cascade spreads beyond defservice definers/callers** — the blast radius is `wat/service.wat` +
   the ~16 call-site files. A NON-defservice file going red is a surprise; report it.

## Gate (the orchestrator re-runs every line against the disk)

| what | command | expected |
|---|---|---|
| the bridge proves kwargs-start end-to-end | `cargo test --release -p wat --test test telemetry_bridge` (or the bridge deftests) | green (thread + hibernate tiers; process tier stays IGNORED — that's the separate TRUST leg) |
| core service lifecycle green via kwargs | `cargo test --release -p wat --test test counter_on seeded admin_stop stop_resp hibernate_resume` | green |
| multi-param init via kwargs | `cargo test --release -p wat --test test` (offset-counter deftests) | green |
| hygiene gate not regressed; macros^unbounded intact | `cargo test --release -p wat --test probe_kwargs_emitted_by_macro --test probe_macros_unbounded_depth --test probe_arc260_1b_call_sugar` | all green |
| no new workspace regressions | `cargo test -p wat --no-fail-fast`, failing-test SET vs HEAD (`910b9bcd`) | **∅** new (floor ≈ 203; weigh by SET, never absolute count) |

Runtime: 45–90 min (the macro flip is small; the migration is the bulk). Trap-doors: (a) the `locus` hygiene
(anticipated above — the gate makes it loud); (b) per-service kwarg keys (read each `:init` to get the param
names; first→`record`). Report the full migrated-site list + the verbatim gate output.
