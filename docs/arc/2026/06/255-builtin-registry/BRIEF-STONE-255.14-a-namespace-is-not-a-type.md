# BRIEF — STONE 255.14: a namespace is not a type

**Drawn 2026-09-23 against `main` @ `170e39834`.** Landable floor 6013/6013, clippy 0, census
`no STOP-8`, delta **3 / RECOVERY 0**, heresy ledger **220**. Converted floor **112 → 107**.

## ⭐⭐ THE BUILDER'S RULING, 2026-09-23 — option A of four, the only 4-YES

> **A non-type parent uses the NAMESPACE join. Respell `/` → `::` in the declaration.
> The faithful form does not change.**

⭐ **THE NAMES DO NOT CHANGE. THIS IS NOT A RENAME.** `wat.spawn.process/post-spawn` is already
correct faithful Clojure — namespace `wat.spawn.process`, name `post-spawn`. **Only the rust-scheme
DECLARATION is wrong**, and it is wrong because it spells a namespace as if it were a type.

**Measured by the orchestrator — the all-`::` form already round-trips:**

```
(:wat::core::defn :my::ns::process::post-spawn …)   --check rc=0
       ↓ to-faithful-clojure
(wat.core/defn my.ns.process/post-spawn …)          --check rc=0
```

⭐ **Byte-identical to the target spelling.** Rejected: **B flatten** (0 YES — renames a name that is
already right), **C promote the parent to a type** (1 YES — invents types; `stdin-svc` as a type is a
fiction), **D registry disambiguation** (0 YES — the second-door shape this arc has killed four times).

## Why this matters

⛔ **This is the last blocker on 8d-iii.** The eighth draw proved the `/` join at a non-type parent is
**UNSPELLABLE**: `:my::ns::a/b` and `:my::ns::c::d` **both** convert to `<ns>/<leaf>`, so two distinct
keyword names share one faithful image. ⭐ **Every remaining `UnresolvedReference` in the converted
floor is this one defect** — 7 spawn constructors and one deliberate bogus control.

## The work — measured, two parts

**PART 1 — 7 hand-written declarations** (`wat/spawn.wat`, verified by the orchestrator):

```
:105 thread/init      :110 thread/post-spawn   :116 thread/runner-count
:130 process/post-spawn  :133 process/env  :141 process/max-message-bytes  :149 process/runner-count
```

**PART 2 — ONE mint template** (`wat/service.wat:2021-2023`):

```wat
method-name  (:wat::keyword::from-string
               (:wat::string::interpolate "{b}/{op-str}" :b fqdn-base :op-str op-str))
```

⭐ **That single interpolation mints the whole `<svc>/<op>` family** — 12 measured
(`hologram-svc/{get,put,start,stop}`, `lru-svc/{get,grant,put,start,stop}`,
`std{in,out,err}-svc/start`) ⚠ **and an unmeasured ~140 runtime-minted members the eighth draw
reported but did not verify.**
⭐ **The same macro already mints `<fqdn>::Handle` with `::`** — this unifies it, it does not
introduce a new convention.

**Blast radius, measured:** spawn **62 occurrences / 28 files** · svc **24 / 6 files** ·
**34 files total.**
⛔ **`.wat` migrations are a SELF-HOSTED CODEMOD, never hand-edits or sed** (`wat/fix.wat`,
`wat-scripts/fixes/*.wat`). **Record it.**

## ⭐ 255.4's lint does NOT conflict — verified, not assumed

`tests/lint/one_member_join.rs` says *"a member join is `/`, always"* and refuses colon-joined type
members. ⭐ **Its discriminator is CASE-based:**

```rust
tc.is_ascii_uppercase() && (mc.is_ascii_lowercase() || mc == '_')
```

`process`, `thread`, `stdin-svc` are **lowercase**, so the wall does not fire. ⭐⭐ **The lint's own
heuristic already encodes the namespace-vs-type distinction this ruling makes.**
⚠ **Confirm by running it, not by re-reading this paragraph.**

## ⚠ WHAT IS **NOT** ESTABLISHED

1. ⛔ **Whether the ~140 runtime-minted members all come from THAT template.** The eighth draw
   reported the number and said it narrowed *by argument, not measurement*. ⭐ **Measure it.**
2. ⛔ **Whether the 416 `Enum.Variant/field` accessors are in this class at all.** Unexamined.
   **If they are, say so and STOP — that is a separate ruling.**
3. ⛔ **Whether changing the mint collides with the macro's OTHER minted names**
   (`<fqdn>::Handle`, `<Surface>::<op>/Request`, `::Record`). ⭐ **This is the real risk in part 2.**

## The gate

- ⭐ **The 8 frozen names in `UNSPELLABLE_IN_THE_FAITHFUL_SURFACE` (`src/freeze/env.rs`) SHRINK** —
  the 7 spawn names leave. ⛔ **Re-freeze in the same commit; the gate's own doc says it is retired or
  re-aimed when 8d-iii lands, NEVER loosened.** ⚠ Its non-vacuity floor `slash_joined >= 16` will move
  — **say what it becomes and why it still discriminates.**
- ⭐ **Converted floor 107 → ?** with **FIXED/NEW against the eighth draw's 107**. ⚠ Use `clean.log`,
  strip the whole `FAIL […] (n/m)` prefix. ⛔ **DO NOT PROMISE a subtraction.**
- **Non-vacuity, one test:** the respelled name resolves in **both** spellings, same identity ·
  ⛔ **a genuinely unknown member is STILL refused** · ⛔ **`ProcessLaunch/bogus-field` (a type
  parent) is UNTOUCHED and still resolves** — ⭐ **this stone must not move legitimate `Type/member`
  joins.**
- `scripts/floor.sh` green; clippy 0 (⚠ not cached); census `no STOP-8`; `delta.sh` baseline **3**,
  ⛔ **RECOVERY non-zero is a STOP**; ledger 4/4 (⚠ it may not move — **say so if it does not**).
- ⭐ **This stone LANDS its `.wat` changes** (it is a corpus respelling, not a dialect flip).
  ⛔ **It does NOT convert the stdlib and does NOT start 8d-iii.**

## ⛔ Operational traps

- ⛔⛔ **NEVER `git add -A`, NEVER invoke cargo, while a codemod is writing `.wat`.**
- ⛔ **Strip ANSI (`sed 's/\x1b\[[0-9;]*m//g'`) on ANY live tool output you match against** — a probe
  matching `^\s+PASS` on live `nextest` output **cannot return PASS** and cost a stone its address.
  ⭐ **Prove any probe can say BOTH words, and carry a control in every run.**
- ⛔ `git ls-files 'wat/**/*.wat'` → 31 of 64. Use `git ls-files | grep -E '^wat/.*\.wat$'`.
- ⛔ The stdlib is `include_str!`'d — rebuild between steps and say you did.
- ⚠ `cargo clippy` can report rc 0 **from cache in 0.07 s**.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** ⚠ One bookkeeping exception: `harvest_wrap_split`
  (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`). **Cite and report; never a pass, never
  re-run to clear. Name it either way.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Twenty stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Twenty-six corrections across
  twenty-four stones**, five of them wrong addresses. ⭐⭐ **Every "measured" line here was produced
  by the orchestrator this session and is quoted with its site. §"what is not established" is the
  honest boundary.** Assume a twenty-seventh.
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

The 416 variant accessors (**a separate ruling if they are in this class**) · shape **E** —
⛔ **125 sites, 78 in `check.rs`, untouched across eight draws; the terminal cut needs A+E at zero** ·
`match_qq_head`'s ledger blind spot · variant tags in declarations (**ruled, unbuilt**) ·
`wat.type`'s roster (**undecided**) · `is_quasiquote_form` · `Ngram` · 8d-iii.
