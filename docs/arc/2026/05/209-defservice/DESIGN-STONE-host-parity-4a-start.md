# DESIGN — Stone host-parity-4a: the host-agnostic `start [host <- :Host]`

> Opened 2026-06-15. C.3 shipped a THREAD-HARDCODED `start [state0]` (bakes `(:wat::spawn::thread)`
> into `listener'` + `spawn-program'`, service.wat:500-509). 4a makes `start` take the host first and
> route the per-tier launch through the `:wat::spawn::Host` protocol (arc 232). Probe (RED, committed
> `ff09e85b`): `tests/probe_arc209_host_agnostic_start.rs` — `(:my::counter/start (:wat::spawn::thread) 0)`
> → round-trip → 5. Grounded against HEAD `c30ef004`.

## What is actually host-specific (the crawl result)

In the C.3 `start` body, only TWO things touch the host, and `listener'`/`spawn-program'` already
dispatch on the host's runtime type:
1. `(listener' host Op Reply)` → `Bound<Op,Reply>` (the listening state).
2. `(spawn-program' host <prog>)` → `Spawned` handle — where `<prog>` is the THREAD-ONLY capturing
   closure `(fn [self] (serve self l (Vector peer) state0))`. **This closure is the whole problem:**
   thread captures (shared memory); process (4b) needs `Vector<WatAST>` forms. A closure can't cross a
   fork ([[project_shared_memory_partition_hosting]]).

## The hard constraint (grounded, not assumed)

`infer_listener_prime` (check.rs:10322-10398) dispatches on the host type at CHECK time: host must
reduce to `:wat::spawn::ThreadOpts` or `:wat::spawn::ProcessOpts`, ELSE **TypeMismatch** (10385).
So a `start` body that calls `(listener' host …)` with `host : :wat::spawn::Host` (abstract) **fails
the checker**. The host-specific kernel calls must run where the type is CONCRETE.

## Validated mechanism (PROVEN this session)

The tier-neutral way for a GENERIC launcher to invoke a PER-SERVICE `serve` fn: pass `serve` as a
runtime-built keyword (`keyword/from-string`, which stays `keyword`-typed — a literal `:my::serve`
resolves to a `Fn` value via Arc-009 and would NOT cross as data), and invoke it via the arc-232
`:wat::core::apply` primitive (`eval_apply`, runtime.rs:7560 — `<head>` is evaluated; a runtime keyword
dispatches through the full name-lookup chain). Proven end-to-end (`/tmp/host_proto_probe.wat` → `42`):
```wat
(:wat::core::defprotocol :wat::spawn::Host
  (run-it [self <- :wat::spawn::Host  f <- :wat::core::keyword] -> :wat::core::i64))
(:wat::core::extend-type :wat::spawn::ThreadOpts :wat::spawn::Host
  (run-it [self f] (:wat::core::apply -> :wat::core::i64 f [])))
;; (:wat::spawn::Host/run-it (:wat::spawn::thread) (keyword/from-string "my::compute")) => 42
```
The keyword is the universal `serve` reference: thread captures-and-applies; process (4b) ships forms
that apply the same keyword. This is what makes "new transport = zero central edit" true.

## The candidate design (two pieces)

**Piece 1 — teach `listener'` to accept an abstract `:wat::spawn::Host`.** Extend
`infer_listener_prime`: if the host type reduces to `:wat::spawn::Host` (the protocol marker) with the
3-arg `(host :S :R)` shape, return `Bound<S,R>` from the LITERAL `:S`/`:R` args (still passed by
`start`, where Op/Reply are known at defservice expansion). Runtime `eval_listener_prime` already
dispatches on the concrete value (`start`'s caller passes a real `ThreadOpts`), so only the checker
needs the new arm. This keeps the typed `Bound<Op,Reply>` WITHOUT routing `listener'` through a
generic method (which would hit the typed-parametric-return-in-a-generic-method problem — see Rejected).

**Piece 2 — ONE protocol method `:wat::spawn::Host/spawn` absorbing the prog-build + spawn.** Returns
plain `:wat::spawn::Spawned` (NON-parametric → no typed-return-in-generic-method problem). The thread
impl builds the capturing closure + calls `spawn-program' self` (self concrete = ThreadOpts inside the
impl, so the kernel dispatch is concrete). Process (4b) = a not-shared `extend-type` building forms,
SAME method signature.

`start` (defservice codegen) becomes:
```wat
(defn <fqdn>/start [host <- :wat::spawn::Host  state0 <- <state-ty>] -> <fqdn>::Handle
  (let [b    (listener' host :Op :Reply)                 ; Piece 1: listener' accepts abstract Host
        l    (Bound/listener b)
        addr (Bound/address b)
        svc  (:wat::spawn::Host/spawn host l (Vector peer-ty) state0
                (keyword/from-string "<serve-fqdn>"))]   ; Piece 2: protocol absorbs the prog
    (Handle svc addr)))
```

## The gap — PROBED, and it is a real substrate BLOCKER (2026-06-15)

Piece 2's `Host/spawn` must be generic over BOTH `S` and `R` (the `Listener'<S,R>` param + the
`Peer'<R,S>` closure it builds). Two isolated probes settled it:

- **`(sp<S,R> …)`** — multi-param generic method name **does not parse**: *"name keyword `sp<S` opens
  '<' but does not close '>'"*. arc-232 / arc-232-generic-method only built SINGLE-param `<T>` method
  names. (Generic FNS already parse multi-param `<A,B>` — `foldl<T,Acc>`, `map<I,O>` — so the splitter
  exists; the defprotocol method-name parser just doesn't use it.)
- **implicit `S,R`** (free in the sig, no `<…>` suffix) — **parses but doesn't instantiate**:
  `parameter #2 expects Listener'<S,R>; got Listener'<i64,i64>` (S,R treated as literal `Path(":S")`,
  exactly the monomorphic behavior arc-232-generic-method documented).

**Conclusion: the clean Piece 2 is blocked on a deferred dep — MULTI-TYPE-PARAM GENERIC PROTOCOL
METHODS** (`(method<S,R> …)` in defprotocol). Block-and-build it first
([[feedback_deferred_dep_becomes_necessary_block_and_build]]): extend the defprotocol method-name
parse to reuse the multi-param `<A,B>` splitter generic fns already use (+ instantiate both at the
call site, mirroring the single-`<T>` arc-232-generic-method work). THEN 4a-ii is buildable.
(One thing left to try when building the dep: a BARE closure param `(fn [pp] …)` inferred from
spawn-program''s thread-clause expectation — might let the impl avoid naming R,S — but the METHOD
sig still needs `<S,R>` for the `Listener'<S,R>` param, so the dep stands.)

## Rejected

- **`Host/listen` as a generic method** returning `Bound<S,R>`: the method has no value arg carrying
  S/R, so they'd need explicit type-args at the call AND the impl would pass type-params to the
  `listener'` intrinsic — two unproven mechanisms. Piece 1 (listener'-accepts-abstract) is strictly
  smaller. CUT.
- **Forms-unified prog for both tiers** (thread also runs forms): erases the shared-memory benefit the
  thread tier exists for (capture is cheap/direct); would need a forms-based thread spawn (unbuilt).
  CUT — the partition keeps thread=capture, process=forms.

## Decomposition (sub-stones)

- **4a-i** — probe + build Piece 1 (`infer_listener_prime` accepts abstract `:Host`). Smallest;
  de-risks the listener' path.
- **4a-ii** — probe the typed-closure-in-generic-method gap, then build `:wat::spawn::Host` +
  `Host/spawn` + the ThreadOpts impl.
- **4a-iii** — rework defservice `start` codegen → `[host <- :Host  state0]`; the committed probe
  (`probe_arc209_host_agnostic_start`) goes GREEN. Fix the probe's stale `:wat::kernel::Host` doc
  comment → `:wat::spawn::Host`.

Pairs [[project_shared_memory_partition_hosting]] + [[feedback_deferred_dep_becomes_necessary_block_and_build]]
+ [[feedback_reach_stumble_is_the_signal]].
