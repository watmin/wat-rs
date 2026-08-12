# BRIEF — `<fqdn>::child-entry`: the static entry that kills the manifest

Design: `DESIGN-STONE-the-child-entry-kills-the-manifest.md`. Worked reference (type-checks and
runs today): `wat-scripts/scratch-pad/probe-arc278-child-entry-static-call.wat`.

## The work, in one paragraph

`defservice` currently generates a child `:user::main` that reaches its own service internals
through `(:wat::core::apply (:wat::core::keyword/from-string "…") …)`, and ships those internals as
a **hand-enumerated** `<fqdn>::service-forms` bundle. Replace both: emit a per-service
**`<fqdn>::child-entry`** — an ordinary parent `defn` that names `dispatch-admin` and `serve`
**statically** — and make the shipped `:user::main` a one-liner that calls it with the rendezvous
locus. Then the shipped forms come from **one `fn-forms` over `child-entry`**, and
`service-forms-def` is deleted.

## Read these, in this order, and why

1. **`wat-scripts/scratch-pad/probe-arc278-child-entry-static-call.wat`** — the worked reference.
   Its `:probe::ce::child-entry-shape` is the exact body shape you are generating, already accepted
   by the checker, and its `main` shows the `fn-forms` walk reaching the internals. **Copy this
   shape.** Run it first (`./target/release/wat <path>`) so you have seen it pass.
2. **`wat/service.wat:2065-2125`** — `child-main-form`, the quasiquoted child main you are
   splitting in two. Everything from `listener` through the `Status::Started` send moves into
   `child-entry`; what stays behind becomes the one-liner.
3. **`wat/service.wat:2101` and `:2120`** — the two `apply` sites. These are what become static
   calls. `:2101` is `dispatch-admin`; `:2120` is `serve`.
4. **`wat/service.wat:731` (`serve-name`) and `:854` (`dispatch-admin-name`)** — both already
   keyword nodes. `~serve-name` is spliced as a **static call head** ~20 times inside serve's own
   body (e.g. `:1268`, `:1346`) — that is the mechanism you are reusing, not inventing.
5. **`wat/service.wat:1478` (`serve-params`)** — serve's 5-arg contract:
   `[self, l, selectables, next-id, state]`. The call you emit must match it exactly.
6. **`wat/service.wat:2126-2190` (`service-forms-def`) and `:2190`/`:2366`** — the manifest and its
   emission sites, which this strike retires.
7. **`wat/spawn.wat:451-522`** — the `ThreadOpts` `Locus/launch` impl. **Read it to confirm you are
   not touching it.** Its `apply serve` is the generic-impl hop and it stays.

## Implementation sketch — fill this in, do not invent the shape

```clojure
;; NEW, emitted per service, a real top-level defn (goes in the same emission list as serve):
child-entry-def
`(:wat::core::defn ~child-entry-name [~locus-sym <- <THE LOCUS TYPE — see STOP-1>] -> :wat::core::nil
   (:wat::core::let
     [~cm-b-sym    (:wat::kernel::listener ~locus-sym ~proto-op-ty-kw ~proto-reply-ty-kw ~max-frame-bytes-node)
      ~cm-self-sym (:wat::program::self-peer ~status-ty ~admin-ty)
      ~cm-ship-sym (:wat::core::match (:wat::kernel::recv ~cm-self-sym) …)   ; VERBATIM from today
      ~cm-st-sym   (~dispatch-admin-name ~cm-ship-sym)                        ; ← was `apply`
      ~cm-und-sym  (:wat::core::match (:wat::kernel::send …) …)]              ; VERBATIM from today
     (~serve-name ~cm-self-sym                                                 ; ← was `apply`
       (:wat::spawn::Bound/listener ~cm-b-sym)
       (:wat::core::Vector ~selectable-entry-ty)
       0
       ~cm-st-sym)))

;; The shipped main collapses to one line naming the FREE rendezvous keyword:
child-main-form
`(:wat::core::defn :user::main [] -> :wat::core::nil
   (~child-entry-name :user::spawn::service-locus))
```

Then: the process arm's shipped forms become `(:wat::kernel::fn-forms ~child-entry-name …)`
concat the one-liner, and `service-forms-def` is deleted along with its `~service-forms-kw`
plumbing.

## Blast radius

`wat/service.wat` ONLY. No `src/` Rust. No `wat/spawn.wat`. No test fixture edits unless a STOP
fires. Every `defservice` in the corpus recompiles — expect a wide cascade on the first build and
read it as the worklist (`docs/SUBSTRATE-AS-TEACHER.md`), not a crisis.

## STOP triggers — each REJECTS; ship nothing and report

1. **STOP-1 — the locus parameter's TYPE.** The reference probe sidesteps this by taking `l`
   directly. `child-entry` must take the locus. If you cannot type that parameter such that
   `(:wat::kernel::listener locus …)` accepts it AND the child's `(def :user::spawn::service-locus
   (process))` value satisfies it, **STOP and report the exact checker error.** Do not widen a
   signature, do not reach for `:wat::core::Value`, do not fall back to `apply`.
2. **STOP-2 — a hygiene refusal.** The child main's binders are `symbol-node`+unquote precisely to
   dodge `ProgramBodyIntroducesName` (`service.wat:2047`). If moving the body into a real defn
   trips a hygiene or reserved-prefix gate, **STOP and report the gate's name and the form it
   refused.**
3. **STOP-3 — `fn-forms` over `child-entry` does NOT reach a name the manifest carried.** Diff the
   walk's declared names against today's `service-forms` output for the same service. Any name
   present in the manifest and absent from the walk is a REJECTION: **STOP and report the missing
   names.** Do not re-add a hand-enumerated form to paper the gap.
4. **STOP-4 — the thread tier changes behaviour.** It must not. If any thread-tier test moves,
   **STOP** — the thread arm ignores `service-forms` and this strike is process-side only.
5. **STOP-5 — you find yourself editing `wat/spawn.wat` or any `src/*.rs`.** Out of blast radius.
   **STOP and report why you believe it is needed.**

## Verification you run yourself

**Cargo is yours.** You are the only rider in the field, so the `target/` build lock belongs to
you — build and test freely. (The standing caution about riders and cargo is about N riders
thrashing one lock, which does not apply to a solo strike.)

**Run every verification in the FOREGROUND and block on it.** You are a rider, not the
orchestrator: **ending your turn ENDS you.** It does not suspend you, nothing will wake you, and no
notification is coming. Your turn ends when the numbers are in your hands, not when the command is
launched. A backgrounded run you return early from reports nothing.

- `./target/release/wat --check <a corpus service .wat>` — fast per-file arbiter (~0.2s), after a
  `cargo build --release` (a `wat/` edit is baked, so the binary must be rebuilt to see it).
  **Read the output, not a piped exit code:** `--check f | tail` returns *tail's* exit status.
- `:wat::deporder::verify-stdlib` must print `[]` (a two-line `:user::main`) — catches stdlib
  load-order violations `--check` cannot see. Mandatory for any `wat/` edit.
- `cargo test --release --test services -- probe_arc209_c2_defservice_dispatch` and
  `-- probe_arc272_6b_defservice_on_process` — the two closest existing gates.
- `cargo nextest run --release` for the whole floor once you are close. Baseline is
  **4391 passed / 0 failed / 262 skipped**. Read the **Summary line** — never a piped exit code.

**On a RED: do NOT re-run it.** A re-run that goes green destroys the only evidence. Copy the
failing test's whole stdout+stderr **verbatim** into your report, name the exact assertion or match
arm that fired, and surface it. There is no such thing as a known flake.

## Do not

Do not commit, push, stash, or revert. Do not use a git worktree. Leave the tree dirty for the
orchestrator, who re-runs the scorecard and weighs the floor independently.
