# BRIEF — exactly one call position, and `(Head :- [])` IS `Head`

Two blockers stopped `defservice` emitting the binder last stone. Both are located and measured. You
will close them and then land the thing they blocked.

Read `DESIGN-STONE-exactly-one-call-position.md` first, and
`SCORE-STONE-the-last-comma-lives-in-a-symbol.md` for the report shape. The tree is CLEAN and the
floor is green at 4903/4903.

## STEP 1 — hoist the peel above the dispatch

Both call arms live in **one function**, `infer_list` (`src/check.rs:2540`):

```
if k.contains('/') {           ← 4962   surface-method arm, fires FIRST
    split_type_params_pub(…)   ←        reads type args from the ANGLE SUFFIX
    …its own arity check…
}
…
peel_param_spec(args)          ← 5494   generic-call arm (learned `:-` in 69933d362)
```

Move `peel_param_spec` **above** the `if k.contains('/')`, to where the call form is first
destructured. Both arms then consume `(type_args, args)` already separated; neither extracts.

The rule, because it decides the placement: **whether a call carries a param-spec is a property of the
FORM; which arm handles it is a property of the CALLEE.** Form-level work belongs above the branch.

Then the surface arm must BIND the callee's declared type params from the peeled args — same as the
generic arm already does — not merely accept and drop them. Row 3 below is what proves it.

`src/runtime.rs:7046` is the same arm at runtime, with `7432` already peeling. Same hoist.

## STEP 2 — delete the angle-suffix read

`split_type_params_pub` in the surface arm is the **pre-`:-`** carrier. Once step 1 lands it has
nothing to do. Delete its use there. If some other caller still needs the function, say so and leave
the function; if this was its last real use, retire it and report that.

## STEP 3 — `(Head :- [])` normalises to `Head`

`parse_type_form`'s `:-` arm in `src/types.rs` builds `TypeExpr::Parametric { head, args }` with no
empty-guard, and its comment says why:

> *"there is no `!inner.is_empty()` guard here, because under `:-` an empty bracket is a legitimate
> zero-length param-spec (`(Tuple :- [])`), not something to guess about."*

**That reasoning is SUPERSEDED.** The builder's rule this session is that absent, `:- []`, and the
empty binder are the same thing — so `Parametric{args:[]}` must not exist. Normalise to
`TypeExpr::Path(head)` when the peeled vector is empty, and **replace that comment** with the
superseding rule and a pointer to this stone. Do not leave the old reasoning standing next to code
that contradicts it.

## STEP 4 — impose the check; do not trust the one door

Step 3 fixes the door where the measured defect enters. **That is not proof no other door builds the
empty form.** Impose it: assert `Parametric{args:[]}` is never constructed — a `debug_assert!` in a
constructor, or a temporary check at the construction sites — run the floor, and read what screams.

⚠ A debug build currently hits an unrelated pre-existing panic at `src/types.rs:577`
(*"builtin leaf :wat::core::Option already registered"*) before any test runs — last stone hit this.
So a `debug_assert!` may be unusable; a temporary release-mode `eprintln!` or a hard `panic!` is the
fallback. Report which instrument you used and what it could see.

Remove the temporary probe before you report, and say what it found.

## STEP 5 — land what was blocked

Now `wat/service.wat` can do what the previous stone had to revert:

```
2374-85  launch-head-kw       → the BARE keyword, `:- [...]` as call-site siblings
500      proto-tp             → dies with its last consumer
942-943  proto-op-ty-kw / proto-reply-ty-kw   → DEAD BINDINGS (one occurrence each), delete
1021 1024 1360 2014 2025      → five `(if (empty? proto-args) …)` branches go UNCONDITIONAL
```

★ The exemplar is in that file: `proto-op-ty-ann` (line 1021) already mints the reference FORM
structurally off `proto-args`. The only edit to the exemplar itself is dropping its `if`.

⚠ A stdlib `.wat` edit is INVISIBLE until you rebuild.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★★ | `(Head :- [])` ≡ `Head` at reference, ctor AND declaration | all three agree |
| 2★★★ | a SURFACE-METHOD call takes the binder | `(:S/method :- [T] recv arg)` checks and dispatches |
| 3★★★ | the surface arm BINDS, not just accepts | a WRONG explicit type arg → TypeMismatch, not silence |
| 4★★ | `defservice` expands, checks, dispatches | a value comes back |
| 5★★ | a PARAMETRIC defservice round-trips | lru-svc / hologram-svc |
| 6★★ | nothing in `wat/` mints an angle name | `grep -c 'launch<' wat/service.wat` → 0 |
| 7 | the imposed check | what step 4 found, and that it is closed |

**Row 3 decides it.** Rows 1–2 go green for an arm that peels the binder and throws it away. Only a
wrong type argument being CAUGHT proves the surface arm binds — that is the row that separates
"accepts the syntax" from "means it", and it is exactly what proved the generic arm in `69933d362`.

## STOP triggers

- **STOP-1 — hoisting changes what a currently-green call means.** Report the program and both
  meanings; do not proceed on that arm.
- **STOP-2 — step 4's imposed check screams somewhere step 3 does not cover.** Report every site. That
  is the finding, not an obstacle.
- **STOP-3 — a fifth caller of `split_type_params_pub` genuinely needs the angle suffix.** Report it;
  it means something still carries type args the old way.
- **STOP-4 — dropping an `(if (empty? proto-args) …)` branch changes a monomorphic service.** Then
  step 3 is not actually done. Report which service and what changed.

## Boundaries

- `src/check.rs` (`infer_list`), `src/runtime.rs` (the dispatch arm), `src/types.rs`
  (`parse_type_form`), `wat/service.wat`, and goldens.
- **Do NOT apply the minting wall or `symbol-node`'s wall.** Next stone.
- **Do NOT delete the angle PARSERS** (`split_type_params`, `canonical_callable_name`,
  `check.rs:5159`'s explicit-suffix arm). After the wall, with a green floor to prove them dead.
- **Do NOT touch `keyword/from-string`.** Its own NOTE, decided with the verb-equals-type family.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- Goldens: the ruling is **KEEP PINNING THE SPAN** and recapture — the pin discriminates the emitter.
  Verify each recapture is the same call site, only moved.
- The orchestrator runs the full floor and clippy centrally. Use `./target/release/wat --check <file>`
  (~0.2s) and scoped `cargo nextest run --release -E '...'`.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.
`cargo wat` uses the STALE installed binary; always `./target/release/wat`.

## Your report

Row 3 verbatim first — the wrong-type-argument refusal — because that is the row proving the surface
arm means it. Then rows 1, 2, 4-7. What step 4's imposed check found. Whether
`split_type_params_pub` had any surviving caller. Any STOP that fired, with the arm captured verbatim
BEFORE you diagnosed it. What surprised you.
