# BRIEF — STONE 255.74: a set element or map key must be data, refused by the checker

**Drawn 2026-10-01 against `main` @ `70a157245`.** **Executor: a Sonnet subagent.** A strike in `src/` (the checker and
the runtime guard), tests, and two probes. Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not push**.
Your final message is your report.

## Why

255.73's census found `wat-scripts/probes/arc-170/probe-compound-upcast.wat` reaching a Rust `unreachable!`:

```
panicked at src/value/value.rs:913:37: internal error: entered unreachable code: Value::RustOpaque is not atomizable;
is_atomizable predicate in src/check.rs should have rejected this. If you see this panic, the predicate has drifted.
```

The orchestrator isolated it without a process, in two committed probes under `wat-scripts/scratch-pad/arc-255/`:

| probe | `wat --check` | run |
|---|---|---|
| `probe-255.74-a-key-must-be-data.wat`: `(wat.type/HashSet :- [[i64 :-> i64]] :probe::inc)` | **rc 0** | rc 1, runtime `TypeMismatch` (the shallow guard catches a bare fn) |
| `probe-255.74-a-key-must-be-data-deep.wat`: a set whose element is `(Vector :- [[i64 :-> i64]])` | **rc 0** | **rc 2, `panicked at src/value/value.rs:888:40 … Value::wat__core__fn is not atomizable`** |

The design is already on disk. `impl Hash for Value` (`src/value/value.rs:883-972`) keeps an `unreachable!` arm for
every non-key variant, and its rune (`:887`) says *"`is_atomizable` (src/check.rs) statically gates every
non-atomizable variant out of all hashing contexts"*. **Measured, that is false:**

1. **The static gate is not called where values are hashed.** `is_atomizable` (`src/check.rs:1606`) has one caller,
   `to-holon`/`leaf` (`src/check.rs:3937-3970`). Building a `HashSet`, a `HashMap` key, a `#{}`/`{}` literal, `conj`,
   `assoc` never consults it.
2. **The runtime guard is shallow and a second hand list.** `value_is_hashable` (`src/runtime.rs:6771`) matches only
   the outer variant, so a `Vector`, `Tuple`, struct or record holding a resource passes and `Hash` recurses into the
   resource. Its list also differs from the `Hash` arms: it omits `wat__stream__Stream`, which `Hash` refuses (`:967`).
   The exhaustive per-variant classifier already exists: `Value::key_eligibility()` (`src/value/value.rs:1341`, gated
   against `is_atomizable` by `every_interior_mutable_variant_is_rejected_as_a_key`, `src/check.rs:~25745`).

## The work

1. **Census every hashing context first** (report it): every site where a `Value` is hashed or used as a key: `HashSet<Value>`
   and `HashMap<Value, _>` inserts and lookups, rpds `PersistentMap`/set keys, and verbs that hash internally
   (`distinct`, `frequencies`, `group-by`, `contains?` …, whatever the code has). Name each by file:line, with the type
   position the checker sees for it.
2. **The static wall.** The checker refuses a set element type or map key type that is not key-eligible, through **one
   door** (`is_atomizable`, with the record-subtype case `to-holon` already handles at `:3955-3963`), wherever the
   checker learns that type: the constructors (`HashSet`, `HashMap`, `PersistentMap`, either spelling), the `#{}`/`{}`
   literals, and the verbs from item 1 whose key type comes from an argument. The error names the offending type and the
   container, and points at the element or key. Both probes must then fail `--check` with that error. A set of
   `Capability` (a service handle) is refused the same way.
3. **The runtime guard follows the same rule, deeply, from one source.** `value_is_hashable` descends into containers,
   tuples and aggregates, and classifies each leaf from `key_eligibility()` rather than its own list (so `Stream` and any
   future variant cannot drift). It stays as the defence for values the checker cannot see (`eval-ast!`, `Value`-typed
   paths); report which paths those are. After this, each `unreachable!` arm's rune must be true; correct its prose if
   your census shows a path the checker does not cover.
4. **The probes.**
   - The two scratch probes become this stone's negative fixtures (a `.wat.bad` each, or move them, your choice: say
     which) with a driven Rust test asserting the checker's refusal and its names. They must not stay in `wat-scripts/` as
     files that fail to load.
   - A driven runtime test for the deep guard: a value the checker cannot see (built by `eval-ast!` or through a
     `wat.type/Value` path, whichever the code offers) holding a fn inside a vector, inserted into a set, must give a
     `TypeMismatch`, never a panic.
   - `probe-compound-upcast.wat`: its Set case claims an up-cast of a handle into `(HashSet :- [Capability])`, which this
     stone makes illegal. Keep its Tuple and Map claims; change its Set case to the refusal, or to an up-cast between two
     key-eligible types if the code has such a subtype pair. Then it must run with rc 0, and get the same kind of driven
     test 255.73 added (`tests/process/probe_arc170_s3b_astsplice.rs` is the shape).
5. **Tests:** refusal rows for each container in item 2 (element or key a fn, a `Capability`, a vector of fns), and
   acceptance rows proving key-eligible types still check (i64, String, keyword, a record, a vector of i64, a tuple of
   them).

## Gates

| what | how | expected |
|---|---|---|
| the probes | `./target/release/wat --check` on each | refused, naming the type |
| release floor | `scripts/floor.sh`, **one run at a time, in the foreground, nothing else running** | all passed; the count against 6236 at `9a54f673c`, plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census | pre-census on the unmodified draw; `scripts/replay/census.sh --diff` after | no rc flips except files that held a non-key set element or map key (list each) |

## Reds and STOPs

- A red caused by this stone's own change (its wall firing on a site that really sets or keys a non-key type, its own
  probes, a golden whose only change is the new error): capture it **verbatim** from `.floor/<stamp>/`, cure it, run a
  **new** floor. Never re-run unchanged code for a green. "Pre-existing" needs the prior green floor's line
  (`.floor/2026-10-01T01-32-01Z`, 6236/6236).
- **STOP-1:** the wall refuses a real corpus or stdlib site (not a probe, not a test written to be refused) that keeps a
  resource in a set or as a map key on purpose. That needs the builder's ruling (identity sets of handles are a design
  question). List every such site with file:line and STOP.
- **STOP-2:** the wall refuses a type that `key_eligibility()` classifies `Hashable` (a false positive). Quote it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you (codemods for corpus-wide `.wat` changes; these are a handful of named fixture files, edit
them directly and say so). Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number,
file:line or example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.74-a-key-must-be-data.md` beside this brief, commit it, **do not push**.
