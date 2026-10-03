# BRIEF — STONE 255.88: the first slash is the only partition

**Drawn 2026-10-03 (queued behind 255.87).** **Executor: grok via pulsare, working solo** (it runs the floor). A strike in
the reader's `Identifier`, normalization, declaration registration, tests. Commit locally on `main` (`git add -- <paths>`,
never `-A`; `git status` clean before the floor you report); **do not push**.

## The ruling (builder, 2026-10-03)

> *"symbols may contain slashes... the first slash is the partition for namespace and name... any slash in a name is just
> a regular symbol character... `wat.core//` => `{:ns wat.core :name /}`"*

```edn
(wat.core/let
  [foo/bar 0]  ;; {:ns $bound :name foo/bar}
  foo/bar)

(wat.core/defn u/pathological/name//foo      ;; {:ns u :name pathological/name//foo}
  [] :- wat.type/nil
  (wat.core/let
    [completely/insane 42]                   ;; {:ns $bound :name completely/insane}
    (wat.kernel/println completely/insane)))

(u/pathological/name//foo) ;; :=> "42\n"
```

So: **(1)** the first `/` partitions namespace from name, and every later `/` is a name character; **(2)** a symbol in a
**binder** position is `{$bound, <whole spelling>}`, whatever slashes it holds; **(3)** a body reference resolves
locally first (hygiene, by scope set) and globally otherwise, so a slashed symbol can name a local.

## What the code does today (orchestrator, measured on the current binary)

| | result |
|---|---|
| `Identifier` split (`crates/wat-reader/src/identifier.rs:186`) | ✅ `flat.find('/')`, first slash (the field doc at `:112-118` still says "before the last `/`": stale) |
| `:wat::core::canonical-identity` | ✅ `"u/a/b"` → `":u::a/b"`; `"u/pathological/name//foo"` → `":u::pathological/name//foo"`; `"wat.core//"` → `":wat::core::/"` |
| `(wat.core/let [foo/bar 0 completely/insane 42] (wat.kernel/println completely/insane))` | ❌ rc 3: `let binder must be a bare symbol … got a keyword in binder position` (normalization rewrote the binder into a keyword before the `let` check) |
| `(wat.core/defn u/a/b [] :- wat.type/i64 7)` then `(u/a/b)` | ❌ rc: startup `UnresolvedReference` path `:u::a/b` (declared, but the call does not meet it) |
| the builder's pathological program above | ❌ `UnresolvedReference` path `:u::pathological/name//foo` |

The likely collision: the old **keyword** grammar used `/` as the type-member join (`:wat::core::Option/expect`), so a
declaration keyed `:u::a/b` may be read or re-keyed as "type `a`, member `b`" (R-a, `rekey_type_member_functions`). The
**symbol** grammar has no such reading: a type member is spelled by the dotted namespace (`wat.core.Option/expect` is
`{wat.core.Option, expect}`). Find what actually happens; the lead is not a verdict.

## The work

1. **The probe first**, committed: the builder's three examples above, in a co-located fixture, as driven tests (the
   value `42` printed; `foo/bar` bound and read; `u/a/b` callable) plus `wat.core//` read as `{wat.core, /}` and the
   hygiene case: a macro that binds `completely/insane` internally does not capture the caller's `completely/insane`.
   All red today.
2. **Binder positions are `$bound`:** every binder position (`let`, `fn`/`defn` params, `match` and destructuring
   binders, `defclause`) keeps a slashed symbol as `{$bound, <whole spelling>}`; normalization never rewrites a binder into
   a reference keyword. Find the pass that does it today and route binders around it by **position**, the way the reader
   already decides `$bound` for a slash-less symbol.
3. **One door for a declared name and its references:** a declared name is registered under exactly the identity
   `canonical_identity` gives its spelling, and a reference resolves through the same function, so `u/a/b` declared is
   `u/a/b` called. Where a keyword-grammar `Type/member` reading (R-a's member join, `rekey_type_member_functions`) treats a
   symbol's later `/` as a join, it applies to the **keyword** spelling only; say exactly where, and what the 255.86
   member-join wall now means for a symbol name containing `/`.
4. **The stale doc** at `identifier.rs:112-118` says what the code does (first slash).
5. **`flat` (measured, the builder's question):** `Identifier.flat` is derivable (`ns + "/" + name` for a reference, `name`
   for a binder, under the first-slash rule). It exists so `as_str`/`leaf`/`path` return a borrowed `&str`
   (`identifier.rs:217-245`; ~500 `.as_str()/.leaf()/.path()` call sites across `src/` and `crates/`, an upper bound over all
   receivers). **Measure, do not remove:** count `Identifier` callers of those three, and estimate the allocation cost of
   reconstructing on demand (a counter on one heavy workload). Report; removal is a later, separate decision.

6. **Owed by 255.87: the ignore ledger 19 → 18.** The doc-link judge (`tests/lint/no_new_broken_doc_link.rs:277`) is an
   `#[ignore]` test that `scripts/floor.sh` runs against the captured `cargo doc` log. The builder's ruling is one ignore
   at the end, so the judge becomes a non-test entry (a small `src/bin` or the script itself reading the log against
   `KNOWN_BROKEN_DOC_LINKS`), keeping its two-way ratchet and its planted-break proof.

## Gates

| what | how | expected |
|---|---|---|
| the probe | its driven tests | green |
| census | `scripts/replay/census.sh` pre and `--diff` after | no rc flips (list any) |
| release floor | `scripts/floor.sh`, in the foreground, nothing else running | all passed; the count against 255.87's final green floor, plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change: capture it **verbatim**, cure it, run a **new** floor. Never re-run unchanged
  code for a green.
- **STOP-1:** a corpus or stdlib name today depends on a later `/` meaning a type-member join in **symbol** spelling (it
  would change identity under the ruling). List them and STOP.
- **STOP-2:** making slashed binders `$bound` changes what an existing program binds or resolves. Quote it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. New fixtures in the target spelling (symbol heads and symbol names). Capture `rc=$?` on the
next statement. Never wait with `pgrep -f`. Never write a number, file:line or example you did not measure. If this brief
contradicts the code, the code wins: say so. Write `SCORE-STONE-255.88-the-first-slash-is-the-only-partition.md` beside
this brief, commit it, **do not push**.
