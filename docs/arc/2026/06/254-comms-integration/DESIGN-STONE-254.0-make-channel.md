# DESIGN — Stone 254.0: `make-channel`, the one canonical channel constructor

**Pulled forward from the deferred "254.5 annihilate".** The builder's cut:
depth is *always 1*, so the word "bounded" and the capacity argument `N` are both
noise. The whole channel-construction surface collapses to one honest verb.

> Mini-TCP at depth 1 (doctrine 2026-05-19; arc 254 §contract): capacity-1,
> send-one-read-back, lock-step. There has never been a legitimate second channel.

## The canonical end-state

```
(:wat::kernel::make-channel :T)   →  (Sender<T>, Receiver<T>)   ; depth ALWAYS 1
```

One verb. One arity (the payload type keyword). No capacity argument. The return
type stays `:wat::kernel::Channel<T>` (the `derive_type_ann_from_rhs` projection).
Wrong-capacity is **unrepresentable** (✅✅✅): there is no `N` to pass, so there
is no runtime gate and no checker gate to maintain.

## ⚰️ Ripped from existence (the kill — none conceded)

- **`make-unbounded-channel`** — unbounded violates depth-1. Verb, runtime
  dispatch (`runtime.rs:4089`), `eval_make_unbounded_queue` (`runtime.rs:17929`),
  `typed_channel::unbounded()` (`typed_channel.rs:603`), the check arms
  (`derive_type_ann_from_rhs` 3035, infer arm 4837), verb-list entry (1926),
  anchor-binding alternative (2883/2957).
- **`make-bounded-channel`** — the *name* and the `N` capacity argument: the
  capacity-parse + non-negativity validation block (`runtime.rs:17900-17915`),
  the `with_capacity=true` path. Collapses to `make-channel`.
- **`make-bounded-queue` / `make-unbounded-queue`** — PHANTOM verbs. They already
  error at the wat surface (probe D/E green at HEAD) but their corpse-code still
  litters `check.rs`: verb-list entries (1921/1925), infer arms (4775/4792), the
  `infer_make_queue` doc + `with_capacity` param (10458). Headstones that lied.
- **`eval_make_*_queue` / `infer_make_queue` fossil NAMES** — channels were never
  queues; the misnomer dies with the collapse (rename → `*_channel`).

## 🔥 Raised (the rebirth)

- **`make-channel`** — the one canonical depth-1 constructor — minted in this
  stone. **254.0b (next beat):** lift it out of the `runtime.rs` (31k) +
  `check.rs` (19k) quarries into a warded home **`src/channel/`** + vigilia +
  vigilatum stamp. That home is the **seam** where 254.2/254.3 later swap the
  backing from `typed_channel`(crossbeam `bounded(1)`) to `comms` — without the
  wat surface ever knowing the channel changed underneath it. (`typed_channel::
  bounded()` STAYS — it backs `make-channel(1)` until comms takes over.)

## The migration cascade (~142 sites, substrate-as-teacher)

`make-bounded-channel` (106 occ) + `make-unbounded-channel` (36 occ) → `make-channel`:
- **stdlib wat**: `wat/kernel/channel.wat`, `services/{stdin,stdout,stderr}.wat`, `stream.wat`
- **corpus wat**: `wat-tests/{service-template,counter-service-thread-N1,-N3,counter-service-capability-N3}.wat`
- **rust tests**: `tests/{wat_arc170_closure_extraction,wat_stream}.rs`, `tests/types/typealias.rs`, `tests/nursery/probe_arc254_channel_payload_portable.rs`, + the `check.rs` inline test fixtures (18127+)
- drop the `N` arg at every `make-bounded-channel :T N` site (every observed N is 1)

## Discriminations (do NOT conflate)

1. **`N` removal**: `make-bounded-channel` was arity-2 (TYPE N); `make-channel` is
   arity-1 (TYPE). Remove `N` at every call site.
2. **value-position nil is a SEPARATE migration**: do not "fix" channel tests by
   touching their `:wat::core::nil` value-position fixtures beyond what the
   make-channel rewrite requires; that nil sweep is its own track (#183).
3. **`bounded()` lives**: only `typed_channel::unbounded()` dies. `bounded()`
   backs `make-channel`'s `bounded(1)`.

## Probe (RED at HEAD → GREEN after)

`tests/nursery/probe_arc254_make_channel.rs` (committed, `#[ignore]`'d):
- A `make_channel_is_the_one_constructor` — make-channel checks OK (RED: absent today)
- B/C `condemned_channel_verbs_are_annihilated` — unbounded + bounded(N) no longer resolve (RED: alive today)
- D/E `phantom_queue_verbs_are_annihilated` — already green (phantoms already error); regression guard
Sonnet un-ignores all three after the cascade; they flip GREEN.

## Gates

lib 940/0/1 held; `cargo build --release --tests` green; probe A/B/C GREEN
un-ignored; full corpus (integration-run) green; no `make-unbounded-channel` /
`make-bounded-channel` / `make-*-queue` token survives (grep = 0, the three-nil
scoreline). THE-TALLY: every site migrated, none conceded.
