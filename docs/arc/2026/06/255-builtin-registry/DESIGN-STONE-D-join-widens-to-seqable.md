# DESIGN — STONE D: `join` widens to `Seqable`, and the chain's last rung before the string home

> Stone **D** of `CHAIN-rendering-before-the-string-home.md`. Both of D's prerequisites shipped;
> this is the widening they were for. **E is next, and E is the string home this detour began at.**

## The gap, measured — not read

Run this session against `target/release/wat` (a Vector `map`ped to a Stream, then joined):

```
:wat::core::string::join: parameter #2 expects (:wat::core::Vector :- [:?2046]);
                          got (:wat::stream::Stream :- [:wat::core::i64])
```

**It fails at CHECK time.** The registered scheme refuses a Stream before the runtime arm is ever
reached — so this is **TWO sites**, not the one the chain implies.

Positive control, same session: `(join "-" (Vector :- [i64] 1 2 3))` → `"1-2-3"`. Non-strings render
with no caller-side `str`. **279.3's half of D is genuinely done**; only the container is narrow.

## Why the prerequisites are actually met

| the chain's blocker | disposition, grounded |
|---|---|
| *"`join` needs a type-variable bound wat has no form for"* | Dissolved by **C** (`str` total, `25d9d015`) — with a total renderer there is nothing left to bound `T` by. That entry is about ordering D before C; **it was never about param-spec.** |
| `Seqable` as a nameable type | **LIVE** — `wat/seq.wat:75`, `(:wat::core::defsurface :wat::core::Seqable :- [T] …)`, extended to Vector · PersistentVector · List · Stream (`:81-90`). |
| a native fn that walks any seqable | **SHIPPED at 118.B6** — `wat/seq.wat` records it: *"the native `foldl` walks any seqable — so there is nothing left to hand-walk."* |

## The one contract decision, pinned

**`join` is TERMINAL and it CONSUMES.** Ruled already; do not re-open. The chain: *"Terminal ops
consume — a Stream handed to `join` is consumed, which is not a problem to solve"* (builder: *"if the
user passes it a stream, its consumed - why is this confusing?"*). `Stream` is **read-once**; wat does
not cache (builder, 2026-08-17: *"wat does not cache... its read once"*). A single pass is the whole
contract.

**And the Vector fast path is PRESERVED, deliberately.** `Value::Vec` keeps its direct iterator and
never routes through the stream normaliser. This is 118.B7's own documented discipline, verbatim from
`wat/seq.wat`: *"There is NO `(Seqable/seq coll)` normalisation here, deliberately: that would force
every eager reduce onto the lazy path for a Stream it never needed."* Copy that, or D costs
performance it has no reason to cost.

## The rooms — every name below was signature-checked this session

1. **`src/check.rs:17534`** — `join`'s registered `TypeScheme`. `type_params: ["T"]`, params
   `[String, Parametric{head:"wat::core::Vector", args:[T]}]`. **The check-time refusal lives here.**
2. **`src/string_ops.rs:483-492`** — `eval_string_join`'s element door, the whole runtime gap:
   `match … { Value::Vec(items) => items, other => TypeMismatch{expected:"(Vector :- [T])"} }`.
3. **`src/collection/transform.rs:1187`** — `seqable_value_to_stream(coll, op, span)`. The
   **value-level** normaliser, factored out expressly *"so `filter` can COMPOSE through it on an
   already-evaluated `Value` instead of re-deriving the same container walk."* `join` holds an
   already-evaluated `Value`. This is the door.
4. **`src/collection/transform.rs:709-768`** — `eval_stream_to_vec`'s drain loop: `stream::realize`
   → `match Stream::Empty | Cons{head,tail}`. The walk shape to copy. Note it is AST-level; `join`
   needs the same loop over an `Arc<Stream>` it already holds.
5. **`src/string_ops.rs:513`** — `render_str_total(item, types)`. Already total (279.3). Unchanged.

## The sketch

```rust
let pieces_owned: Vec<String> = match eval(&args[1], env, sym)?.value_owned() {
    // FAST PATH — unchanged. Eager containers never touch the lazy path (118.B7).
    Value::Vec(items) => items.iter().map(|i| render_str_total(i, types)).collect(),
    // WIDENED — any other Seqable: normalise once, then render per element as we walk.
    other => { /* seqable_value_to_stream(other, OP, args[1].span())? then realize-loop,
                  render_str_total on each head. Single pass; nothing materialised. */ }
};
```

Rendering **per element during the walk** means a Stream is joined without ever building an
intermediate `Vec` — strictly better than the eager path, not a concession.

## Out of scope — affirmatively cut

- **`length`/`empty?` on a Stream.** `118/DESIGN-118.4` is SUPERSEDED IN PART on exactly these; the
  live defect (`length` type-checks then RAISES) is arc 118's, tracked in
  `118/UX-118.7-the-user-forms-and-a-correction-to-118.4.md`. Not D's.
- **`concat`.** The chain: *"`string::concat` is String→String and `Vector/concat` is
  Vector→Vector — genuine receiver dispatch and a separate question."*
- **Stone E** (`wat.string/*`, 1,617 sites, then home #4). D unblocks it; D is not it.

## ⚠ The trap a rider handed the chain would fall into

The chain's D section sketches `(seq [self] :- (wat.type/Seq [T]))`. **`wat.type/Seq` does not exist
and never did** — the chain says so itself. And its redirect target, `118/DESIGN-118.4`, is **itself
superseded in part** by `118/UX-118.7`. That is a two-hop redirect ending in partly-wrong text. The
authoritative surface is on disk: `wat/seq.wat:75-92`. Read the code, not either sketch.
