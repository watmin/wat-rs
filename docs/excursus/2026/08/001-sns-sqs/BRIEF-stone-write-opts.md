# BRIEF — excursus 001 stone WRITE-OPTS: serialization options are a VALUE the caller passes

**Builder's ruling 2026-08-30.** Not a global, not a fixed default, and **not a `digits`
parameter**:

> *"write-json v digits … this is only applicable for timestamps … we need a write config or
> something that's passed in … with a default being a sane one you basically never need to touch"*
>
> *"this should be a point in code decision.. not a global.... the caller chooses to use full
> precision... or whatever precision they want at serialization time"*

## The work, in one paragraph

Stone INST made `#inst` render at constant nanosecond width, which is **correct and mandatory
for EDN** because `:wat::query::Store/scan` orders by the `sk` *string*. It is not obviously
right for JSON, which this project emits only because the outside world demands it. `json.rs`
was left at `AutoSi` on the reasoning *"for no consumer that asked"* — **no consumer was
asked**, and that assumption should not be frozen into the substrate in either direction.
Instead: mint a `WriteOpts` struct with a sane default, and let the JSON writers take one.

## Why a struct and not a `digits` argument

Fractional-second precision is a **timestamp** concern. Putting it in a general serializer's
signature puts the wrong thing on the wrong axis — the next rendering choice (float format, key
ordering, map sorting) makes it `write-json v digits float-fmt key-order`. A struct absorbs
each of those as a **field**, with no signature churn and one place to read what is configurable.

## The precedent is already in the tree — copy it exactly

`wat/spawn.wat` — and this excursus's own SNS demo uses both halves:

```wat
(:wat::core::defstruct :wat::spawn::ProcessOpts [...])              ;; :77

(:wat::core::defn :wat::spawn::process [] -> :wat::spawn::ProcessOpts   ;; :122 — zero-arg DEFAULT
  (:wat::spawn::ProcessOpts …defaults…))

(:wat::core::defn :wat::spawn::process/post-spawn [f] -> …)         ;; :130 — customize ONE thing
```

`(:wat::spawn::process)` for the common case; `(:wat::spawn::process/post-spawn f)` when you
care. Mirror that shape.

## Read in order

1. **`wat/spawn.wat:77`, `:122`, `:130`** — the struct, the zero-arg default constructor, and a
   named single-field variant. This is the pattern; do not invent another.
2. **`src/intrinsic/edn.rs:183` (`write-json`) and `:207` (`write-json-natural`)** — the two
   verbs that take opts.
3. **`src/intrinsic/edn.rs:140` (`write`)** — **DO NOT TOUCH.** See out-of-scope.
4. **`crates/wat-edn/src/writer.rs:227` and `crates/wat-edn/src/json.rs:170`** — where the
   precision is actually applied. The crate needs a `write_json_with(v, opts)`; `write_json(v)`
   delegates with the default, so the 88 `write` call sites see nothing.
5. **`src/intrinsic/time.rs:208` (`to-iso8601`)** — prior art for clamping: *"fractional-second
   digit count, clamped to [0, 9]"*. Clamp the same way, at the same bounds.

## Implementation sketch — you fill it

```wat
;; wat/edn.wat (or wherever the edn surface's wat-side declarations live — FIND IT, do not
;; assume; `:wat::edn::` verbs are Rust intrinsics today and the struct may need a home)
(:wat::core::defstruct :wat::edn::WriteOpts
  [inst-digits <- :wat::core::i64])          ;; clamped [0,9]

(:wat::core::defn :wat::edn::opts [] -> :wat::edn::WriteOpts
  (:wat::edn::WriteOpts :inst-digits 9))     ;; the sane default you never touch

(:wat::core::defn :wat::edn::opts/inst-digits [n <- :wat::core::i64] -> :wat::edn::WriteOpts …)
```

```wat
(:wat::edn::write-json v (:wat::edn::opts))                  ;; normal
(:wat::edn::write-json v (:wat::edn::opts/inst-digits 3))    ;; when the outside world forces it
```

## Blast radius

- `crates/wat-edn/` — a `WriteOpts`-equivalent + `write_json_with`; `write_json` delegates
- `src/intrinsic/edn.rs` — `write-json`, `write-json-natural` take the opts arg
- wherever the `:wat::edn::` wat-side surface lives — the struct + two constructors
- **23 `write-json` / `write-json-natural` call sites** — census command:
  `grep -rn ':wat::edn::write-json' --include=*.wat --include=*.rs . | grep -v src/intrinsic/edn.rs`
- `crates/wat-edn/docs/USER-GUIDE.md:336` — **already false** since stone INST; fix it here
- this excursus's SCORE

## STOP triggers

1. **If `:wat::edn::write` (the 1-arg EDN verb) needs to change — STOP.** It has 424 call sites
   and it is the Store sort-key path. Its width is a **correctness invariant**, not a
   preference, and a caller who can weaken it is a caller who can silently drop rows from a
   range scan. If opts cannot be added to the JSON verbs without touching it, that is a finding.
2. **If you reach for a global, a config knob, or a thread-local — STOP.** That was the
   rejected design, and it is a footgun: one setting, and every `StoredRow` written afterwards
   loses its ordering. An argument at the call site cannot do that.
3. **If the struct's home is not obvious** (the `:wat::edn::` verbs are Rust intrinsics; there
   may be no wat-side `edn.wat`) — STOP and report where you think it should live rather than
   creating a new stdlib file on your own judgement.
4. **If the floor reds on anything but the known journal arm** — STOP, capture whole, do NOT
   re-run.

## Out of scope, affirmatively cut

- **`:wat::edn::write`** — stays 1-arg, nanos, no opts. STOP-1.
- **`write-pretty`** — same reasoning as `write` unless you find a caller that needs otherwise.
- **The journal `SortKey`** and **deleting `time-sk`** — both still owed, both elsewhere.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

Floor here is **5103 with ONE known failure** — `probe_arc278_span_macros`, the journal
key-collision arm. **That red is expected and is NOT yours.** Two failures means you added one.
