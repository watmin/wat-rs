# FINDING — generic service lifecycle (option E), and the one type-checker gap it hits

**Measured 2026-09-23 by the orchestrator at `c2e2ed15c`**, before any stone is drawn, per the builder:
*"measurements first — ensure we know what we are doing."* Context: the ruling *exactly one way to do
things* chose option E — one generic `wat.service/start` dispatched on the service, instead of a
`svc/start` minted per service. The builder added: *"we have thread and process locus now — soon(tm)
we have various networked locus of various kinds of connectivity."*

Every probe below was run on `target/release/wat` built from that tree, and each negative control was
shown to be able to fail. The full probe text is at the end so any hand can re-run them.

## ✅ 1. The SERVICE axis works today

A parametric surface `Svc :- [H St]` whose method takes the bound `St` and returns the bound `H`, with
**two implementors binding different types**:

| | result |
|---|---|
| both implementors, each call to its own types | `--check` **rc=0**; run prints `"42"` (A) and `"handle-b"` (B) — dispatch picks each |
| ⛔ claim A returns `String` (bound to `i64`) | **rc=1 `ReturnTypeMismatch`** |
| ⛔ pass A an `i64` state (bound to `String`) | **rc=1 `TypeMismatch`** |

⭐ **One generic method, each implementor's input and output types bound and enforced both ways.**

## ✅ 2. A non-parametric surface as an ordinary parameter accepts implementors

| | result |
|---|---|
| one fn, `loc <- :probe::Loc` (non-receiver), given thread then process | **rc=0**; run prints `"thread"` then `"process"` |
| ⛔ given a record that does not implement `Loc` | **rc=1 `TypeMismatch`** |

## ⛔ 3. The TRANSPORT axis — the gap

**Why it matters.** The per-service `start$impl-thread` / `start$impl-process` copies are NOT
redundant. `a6da457e3` (293.W.2f) *"a process may not dial a shared-memory address"*:
*"`Address<S,R,T>` carries Shared vs Wire. /start stamps Handle from the locus constructor. Process
map/each require-wire-address."* The locus decides the **transport type** `T`, and it is written into
the handle so that a process dialling a shared-memory address is a **type error**. Today that costs
one `start$impl-<transport>` per service — ⛔ **N × M, and every networked locus adds a column.**

For E to survive, the locus must bind `T` and a single generic `start` must carry it. **A/B, same
surface, same implementor, only the type argument changes:**

| parameter typed as | given | result |
|---|---|---|
| `(Loc :- [Shared])` — concrete | `Th` (binds `Shared`) | ✅ **rc=0** |
| `(Loc :- [Shared])` — concrete | `Pr` (binds `Wire`) | ✅ **rc=1** `expects (Loc :- [Shared]); got Pr` — the transport rule, enforced |
| `(Loc :- [T])` — **type variable** | `Th` | ⛔ **rc=1** `expects (Loc :- [_]); got Th` |

⭐⭐ **The gap, exactly: the type checker does not bind a function's type variable from the
`extend-type` binding of the implementor it is given.** `T` stays `_`. Arc 170 taught `assignable` the
*concrete* case (`tests/types/probe_arc170_parametric_surface_param.rs` — *"`assignable` had no
`(concrete-Path actual, parametric-surface expected)`"*); **the type-variable case is not covered.**

## What this means

- ⭐ **E is feasible on the service axis today** — no substrate change needed there.
- ⛔ **E on the transport axis needs ONE type-checker capability:** inferring a type variable from a
  parametric-surface implementor's binding.
- ⭐ **That same capability is the ROOT of the N × M.** Without it, `start` must be written once per
  concrete transport, per service — the per-locus copies are the stem. With it, a networked locus is
  **one `extend-type` binding its `T`**, and no service changes: **N + M.** (That the gap is *why* the
  copies were minted is consistent with `a6da457e3`'s message but is an inference about history, not a
  measurement.)
- ⚠ **Not measured:** combining both axes in one call (service binds `H`, locus binds `T`, result
  carries both); and the alias question (`<S>::<op>/Request` re-attaching surface type parameters).

## The probes (re-runnable)

### `e-base.wat`

```wat
(:wat::core::defsurface :probe::Svc :- [H St] :nature :wat::core::Struct
  :features
  [(start [self <- (:probe::Svc :- [H St]) st <- :St] -> :H)])

(:wat::core::defrecord :probe::ASvc [n <- :wat::core::i64])
(:wat::core::extend-type :probe::ASvc (:probe::Svc :- [:wat::core::i64 :wat::core::String])
  (start [self st] (:probe::ASvc/n self)))

(:wat::core::defrecord :probe::BSvc [s <- :wat::core::String])
(:wat::core::extend-type :probe::BSvc (:probe::Svc :- [:wat::core::String :wat::core::i64])
  (start [self st] (:probe::BSvc/s self)))
```

### `e-pos.wat`

```wat
(:wat::core::defsurface :probe::Svc :- [H St] :nature :wat::core::Struct
  :features
  [(start [self <- (:probe::Svc :- [H St]) st <- :St] -> :H)])

(:wat::core::defrecord :probe::ASvc [n <- :wat::core::i64])
(:wat::core::extend-type :probe::ASvc (:probe::Svc :- [:wat::core::i64 :wat::core::String])
  (start [self st] (:probe::ASvc/n self)))

(:wat::core::defrecord :probe::BSvc [s <- :wat::core::String])
(:wat::core::extend-type :probe::BSvc (:probe::Svc :- [:wat::core::String :wat::core::i64])
  (start [self st] (:probe::BSvc/s self)))

(:wat::core::defn :probe::a-start [] -> :wat::core::i64 (:probe::Svc/start (:probe::ASvc :n 42) "state-a"))
(:wat::core::defn :probe::b-start [] -> :wat::core::String (:probe::Svc/start (:probe::BSvc :s "handle-b") 7))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::str (:probe::a-start)))
    (:wat::kernel::println (:probe::b-start))))
```

### `loc-base.wat`

```wat
(:wat::core::defsurface :probe::Loc :nature :wat::core::Struct
  :features [(where [self <- :probe::Loc] -> :wat::core::String)])
(:wat::core::defrecord :probe::Th [x <- :wat::core::i64])
(:wat::core::extend-type :probe::Th :probe::Loc (where [self] "thread"))
(:wat::core::defrecord :probe::Pr [y <- :wat::core::i64])
(:wat::core::extend-type :probe::Pr :probe::Loc (where [self] "process"))
(:wat::core::defrecord :probe::NotLoc [z <- :wat::core::i64])
(:wat::core::defn :probe::run-on [svc <- :wat::core::String  loc <- :probe::Loc] -> :wat::core::String
  (:probe::Loc/where loc))
```

### `t-base.wat`

```wat
(:wat::core::defrecord :probe::Shared [a <- :wat::core::i64])
(:wat::core::defrecord :probe::Wire [b <- :wat::core::String])
(:wat::core::defsurface :probe::Loc :- [T] :nature :wat::core::Struct
  :features [(transport [self <- (:probe::Loc :- [T])] -> :T)])
(:wat::core::defrecord :probe::Th [x <- :wat::core::i64])
(:wat::core::extend-type :probe::Th (:probe::Loc :- [:probe::Shared]) (transport [self] (:probe::Shared :a 1)))
(:wat::core::defrecord :probe::Pr [y <- :wat::core::i64])
(:wat::core::extend-type :probe::Pr (:probe::Loc :- [:probe::Wire]) (transport [self] (:probe::Wire :b "w")))
(:wat::core::defn :probe::start :- [T] [loc <- (:probe::Loc :- [T])] -> :T (:probe::Loc/transport loc))
```

### `ab-concrete.wat`

```wat
(:wat::core::defrecord :probe::Shared [a <- :wat::core::i64])
(:wat::core::defrecord :probe::Wire [b <- :wat::core::String])
(:wat::core::defsurface :probe::Loc :- [T] :nature :wat::core::Struct
  :features [(transport [self <- (:probe::Loc :- [T])] -> :T)])
(:wat::core::defrecord :probe::Th [x <- :wat::core::i64])
(:wat::core::extend-type :probe::Th (:probe::Loc :- [:probe::Shared]) (transport [self] (:probe::Shared :a 1)))
(:wat::core::defrecord :probe::Pr [y <- :wat::core::i64])
(:wat::core::extend-type :probe::Pr (:probe::Loc :- [:probe::Wire]) (transport [self] (:probe::Wire :b "w")))
(:wat::core::defn :probe::start-concrete [loc <- (:probe::Loc :- [:probe::Shared])] -> :probe::Shared (:probe::Loc/transport loc))
(:wat::core::defn :probe::use [] -> :probe::Shared (:probe::start-concrete (:probe::Th :x 1)))
```

### `ab-typevar.wat`

```wat
(:wat::core::defrecord :probe::Shared [a <- :wat::core::i64])
(:wat::core::defrecord :probe::Wire [b <- :wat::core::String])
(:wat::core::defsurface :probe::Loc :- [T] :nature :wat::core::Struct
  :features [(transport [self <- (:probe::Loc :- [T])] -> :T)])
(:wat::core::defrecord :probe::Th [x <- :wat::core::i64])
(:wat::core::extend-type :probe::Th (:probe::Loc :- [:probe::Shared]) (transport [self] (:probe::Shared :a 1)))
(:wat::core::defrecord :probe::Pr [y <- :wat::core::i64])
(:wat::core::extend-type :probe::Pr (:probe::Loc :- [:probe::Wire]) (transport [self] (:probe::Wire :b "w")))
(:wat::core::defn :probe::start :- [T] [loc <- (:probe::Loc :- [T])] -> :T (:probe::Loc/transport loc))
(:wat::core::defn :probe::use [] -> :probe::Shared (:probe::start (:probe::Th :x 1)))
```

### `ab-concrete-wrong.wat`

```wat
(:wat::core::defrecord :probe::Shared [a <- :wat::core::i64])
(:wat::core::defrecord :probe::Wire [b <- :wat::core::String])
(:wat::core::defsurface :probe::Loc :- [T] :nature :wat::core::Struct
  :features [(transport [self <- (:probe::Loc :- [T])] -> :T)])
(:wat::core::defrecord :probe::Th [x <- :wat::core::i64])
(:wat::core::extend-type :probe::Th (:probe::Loc :- [:probe::Shared]) (transport [self] (:probe::Shared :a 1)))
(:wat::core::defrecord :probe::Pr [y <- :wat::core::i64])
(:wat::core::extend-type :probe::Pr (:probe::Loc :- [:probe::Wire]) (transport [self] (:probe::Wire :b "w")))
(:wat::core::defn :probe::start-concrete [loc <- (:probe::Loc :- [:probe::Shared])] -> :probe::Shared (:probe::Loc/transport loc))
(:wat::core::defn :probe::use [] -> :probe::Shared (:probe::start-concrete (:probe::Pr :y 2)))
```

