# 259 S3b — the loci-agnostic bracket: two blockers, both resolvable (2026-07-07)

> **STATUS.** S3b (widen `map`/`each`/`map-worker`/`collect-loop` from `ThreadOpts` → `:Locus`) hit
> two substrate blockers when the shadowdancer built it. The **thread arm works end-to-end and is
> test-clean**; the **process arm** and a **deporder analyzer bug** were the walls. Both are resolvable
> — this note captures the corrected design + the resolution so we build it right on resume. The S3b
> WIP (bracket.wat + spawn.wat) is **stashed** (`git stash list` — "259 S3b WIP (blocked on Blocker A
> generics)") by the deporder-fix agent; the thread arm redoes mechanically.

## The corrected `spawn-runner` design (already de-risked by a probe)

`spawn-runner` is a method on the `:wat::spawn::Locus` **surface** (`Locus` is a `defsurface :nature
:Struct` since Stone A / R38), dispatched by two `extend-type` impls (ThreadOpts, ProcessOpts). It
takes the **RAW `Fn(I)->O`** work-fn — NOT the index-wrapping `wf` closure.

**Why raw, not `wf`** (disconfirming probe `scratchpad/probe-s3b-crux-fnforms-closure.wat`, RED, proved
it): `fn-forms` (closure_extract slice-1) **cannot reify a closure that CAPTURES a fn value**
(`closure_extract.rs:2025` — deliberate slice-1 limit, "same as fn/Stream"). The index-wrapping
`(fn [pair] (Tuple (first pair) (work-fn (second pair))))` captures `work-fn`, so fn-forms'ing IT fails.
So each tier index-wraps its OWN way over the raw fn:
- **Thread arm**: build the index-wrapper inline (a thread closure captures freely — no fn-forms) + `runner-loop`.
- **Process arm**: `fn-forms` the RAW work-fn (top-level, no captured fn) + ship an index-wrapping pool-runner.

The raw-fn process shape is GREEN standalone: `scratchpad/probe-s3-process-runner.wat` → `"6 10"`.

## Blocker A — the process arm's generic types can't be monomorphized into shipped source

**The wall.** The shipped `__pool-runner` carries peer annotations `Peer'<(i64,I),(i64,O)>` and
`self-peer :(i64,O) :(i64,I)`, where `I`/`O` are `spawn-runner<I,O>`'s **checker-only** type-params.
A generic RUNTIME method has no concrete `I`/`O` at ship time, and a generic fn does NOT substitute its
type-param into a `(forms …)` quote — it lands as literal `:I`/`:O`, unbound in the child universe.
(`self-peer`'s type args must be literal keywords — `runtime.rs:19768` — not inferred.) `defservice`
escapes this only because it's a **macro** that splices the concrete type AST at expansion; a dispatched
*method* has no such access.

**The resolution (ratified direction — parent-side AST-splice).** The concrete types **already live in
the fn-forms'd work-fn**: `fn-forms` of `:my::double` emits `(defn :bracket::__pool-work [n <- :wat::core::i64]
-> :wat::core::i64 …)` — `:i64` is *literally in the shipped define*. So the process arm never needed
`spawn-runner`'s erased generics at all. `spawn-runner`'s ProcessOpts impl:
1. `(fn-forms work-fn :bracket::__pool-work)` → a `Vector<WatAST>` whose define carries the CONCRETE arg/return type keywords.
2. Read those keywords off the AST (`ast->children` → the `<- :T` arg type + `-> :R` return type — the
   `deporder.wat`/`fix.wat` walk pattern), and SPLICE concrete `self-peer`/`Peer'` types into the shipped
   pool-runner. Generic `I`/`O` never cross; the concrete types come from the reified work-fn.

**Why AST-splice, not runtime reflection in a child macro.** The reflection family exists and is rich —
`:wat::runtime::extract-arg-types`, `return-type-of`, `signature-of-defn`, `signature-of-fn`,
`body-of`, `lookup-define`. AND shipped forms DO macro-expand in the child (`expand_all` runs on every
child-universe load, `runtime.rs:27129+`). BUT runtime reflection resolves against **registered**
defines, and `expand_all` runs **before** the child registers its defns — so a child macro calling
`return-type-of :__pool-work` at expand time may not find it registered. Operating on the **AST**
(parent reads the fn-forms output, or a child macro takes the define *form* as its arg and
pattern-matches it) avoids the ordering trap — the types are in the AST regardless of registration
order. Parent-side AST-splice is simplest (no new macro, no ordering concern, one place that has the
concrete `fn-forms` output).

**The fn-forms output shape (grounded — `scratchpad/probe-fnforms-shape.wat`).** For a work-fn value,
`(fn-forms work-fn :bracket::__pool-work)` returns a `Vector<WatAST>` whose LAST element is the
`__pool-work` define wrapping the concrete-typed fn:
```
[(:wat::core::defn :my::double [n <- :wat::core::i64] -> :wat::core::i64 …)   ; captured transitive dep
 (:wat::core::def  :bracket::__pool-work (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 …))]
```
The concrete type keywords are **literal AST nodes**: the fn form's argspec `[n <- :T]` carries the ARG
type keyword (the node after `<-`), and `-> :R` carries the RETURN type keyword. So the parent extracts
both by walking the last element's fn form (`ast->children`: def → fn → argspec[after `<-`] + [after `->`]).
Grounded reflection alternatives (all reflect the concrete work-fn VALUE the parent holds):
`(:wat::runtime::return-type-of work-fn)` → the return FQDN as a String (`"wat::core::i64"`, prepend `:`
for the keyword); `(:wat::runtime::extract-arg-types work-fn)` → `Vector<HolonAST>` of the arg types;
`(:wat::runtime::signature-of-fn work-fn)` → the full signature AST.

**NEXT (build on resume):** a disconfirming probe first — extract the two concrete type keywords off a
`fn-forms` result (AST-walk the last element's fn form, or reflection), splice them into a shipped process
runner's `self-peer`/`Peer'` tuple types, drain it (`"6 10"`). Then re-scope `spawn-runner`'s ProcessOpts
impl to that. AST-walk is most direct (the keywords are literal nodes); reflection is the fallback.

## Blocker B — deporder mis-attributes `extend-type` as a def-site (fix IN FLIGHT)

`deporder`'s `is-def-head?` (`deporder.wat:83`) lists `extend-type`, and `defined-name` (`:112`) takes
**child[1]** as the defined symbol. For `(extend-type :TargetType :Surface …)`, child[1] is the type
being EXTENDED — a REFERENCE, not a definition. So a cross-file extend-type (bracket.wat extending
`:wat::spawn::ThreadOpts`, defined in the earlier-loading spawn.wat) makes deporder record a phantom
def-site for `ThreadOpts` in bracket.wat → spawn.wat's earlier legit use looks like a forward-reference
→ a phantom spawn↔bracket cycle (22 `verify-stdlib` violations). Latent until now because every existing
extend-type sat in the SAME file as its target's `defstruct`.

**Fix (background agent, uncommitted, being weighed):** `extend-type`'s child[1] must be a REFERENCE,
not a def-site (primary: drop `extend-type` from `is-def-head?` so its children are collected as refs;
surgical fallback if that regresses). Proven 22→0 with the WIP + 0 on the clean stdlib. `wat/deporder.wat`
only.

## Resume plan (when deporder lands green)
1. Weigh + commit the deporder fix (own re-run: `verify-stdlib` empty, floor 0-new).
2. Unstash the S3b WIP (`git stash pop`) — the thread arm is sound; the process arm's ProcessOpts impl
   is the part to rebuild via the parent-side AST-splice (Blocker A resolution).
3. Prove the AST-splice with a disconfirming probe, then rebuild the ProcessOpts impl.
4. Gate: `scratchpad/probe-s3-bracket-loci.wat` → `[2 4 6 8 10] [2 4 6 8 10]` (thread pool AND process pool);
   existing bracket tests green; floor 0-new. NB: the acceptance probe has a typo — `:wat::core::edn::write`
   should be `:wat::edn::write` (the registered verb, `edn_shim.rs:63`).
5. Then the arc's final movement: **293 revoke-at-reap** — a process-bracket pool ∘ a long-lived service,
   grant-on-enter / revoke-at-reap (the bracket's drain-and-join IS the reap); revoke verb
   (`Admin::DenyPeer[pids]` + serve arm + `<svc>/revoke`) is the symmetric mirror of the landed grant.

## Do-nots
- Do NOT hardcode `i64` into `spawn-runner`'s process arm to force the acceptance gate — that papers over
  Blocker A; derive the concrete types from the reified work-fn.
- Do NOT fn-forms the index-wrapping `wf` closure (captures a fn → slice-1 gap); fn-forms the RAW work-fn.
- Do NOT put the `spawn-runner` extend-type impls in a file that loads before the deporder fix lands
  (the phantom cycle); the impls belong in bracket.wat (they need `runner-loop`), legal once deporder is fixed.
