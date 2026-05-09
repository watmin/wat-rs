# Arc 168 — INSCRIPTION

**Inscribed 2026-05-08 by orchestrator.** All slices shipped.

## What shipped

`:wat::core::let` consumes a flat-shape vector binding-list with
sequential same-shape pair-iteration. Body is implicit-do over
trailing forms. `:wat::core::fn` and `:wat::core::defn` extend
identically — body forms are 1+ trailing forms, last form's value
is the function's return value:

```scheme
;; Canonical
(:wat::core::let [x 1 y 2] (:wat::core::+ x y))

;; Sequential — y sees x
(:wat::core::let [x 1 y (:wat::core::+ x 1)] y)

;; Empty bindings — Clojure-faithful, returns the body
(:wat::core::let [] (:wat::core::+ 1 1))

;; Empty body — returns :wat::core::nil
(:wat::core::let [x 1])

;; Tuple destructure — Vector of symbols inside binding Vector
(:wat::core::let [[a b c] some-tuple] (:wat::core::+ a b c))

;; Multi-form body — implicit-do; last form is the value
(:wat::core::let [x 1]
  (:my::log "computing")
  (:wat::core::+ x 1))

;; Same body shape for fn
(:wat::core::fn
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:my::log "called")
  (:wat::core::+ x 1))

;; Same body shape for defn (via macro forwarding)
(:wat::core::defn :user::add5
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:my::log "adding 5")
  (:wat::core::+ x 5))
```

The legacy nested-pair-list shape `((name expr) (name expr) ...)`
is gone with no scaffolding remaining. Per the user's
"doesn't leave cruft" discipline, the slice 1 walker fired during
the slice 2 sweep window and hard-retired in slice 3. The
substrate has zero trace of legacy outer-list let support
post-arc-168.

## Slices

| Slice | Commit(s) | What landed |
|---|---|---|
| 1 | `fcf45fe` (substrate WIP) + `8b24398` (eval_fn / step_let / walker) + `83000b7` (tests + odd-count error) | substrate consumer; `BareLegacyLetBindings` walker; `parse_let_binding` Symbol + Vector binders; `eval_fn` synthesize_fn_body multi-form; 15 integration tests |
| 2 | `2136389` (stdlib) + 20 batch commits | sweep ~563 legacy let callsites across `wat/`, `wat-tests/`, `tests/wat_*.rs`, `crates/*/`, `examples/`; 21 batches, ~133 min sonnet |
| 2 follow-up | `b220846` | `register_runtime_defs_form` + `collect_splice_defs_ctx` companion paths slice 1 missed; empty-bindings test migration; FM 5 incident (legacy support added back to make fix work; reverted to Vector-only) |
| 3 | `f108a13` | substrate retirement: `BareLegacyLetBindings` variant + Display + Diagnostic + walker body + freeze.rs registration + `eval_let` legacy List arm + `parse_let_binding` typed-legacy `(name :T)` arm + `parse_legacy_let_binding` entire function + `step_let` legacy arms + 2 vacuous tests retired |
| 4 | `3b34d62` | sweep 81 src/ lib unit-test fixtures (slice 3 retirement leftovers; mirror of arc 167 slice 4b precedent at 5× scale); ~35 min sonnet |
| 4 follow-up A | `89fd0ac` | `check_let_for_scope_deadlock_inferred::binding_names` Vector arm fix |
| 4 follow-up B+C | `751addb` | `walk_for_pair_deadlock` + `extend_pair_scope_with_tuple_destructure` Vector arm fixes; 6 lib unit-test fixtures swept (accidentally-green migrations surfaced parallel walker gap) |
| 5 | `93b6ac7` (SCORE-1) + this commit (closure paperwork) + arc 169 DESIGN companion `ff5dcfb` | the closure record + atomic squash-merge to main |

The slice branch (`arc-168-let-flat-shape`) carries 35+ commits
including slice 2's batch sweep; main has been untouched
throughout. Atomic squash-merge to main happens at this commit.

## Substrate impact

| Surface | Pre-arc-168 | Post-arc-168 |
|---|---|---|
| let outer shape | `((name expr) (name expr) ...)` legacy List | `[name expr name expr ...]` flat Vector |
| let body | single trailing form | 1+ trailing forms; implicit-do over args[1..]; last form is value; empty → `:wat::core::nil` |
| fn body | single trailing form | 1+ trailing forms; same implicit-do shape as let |
| defn body | single trailing form | 1+ trailing forms; macro forwards N body forms cleanly |
| Empty bindings | not legal (parser rejected) | `[]` legal; body returns directly (Clojure-faithful) |
| Empty body | not legal | legal; returns `:wat::core::nil` (Clojure-faithful) |
| Tuple destructure | `(((a b c) rhs) ...)` legacy List shape | `[[a b c] rhs ...]` flat Vector with Vector binder |
| Typed-single binder | `((name :T) rhs)` legacy shape | DELETED (arc 159 retired in user code; arc 168 retired the parser arm) |
| `BareLegacyLetBindings` walker | n/a (minted slice 1) | DELETED (slice 3) |
| `parse_legacy_let_binding` function | n/a (minted slice 1) | DELETED (slice 3) |
| ScopeDeadlock check on flat-shape destructure | gap (Vector binder ignored) | accepts Vector arm |
| ChannelPairDeadlock check on flat-shape | gap (Vector outer ignored) | accepts Vector arm |
| Workspace test count | 2074/8 post-slice-2 | 2075/5 post-slice-4 (5 are pre-existing kernel/signal failures unrelated to arc 168; out of arc 168 scope per SCORE-SLICE-2 delta C) |

## Settled design

### Why flat-Vector outer

The user articulated the rule 2026-05-08:

> *"let's work on 168 - let forms in brackets like clojure has"*

Plus the substrate consistency constraint: arc 167 minted
`WatAST::Vector` as a first-class node specifically for fn-sig
position. Extending `Vector` to let bindings is the natural
parallel — same node, same parser, same ergonomics. Arc 168
opened Vector's legal positions to `:wat::core::let`'s binding-
list slot. Future arcs may extend further (arc 169 struct-
destructure form A is the next queued extension).

### Why empty bindings + empty body return :nil (Clojure-faithful)

User direction 2026-05-08 (per conversation thread referenced
in DESIGN):

> *"clojure precedence" (showing Clojure REPL examples)*

Clojure's `(let [] body)` returns body's value; `(let [x 1])`
returns nil. Wat mirrors. The substrate's `:wat::core::nil`
keyword is the empty-body return; this fits the existing
`:wat::core::nil` semantic without minting new vocabulary.

### Why multi-form body via implicit-do (not require explicit `do`)

User direction 2026-05-08:

> *"yes.. a user... for whatever reason... would need to express
> (:wat::core::defn f [] -> :wat::core::nil)"*

Multi-form body is a natural consequence of accepting `args[1..]`
as a sequence; the substrate synthesizes `(:wat::core::do f1 ... fN)`
internally. Users see the flatter syntax; the substrate still
runs through the canonical `do` evaluation. No new substrate
mechanism needed.

### Why hard retirement vs preserving scaffolding

Per user direction (arc 167 precedent inherited):

> *"no bandaids nor half measures... arcs can be as complex as
> they need to be - we just add more slices as we need."*

Slice 3 deleted every transitional piece slice 1 had as a
legacy fall-through. The walker fired for one sweep window
(slice 2); after that window closed (slice 3), the substrate
returned to the pure canonical-only path. The `MalformedForm`
error covers any legacy shape that surfaces from user code
post-retirement.

## Substrate-as-teacher cycle

Slice 3's substrate retirement created two parallel walker gaps
that the slice 4 sweep + Delta C migration surfaced:

1. `check_let_for_scope_deadlock_inferred::binding_names` — only
   matched Symbol + List(parts); the Vector binder shape (arc
   168 flat-shape destructure) fell through to vec![]. Names
   never reached the ScopeDeadlock collector.
2. `walk_for_pair_deadlock` + `extend_pair_scope_with_tuple_destructure`
   — early-returned on Vector outer; walker never ran on flat-
   shape lets.

Both fixes shipped in slice 4 follow-up commits (`89fd0ac`,
`751addb`). The pattern: substrate retirement creates parallel
surfaces in adjacent walker code; the sweep + post-sweep
migration surface them; each fix lands at substrate level (not
bridged at the test). FM 5 held throughout.

## FM 5 incident records

Two FM 5 incidents recorded during arc 168:

1. **Slice 2 follow-up first draft** — added legacy List support
   back to `register_runtime_defs_form` "to make the fix work."
   User caught: *"did you just make retired forms acceptable?"*
   Reverted to Vector-only.
2. **Delta C migration revealing Δ-B** — `arc_133_tuple_destructure_pair_check_fires`
   flipped from accidentally-green to actually-failing post-
   migration. Held FM 5: STOP, fix at substrate (the pair-deadlock
   walker Vector arm gap), not at the test.

Both reinforced the discipline: substrate retirement creates
parallel surfaces; bridging at the test masks the gap; substrate
fix is the right answer.

## Workspace state at INSCRIPTION

`passed: 2075 failed: 5`. The 5 are pre-existing kernel/spawn/
signal failures unrelated to arc 168:
- `fork_program_round_trip_via_pipes` (arc 104 fork machinery)
- `sigterm_cascades_two_levels_via_process_group`
- `sigterm_to_cli_cascades_via_polling_contract`
- `presence_proof_hello_world` (substrate proof harness)
- `programs_are_atoms_hello_world` (substrate proof harness)

These predate arc 168 (verified pre-edit by stash round-trip
during sonnet's slice 2 sweep). Out of arc 168's scope.
Substrate-architectural reason: kernel/spawn/signal failure
modes orthogonal to let bindings. Arc 168 intentionally does NOT
cover them and does NOT reserve a number for the investigation
arc. If/when investigation begins, that arc opens with its own
number assigned at start.

## Companion arc — 169 (struct-destructure form A)

Arc 169 DESIGN.md was authored 2026-05-08 in this branch as a
companion artifact (commit `ff5dcfb`). Mints the struct-
destructure form `[{field1 field2 ...} struct-value]` for let
bindings via the four-questions discipline. Arc 109 v1 milestone
closure depends on arc 169 shipping per user direction.

The DESIGN is documentation-only; no substrate code in `ff5dcfb`.
The atomic squash-merge for arc 168 carries arc 169's DESIGN
file as a side-load (DESIGN content unchanged). Arc 169's
substrate slices ship via their own slice branches off main
post-arc-168 closure.

## Cross-references

- DESIGN.md — settled four-questions evaluation, scope, dependency ordering
- BRIEF-SLICE-1 + EXPECTATIONS-SLICE-1 + SCORE-SLICE-1 — substrate consumer + walker + tests
- BRIEF-SLICE-2 + EXPECTATIONS-SLICE-2 + SCORE-SLICE-2 — sonnet sweep ~563 callsites + slice 2 follow-up gaps
- BRIEF-SLICE-3 + EXPECTATIONS-SLICE-3 + SCORE-SLICE-3 — substrate retirement
- BRIEF-SLICE-4 + EXPECTATIONS-SLICE-4 + SCORE-SLICE-4 — sonnet lib fixture sweep + parallel walker fixes
- arc 167 INSCRIPTION — fn-flat-signature precedent that minted `WatAST::Vector`
- arc 169 DESIGN — struct-destructure form A; v1 closure blocker for arc 109
