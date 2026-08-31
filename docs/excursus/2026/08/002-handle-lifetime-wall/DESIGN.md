# DESIGN — the handle-lifetime wall

**excursus 002.** Commissioned 2026-08-31. Not an arc — no arc was asked for; `docs/excursus/` is
the sibling tree for work the builder said "do it" to without opening an arc number.

## The failure

A `Peer` can outlive the `Handle` that owns its service. When it does, the service is severed and
the peer is a live channel to nothing. The idiomatic program trips it:

```wat
(:wat::core::defn :app::run [] -> :wat::core::i64
  (:wat::core::let
    [h (:app::svc/start :locus (:wat::spawn::thread) :record …)
     c (:app::conn h)]
    (:app::drive c)))          ;; tail position: this scope ends BEFORE the call runs
```

Nothing here is wrong by any convention in the language. `eval_let_tail` (`src/runtime.rs:4618`)
propagates `EvalSignal::TailCall` out of the let, the scope drops, `h` goes with it, and `drive`
meets a dead service. **This cost 38 days** on `probe_arc278_self_scheduling`, where the symptom
`recv': peer closed` was reasoned into "the timer's `remove-at` is evicting the client" and written
into an `#[ignore]` reason as though measured.

**This is NOT a TCO defect.** TCO's frame release is correct and load-bearing — every serve loop
depends on it, and a tail call whose caller's frame survived would grow memory without bound. The
gap is that wat has values whose `Drop` is observable at a distance and no way to say "this must
outlive the call". Threading it as an argument is the existing idiom: `serve` does exactly that
with `selectables`/`state`.

## The rule

> **A `Peer` may not escape a scope that CREATES its service's `Handle`.**

Creation is a `<svc>/start` call. The FQDN names the service, so no new registry is needed.

⛔ **The discriminator is CREATION, not the parameter — and getting this wrong was the first
draft's error.** These two signatures are identical:

```wat
(defn :app::conn [h <- :app::svc::Handle] -> (Peer :- [S::Op S::Reply]) …)   ;; SAFE
(defn :app::dial-and-drop []            -> (Peer :- [S::Op S::Reply]) …)     ;; DIES
```

A rule keyed on "param is a Handle, return is a Peer" rejects every `conn` helper in the corpus.
`conn(h)` is safe precisely because the handle came from the CALLER, who still owns it.

| shape | handle from | peer escapes | verdict |
|---|---|---|---|
| `conn(h)` | a param — caller owns it | yes | ✅ accept |
| `dial-and-drop()` | `/start` in this scope | yes | ⛔ reject |
| tail escape | `/start` in this scope | via tail call | ⛔ reject |
| `held` | `/start` in this scope | no (returns `i64`) | ✅ accept |

## Three escape shapes — and only one needs a concept the checker lacks

- **1a** — the peer escapes a `let` that created the handle, as the let's VALUE. Decidable at
  `infer_let` (`src/check.rs:7749`).
- **1b** — the peer escapes a FUNCTION that created the handle, as its return. Decidable at
  `src/check.rs:1805-1822`, where `locals` and `scheme.ret` are co-present — proven, because that
  is the site that produced the probe's `ReturnTypeMismatch` naming both.
- **2** — the peer leaves via a TAIL CALL. **The checker has no notion of tail position**: all 21
  `tail` hits in `check.rs` are `strip_prefix` string tails. This needs a concept that does not
  exist yet.

**Stone 1 is 1a + 1b.** Stone 2 is case 2, and it is not drawn until stone 1 lands and the
tail-position question is answered on its own.

## The contract decision, pinned

**One new `CheckErrorKind` variant**, raised at the two stone-1 sites. It names: the escaping
peer's service, the `<svc>/start` call that created the handle (span), and the escape site (span).
It does NOT attempt a remedy string — the right fix is context-dependent (thread the handle, or
return it too), and a wrong ranked remedy is worse than none.

## Measured blast radius — the acceptance criterion

Census, this session, `grep -rn --include=*.wat -E '\-> \(:wat::kernel::Peer' tests/ wat-scripts/ wat/ wat-tests/ examples/`:

- **18** functions return a `Peer`.
- **16** take an `Address` or a `Handle` as a PARAM and are safe — including all three stdlib
  `stdio-connect-{out,err,in}` (`wat/kernel/services/stdio.wat:155/164/173`), `demo::dial-topic`,
  `fanout::dial-worker`, `cache-svc/dial`. **Not one of them creates a handle.**
- **2** create-and-escape, and BOTH are deliberate probe targets:
  `tests/services/probe_severed_reaches_the_client.wat:68` (`:sev::dial-and-drop`) and
  `wat-scripts/scratch-pad/probe-handle-to-surface-relation.wat:131`.

So the wall must reject exactly two sites and accept sixteen. That is the shape of a real wall
rather than a blunt one, and it is checkable before and after.

## ⚠ THE COLLISION — the wall makes its own witness unbuildable

`:sev::dial-and-drop` is the subject of the floor gate
`tests/services/probe_severed_reaches_the_client.rs`, which exists to prove an owner-drop reaches
the client as `Severed` rather than a mute `Closed`. **That gate must construct the forbidden state
on purpose.** When the wall lands, it stops compiling.

This is not a reason to soften the wall. It is what the repo's `rune:` exemption form is for: the
gate carries a rune naming the wall and stating why the construction is deliberate. Any strike that
"solves" this by weakening the rule, or by deleting the gate, has traded a proof for a green build.

Out of scope = REJECTED, affirmatively:
- case 2 (tail escape) — its own stone, gated on the tail-position question.
- any change to TCO, `eval_let_tail`, or the trampoline. The runtime is not touched.
- any change to `LociDiedError::Severed` or the severed sentinel. The runtime notice stays as the
  backstop it is; measured racy at 6/10 in the tightest shape, which is exactly why a static wall
  is worth having.
