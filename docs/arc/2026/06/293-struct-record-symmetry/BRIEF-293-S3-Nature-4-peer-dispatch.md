# BRIEF — 293 S3-Nature-4 (Path B): a `:nature :Peer` surface intrinsically dispatches to its peer

> **Executor: one sonnet SHADOWDANCER.** A **Rust** strike with TWO parts (a checker rule + a runtime forwarding).
> Do PART 1 first, verify its checkpoint, THEN PART 2. If PART 2's runtime forwarding proves tangled, STOP after PART 1
> + report (I will re-scope PART 2). Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; `.claude/worktrees/`
> illegal). `cargo build`; `./target/release/cargo-wat <f>`; `cargo nextest run --release` (NEVER `cargo test`).
> **Commit NOTHING.** Runs on S3-Nature-2 (`23e8c16f`) + S3-Nature-3 (`b2deb815`).

## The idea (278 R32 — "a service is a surface at a coordinate"; the builder's insight)

`send'`/`recv'` are ALREADY polymorphic over any `Peer'<S,R>` — "how to talk to a peer" is a solved substrate
primitive. The only surface-specific parts are WHICH `Op` variant to send and WHICH `Reply` variant to match — and
those come from S1's synthesis (`<S>::Op` / `<S>::Reply`). So a `:nature :Peer` surface's method dispatch is just a
**composition**: `send' peer (S::Op::<M> req); (match (recv' peer) ((S::Reply::<M> resp) resp))`. **No `extend-type`,
no per-peer-type impl** — the surface composes the generic peer ops with its own Op/Reply. This supersedes the
extend-type peer path (S3a/S3-Nature-2/-3 hardened the general substrate; this is the CORRECT mechanism for peers).

The RED probe `scratchpad/s3-probe-peer-satisfies.wat` (NO extend-type now) fails today at the receiver check; it must
round-trip after this stone.

## PART 1 — the checker rule (`src/check.rs`, `assignable` ~14905, BEFORE the Parametric→Path edge branch)

A `Peer'<X,Y>` is assignable to a `:nature :Peer` surface `:S` iff `X == :S::Op` and `Y == :S::Reply` (the surface's
S1-synthesized enums). Add this branch at the TOP of `assignable` (after `a`/`e` are reduced, before the existing
Path-Path / Parametric-Path branches):

```rust
// Path B — a dialed peer intrinsically satisfies a :nature :Peer surface (no extend-type).
if let (TypeExpr::Parametric { head, args: peer_args }, TypeExpr::Path(ep)) = (&a, &e) {
    if head == "wat::kernel::Peer'" && peer_args.len() == 2 {
        if let Some(crate::types::TypeDef::Surface(surf)) = types.get(ep) {
            if surf.nature == Some(crate::types::Nature::Peer) {
                let want_op    = reduce(&TypeExpr::Path(format!("{}::Op", ep)),    subst, types);
                let want_reply = reduce(&TypeExpr::Path(format!("{}::Reply", ep)), subst, types);
                let got_op     = reduce(&peer_args[0], subst, types);
                let got_reply  = reduce(&peer_args[1], subst, types);
                return got_op == want_op && got_reply == want_reply;
            }
        }
    }
}
```
(Confirm `types.get(ep)` returns the `Surface` and `surf.nature` is the `Option<Nature>` field. `:S::Op`/`:S::Reply`
are the exact FQDNs S1 synthesizes — verify against `scratchpad/s1-reference-target.wat` / S1's `format!("{}::Op", surface.name)`.)

**PART 1 checkpoint:** `./target/release/cargo-wat scratchpad/s3-probe-peer-satisfies.wat` → the receiver-check error
(`expects :probe::Kv; got Peer'<…>`) is GONE; it now fails at RUNTIME (`does not implement surface method` / dispatch).
That is the correct intermediate state — proceed to PART 2.

## PART 2 — the runtime forwarding (`src/runtime.rs`, the surface-method dispatch ~5461-5510)

At the dispatch site, AFTER confirming `:S` is a `Surface` with method `method_name` and BEFORE deriving
`concrete_type_fqdn` / the `:<T>/<method>` lookup: if the surface's nature is `:Peer`, COMPOSE the peer ops instead of
looking up a satisfier impl. Build the forwarding expression as a `WatAST` and `eval_inner` it, reusing the call's own
arg ASTs (`args[0]` = the peer expr; `args[1]` = the request expr):

```clojure
;; the expression to construct + eval (for method `put`, variant `Put` = PascalCase(method_name)):
(:wat::core::let
  [__op (:<S>::Op::<Variant> <args[1]>)          ;; construct the Op via its registered constructor
   __   (:wat::kernel::send' <args[0]> __op)
   __r  (:wat::kernel::recv' <args[0]>)]
  (:wat::core::match __r -> <method-ret-type>
    ((:<S>::Reply::<Variant> resp) resp)))         ;; the protocol guarantees this variant; a 1-arm match is correct
```
- `<Variant>` = PascalCase(method_name) via the EXISTING `crate::string_ops::kebab_to_pascal_with_acronyms(method_name, &[])`
  (S1/S2 use it; it is `pub(crate)`).
- `<S>` = `protocol_fqdn` (the surface FQDN already in scope at the dispatch site).
- `<method-ret-type>` = the surface method's `ret` (from the matched `SurfaceMember::Method`), for the `match` ascription.
- Build the `WatAST::List` nodes with the call's `list_span`; then `eval_inner(&forwarding_ast, env, sym)` and return
  its value.

Read `eval_peer_send_prime` (runtime.rs:5350) + `eval_peer_recv_prime` (5543) + how enum constructors resolve
(`:<S>::Op::<Variant>` is a registered constructor, runtime.rs:2396) to confirm the composed AST evals cleanly. If
constructing + evaling the AST proves too tangled, STOP after PART 1 and report — do NOT ship a half-working runtime.

## Read the rooms, in order
1. `src/check.rs:14896-14920` — `assignable` (PART 1 insertion point; the reduce helpers).
2. `src/types.rs:129-186` — `Nature` (confirm `Nature::Peer`) + `SurfaceDef.nature: Option<Nature>` (grep the field).
3. `src/runtime.rs:5461-5510` — the surface-method dispatch (PART 2; `protocol_fqdn`, `method_name`, `args`, the
   Surface/Method lookup already present).
4. `src/runtime.rs:5350`,`5543` — `eval_peer_send_prime`/`eval_peer_recv_prime` (confirm the composed send'/recv' evals).
5. `src/runtime.rs:2396` — enum constructor registration (`:<S>::Op::<Variant>` is callable).
6. `scratchpad/s3-probe-peer-satisfies.wat` — the RED probe (NO extend-type). Fails today at the receiver check; must
   round-trip after.

## STOP triggers (halt + report, do NOT hack)
1. **STOP-PART2-TANGLED:** if PART 1 lands (receiver check clears) but PART 2's runtime AST-construction/eval is
   genuinely tangled or fragile, STOP after PART 1 and report exactly where — I will re-scope PART 2 as its own stone.
2. **STOP-REGRESSION:** the whole floor must stay green modulo the known lint. If any pre-existing test changes
   behavior (esp. aggregate surface dispatch — a `:nature :Struct` surface must dispatch to its `:<T>/<method>` impl
   exactly as before), STOP and report — the PART 2 branch must fire ONLY for `surf.nature == Some(Nature::Peer)`.
3. **STOP-NOCP:** do NOT change S1 synthesis, `send'`/`recv'`, the enum-constructor machinery, or the extend-type path.
   PART 1 is one `assignable` branch; PART 2 is one branch in the dispatch site.

## The gate (EXPECTATIONS — the orchestrator re-runs these)
| what | command | expected |
|---|---|---|
| PART 1 — receiver check clears | `cargo wat` on the probe, after PART 1 | no `expects :probe::Kv` error; fails at runtime dispatch |
| the peer round-trips (no extend-type) | `./target/release/cargo-wat scratchpad/s3-probe-peer-satisfies.wat` | prints `peer-as-Kv put ok = true` / `peer-as-Kv get alpha = one` |
| aggregate surface dispatch unchanged | `cargo nextest run --release -E 'test(smem_roundtrip) or test(sqlite_store_differential) or test(nature) or test(counter)'` | passed (byte-identical) |
| whole floor | `cargo nextest run --release` | verbatim Summary; `0 failed` modulo the known `no_inlined_wat_in_tests` reminder |

Runtime ~40-60 min (a Rust change + rebuild + the suite; PART 2 is the harder half).

## Final report (structured): the exact diff (PART 1 branch + PART 2 branch) · the PART 1 checkpoint result · the
verbatim gate results (the round-trip + the targeted tests + the whole-floor Summary) · STOP triggers hit or "none" ·
did the peer round-trip FULLY (no extend-type) · anything that surprised you.

## Prior comparable: S1 (`b13cab8c`, surface synthesizes Op/Reply — the sibling "the surface generates its wire forms")
+ S3-Nature-2/-3 (`23e8c16f`/`b2deb815`, the :Peer nature + the edge). This is S3b folded into the substrate.
