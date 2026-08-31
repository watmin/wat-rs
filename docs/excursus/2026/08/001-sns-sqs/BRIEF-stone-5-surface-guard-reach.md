# BRIEF — excursus 001 stone 5: the surface-completeness guard sees a parametric field type

**Fix the teacher before re-running the student.** Stone 4 was sent in the wrong direction by a
guard that already exists, already says the right thing, and does not look at the field type
`wat-queue` used.

> Builder: *"i dislike that grok was informed to go in the wrong direction… how do we attack
> that before we have grok attempt this proof again?"*

## The defect, in one branch

`src/types/surface.rs:927-935` — the collector that feeds the `:messages` completeness guard:

```rust
if s.as_str() == "<-" {
    if let Some(WatAST::Keyword(k, _)) = children.get(i + 1) {   // ← ONLY a Keyword
        if let Ok(te) = super::parse_type_expr(k) {
            collect_user_type_paths(&te, out);                    // ← this descends correctly
        }
    }
}
```

A field whose type is a **parametric form** — `(:wat::core::Vector :- [:p::Item])` — is a
`WatAST::List` after the `<-`, not a `Keyword`. The `if let` does not match, and the field is
**skipped entirely**. `collect_user_type_paths` handles parametrics perfectly (`:977-983`); it
is simply never called for these.

## Measured, both directions

```wat
:Ok [item  <- :p::Item]                        →  guard FIRES   (--check = 1)
:Ok [items <- (:wat::core::Vector :- [:p::Item])]  →  guard MISSES  (--check = 0, clean)
```

Reproductions are in the scratchpad and should be promoted as the gate (see below). The second
is exactly `wat-queue`'s shape: `Queue::ReceiveResponse::Ok` carries
`(Vector :- [:queue::Envelope])`.

## What it cost — the reason this stone exists

`wat-scripts/queue/sqs.wat` froze clean. The failure surfaced only at **runtime, in a forked
child**, as `unknown callee: :queue::Envelope/id` — a message that carries no context about
bundles or `:peers` (its `context` field is a `&'static str`, one of two fixed literals,
`src/resolve/error.rs:13`). The correct inference from *"this name is not available here"* is
*"make it available another way"*, so stone 4's executor reached for a foreign-read workaround
before diagnosing the real cause. **It got there — but by derivation, from a runtime symptom,
after a wrong turn the substrate invited.**

The guard's existing message is excellent and would have ended it at authorship:

> *"surface `:p::Src` `:messages` type references `:p::Item` which is not declared in this
> surface's `:messages` — a peer surface that owns `:messages` must declare EVERY non-stdlib
> request/response type it uses, so a `:satisfies` service ships them across a process fork."*

★ **The fix is reach, not wording.** This is `extirpare`'s ladder with the check already on the
right rung — construction-time, located, actionable — and an incomplete reach. Widen the reach
and the class becomes unrepresentable.

## Read in order

1. **`src/types/surface.rs:921-940`** — `collect_message_form_type_refs`. The `<-` handler at
   `:928-936` is the defect; `:938`'s recursion into nested collections already works.
2. **`src/types/surface.rs:974-995`** — `collect_user_type_paths`. **Do not change it.** It
   descends into `Parametric`, `Tuple`, and `Fn` correctly and is the thing to reuse.
3. **`src/types/surface.rs:681-690`** — the caller, and its comment: *"required-membership check
   on each form's DIRECT refs closes the transitive graph: if A references B, B must be in
   `:messages`; B's own refs are checked when B's form is walked."* That argument is sound and
   is **why fixing the one branch is sufficient** — it does not need a new transitive walk.
4. **`src/types/surface.rs:~840-880`** — the sibling guard on *feature signatures*, which
   already uses the descending collector. The shape to match.

## Implementation sketch

The `<-` handler accepts a parametric type form as well as a keyword. Both paths end in the same
`collect_user_type_paths`. Rendering a `WatAST::List` type form to a `TypeExpr` is the only new
step — **find the existing door for that rather than writing a parser**; the codebase already
turns type forms into `TypeExpr` in several places.

## The gate

Promote both reproductions into a test:

- **the parametric case must now FAIL to freeze**, with the existing message naming `:p::Item`
- **the direct case must still fail** — unchanged, so the fix is a widening and not a rewrite
- **a clean surface must still freeze** — the guard must not start rejecting valid code

Assert the reason **byte-identical**, not with `.contains(` — `no_loose_string_assert`, and the
sibling test at `:1200-1210` already does exactly this and says why.

## ⛔ What this stone does NOT do

- **It does not fix `wat-queue`.** After this lands, `wat-scripts/queue/sqs.wat` will **fail to
  freeze** — correctly, and that is the point. Moving `Envelope` into `:messages` is the next
  stone, and the floor will be RED between them. **Say so in the SCORE; do not fix the queue
  here.** If the floor's redness across two stones is unacceptable, STOP and say so rather than
  bundling them.
- **It does not touch `UnresolvedReference`.** The `&'static str` context is a real, separate
  weakness (`SUBSTRATE-AS-TEACHER.md` step 1 would have it carry the bundle's shipped surfaces),
  but with this guard fixed, no correct program reaches that runtime error by this route.
  Recorded, not drawn.

## STOP triggers

1. **If the fix requires changing `collect_user_type_paths` — STOP.** It is correct. If the
   parametric path needs it changed, something else is going on and it is a finding.
2. **If widening the reach makes an EXISTING surface fail to freeze — STOP AND REPORT WHICH.**
   That is a real defect the guard could not see, and it is worth more than this stone. Do not
   fix it; name it. (`wat-scripts/queue/sqs.wat` is the known one and is expected.)
3. **If the floor reds anywhere other than the queue — STOP**, capture whole, do NOT re-run.

## Blast radius

`src/types/surface.rs` — one branch plus its test · the promoted gate · this excursus's SCORE.
**Expected consequence: `wat-scripts/queue/sqs.wat` stops freezing.** That is the guard working.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

Floor is **5122, currently green**. This stone is expected to make it RED on the queue, and only
on the queue.
