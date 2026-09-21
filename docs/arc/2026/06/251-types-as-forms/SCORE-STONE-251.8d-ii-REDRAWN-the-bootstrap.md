# SCORE — STONE 251.8d-ii (REDRAWN): STOP

Branch: `main`. **Not the conversion.** `wat/` restored. **Not pushed.**
Drawn against `402ed7e73` (draw `6236e370f`).
Parent: `BRIEF-STONE-251.8d-ii-REDRAWN-the-bootstrap.md`.
Floor not run: startup dies before a test can. 8d-iii not started.

## What was proven on copies

Path list: `git ls-files | grep -E '^wat/.*\.wat$'` → **64**. `git ls-files 'wat/**/*.wat'` → **31** (the glob gap, again). `.wat.bad` under `wat/` → **0**. Load order in `src/load/stdlib.rs` is the same 64, nothing missing either way.

Dry-run: `/tmp/8d-ii-dry`, current `to-faithful-clojure.wat`. One file first (`wat/holon/Ngram.wat`) **0.31 s**. All 64: **1315.57 s**, rc 0, **64/64 changed**.

Applied the same list to live `wat/`: **1319.64 s**, rc 0, **0 byte mismatches** vs the dry-run. `git status` was exactly those 64 paths. No `src/`, no `tests/`, no `wat-scripts/`.

`cargo build --release` **23.24 s**, exit 0. The binary embeds the converted stdlib.

## ⛔ STOP — the converted codemod cannot run

Self-application, pass 1, on a copy of unconverted `tests/rete/probe_arc278_northstar_cold_and_windy.wat`. The tool dies in **startup**, before it reads the file. **0 bytes changed.**

```
#wat.kernel/LociDiedError.StartupError {:error #wat.type/UnknownNamedType {:message "annotation names unknown type :wat::telemetry::Journal::QueryMetricsRequest — not a declared type, not a type variable, and not a builtin" :location #wat.core/Span {:file "src/check.rs" :line 15206 :col 13 :end #wat.core/Option.None {}} :causes [] :path ":wat::telemetry::Journal::QueryMetricsRequest"}}
```

Pass 2 (the idempotence pass the brief requires) dies the same way, **different first name**:

```
#wat.kernel/LociDiedError.StartupError {:error #wat.type/UnknownNamedType {:message "annotation names unknown type :wat::query::Store::EnsureSchemaRequest — not a declared type, not a type variable, and not a builtin" :location #wat.core/Span {:file "src/check.rs" :line 15206 :col 13 :end #wat.core/Option.None {}} :causes [] :path ":wat::query::Store::EnsureSchemaRequest"}}
```

Not a flake. `validate_named_type_annotations` walks `env.iter()` and returns the **first** miss. `HashMap` order changes per process. Same class, two witnesses.

`scripts/floor.sh` was not run. It would be a second observation of this startup death, and the first name in the log would be whichever miss the map yielded. The load proof already failed.

## Attribution — 255.3, not the codemod

Written form (converted `wat/telemetry.wat`):

```
(wat.core/defrecord wat.telemetry.Journal/QueryMetricsRequest …)
(query-metrics [self :- wat.telemetry/Journal
                req  :- wat.telemetry.Journal/QueryMetricsRequest] …)
```

Same shape for `wat.query.Store/EnsureSchemaRequest`.

`canonical_identity` / `normalize_name_slot` (`ns_to_wat_path`) register that name as

`:wat::telemetry::Journal::QueryMetricsRequest`

`reconstruct_call_path` (255.3) does **not**. `Journal` is a known type, so the slash joins as a **member**:

`:wat::telemetry::Journal/QueryMetricsRequest`

`canonical_identity` leaves a string that already contains `::` unchanged, so it never folds that `/` back to `::`. `is_known_type` on the member spelling does not see the registered `::` type. The annotation wall then reports the `::` path as `UnknownNamedType`.

The function's own comment already names this and then the code does the other thing:

> *"a type name `my.Counter/Req` is not a method. Identity reconstruction stays `::` always."*

The registry join has no exception for a **type whose namespace's last segment is itself a type**. 255.4 made `/` the only member join; it did not give `Journal/QueryMetricsRequest` a way to stay a type. ⭐ **Fourteenth correction, and it is why 8d-ii still cannot load.**

`wat.core.Option/expect` is a real member (lowercase method). `wat.telemetry.Journal/QueryMetricsRequest` is a nested type. Capitalisation would split them; 255.3 forbade capitalisation. The door cannot express both.

## Recovery — proven

`git checkout -- wat/` then `cargo build --release`: **22.48 s**, exit 0. The restored binary converts the unconverted northstar copy (`RECOVERED_CONVERTS`). No stash. `wat/` is clean.

## Not done, because the tool is dead

- Floor, clippy, census, lint, nextest list, 179-file delta. A spelling flip that does not load has no test-count delta to report.
- Idempotence of converted `wat/`. The second pass never reached the files.
- 8d-iii. No tool.

**VERDICT: STOP.** The stdlib converts. It does not load. The cause is 255.3's join on a namespace that is also a type, not a missed `Keyword` arm.
