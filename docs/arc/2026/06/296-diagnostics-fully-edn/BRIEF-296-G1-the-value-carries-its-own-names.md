# BRIEF — 296 G-1: `AggregateValue` carries its own field names

> **Scope note.** G splits in two. **G-1 (this brief) installs the carrier**: `names` on the value,
> every construction site supplying it, floor green. **G-2 deletes the 7 `format!("field-{}", i)`
> fallbacks in `edn_shim.rs`** and is a separate strike. The split is deliberate: each cascade's
> screams then have exactly one possible cause. A red during G-1 means "a site built its names
> wrong"; a red during G-2 means "a site's names are wrong *in content*". Bundled, a red is
> ambiguous between them, and the whole point of this stone is to stop being ambiguous.

Read `DESIGN-STONE-G-the-value-carries-its-own-names.md` first — it carries the *why*. This brief
carries the *where* and the *how*.

---

## THE WORK, IN ONE PARAGRAPH

`Value::Aggregate` carries positional `fields` and no names, so naming them at render time requires
a registry lookup — and the four ways that lookup can fail all collapse into one arm that answers
`{:field-0 1 :field-1 2}`. That is not a degraded rendering; it is a **lie with a plausible shape**,
and it has shipped in `str`, on `send'`'s wire, and in every failing `deftest`'s diagnostic. Add
`names: Arc<Vec<String>>` to `AggregateValue`, thread it through the three constructors, and supply
it at every construction site **from a source that is not a human's fingers**. The question then
does not get answered — it stops existing.

---

## THE SHAPE

`src/value/value.rs:976` — the struct gains one field:

```rust
pub struct AggregateValue {
    pub class: String,
    /// Field names in declaration order. **Same length as `fields`, always.**
    /// Arc 296 G: carried, never looked up — see the sibling `Value::ForeignRecord`,
    /// which self-carries its keys and has never had the `field-N` bug.
    pub names: Arc<Vec<String>>,
    pub fields: Arc<Vec<Value>>,
    pub nature: Nature,
    pub holon: HolonForm,
}
```

`src/value/value.rs:990-1003` — all three constructors take it, in the same position:

```rust
pub fn struct_(class: String, names: Arc<Vec<String>>, fields: Vec<Value>) -> Self
pub fn record(class: String, names: Arc<Vec<String>>, fields: Arc<Vec<Value>>) -> Self
pub fn holon_record(class: String, names: Arc<Vec<String>>, fields: Arc<Vec<Value>>,
                    hologram: Arc<HolonAST>) -> Self
```

`names` sits next to `class` and before `fields` at every site, so the shape reads uniformly and a
transposed argument is a type error rather than a silent swap.

---

## ⛔ WHERE THE NAMES COME FROM — the three classes, and nothing else

Every site is class A, B, or C. Classify before you edit; the class picks the line.

### Class A — a registry is already in scope → `agg.names_arc()`

The `AggregateDef` is **already resolved and in hand** at these sites. This is the largest and
easiest class, and it includes the one site all user wat code funnels through.

**`src/runtime.rs:15806-15850` — the generic aggregate constructor.** Read this one first; it is the
worked exemplar for the whole class. It already looks up `types.get(&type_key)`, already binds
`agg`, already validates arity against `agg.fields.len()`, and its HolonRecord arm already builds
`agg.field_names()`. The three `AggregateValue::…` calls at `:15834`, `:15837`, `:15843` each gain
`agg.names_arc()`. Nothing else about the function changes.

> **STOP-2 is already satisfied here.** `:15812` already returns `MalformedForm` for a class that is
> not a registered aggregate. There is no positional fallback to remove and none to add.

Also class A: `src/runtime.rs:16067`, `:16088`; `src/edn_shim.rs:2527-2528`, `:3121`, `:3187`,
`:3283`.

### Class B — rebuilding from an existing aggregate → `a.names.clone()`

The site already clones `class` off a source aggregate. Clone `names` in the same breath, from the
same value. These are the cheapest sites and the most obviously correct: the names travel with the
value they describe.

| site | the source binding |
|---|---|
| `src/runtime.rs:16473` | `agg.class.clone()` — **a struct literal, not a constructor call**; add the `names:` field |
| `src/rete/kernel.rs:625` | `a.class.clone()` |
| `src/rete/kernel.rs:3693` | `agg.class.clone()` |
| `src/rete/kernel.rs:3780` | `agg.class.clone()` |
| `src/rete/compiled_rhs.rs:222` | `c.class.clone()` |

### Class C — a hardcoded class literal, no registry → a `wat_field_names_from!` const

The site writes `"wat::core::Fault".to_string()` and a positional `vec![…]`. The names come from the
**same wat declaration** that `wat_record_from!` already reads to generate that type's registration:

```rust
::wat_source_derive::wat_field_names_from!(FAULT_FIELDS, "wat/core.wat", ":wat::core::Fault");

/// `OnceLock` so a hot error path allocates the name vector once, not per raised fault.
fn fault_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| Arc::new(FAULT_FIELDS.iter().map(|s| (*s).to_string()).collect())).clone()
}
```

then `fault_names()` at the call site. One const per (file, type); reuse it across every site that
constructs that type — `wat::core::Fault` alone has 8.

> **A literal `vec!["message".into(), …]` is not an option at any site.** It is a second place the
> names are written, free to drift, and a right-count/wrong-name literal renders **confidently and
> wrongly** — strictly worse than the `:field-N` this arc exists to annihilate, because it looks
> like an answer. The builder stopped exactly this in G's first draft.

**All 16 hardcoded classes, pre-verified against the corpus this session** — 15 have a declaration
to read; the 16th is STOP-1 below.

| class | declaration | note |
|---|---|---|
| `:wat::core::Fault` | `wat/core.wat:1873` | 8 sites — the big one |
| `:wat::core::EvalError` | `wat/core.wat:1895` | |
| `:wat::kernel::Location` | `wat/core.wat:1835` | |
| `:wat::kernel::Frame` | `wat/kernel/diagnostics.wat:25` | |
| `:wat::kernel::StopAccepted` | `wat/kernel/diagnostics.wat:57` | |
| `:wat::kernel::StopFailure` | `wat/kernel/diagnostics.wat:72` | |
| `:wat::kernel::StopFailed` | `wat/kernel/diagnostics.wat:90` | |
| `:wat::kernel::Failure` | `wat/kernel/diagnostics.wat:107` | |
| `:wat::holon::CapacityExceeded` | `wat/holon.wat:106` | |
| `:wat::holon::CoincidentExplanation` | `wat/holon.wat:120` | 6 fields — the registry corrected a hand-count here once |
| `:wat::holon::Match` | `wat/holon.wat:140` | |
| `:wat::rete::Session` | `wat/rete.wat:184` | |
| `:wat::rete::AxisViolation` | `wat/rete.wat:578` | |
| `:wat::intrinsic::Example` | `wat/doctest.wat:13` | |
| `:wat::spawn::Bound` | `wat/spawn.wat:278` | **parametric — see the note below** |
| `:wat::kernel::ThreadPeer` | **none** | **STOP-1** |

> **The parametric spelling — pass the declared keyword verbatim.** `wat/spawn.wat:278` declares
> `:wat::spawn::Bound<S,R>`, and `field_names_of` matches the type path **exactly**
> (`crates/wat-source-derive/src/lib.rs`, `if tp != want { continue }`). So the invocation is
> `wat_field_names_from!(BOUND_FIELDS, "wat/spawn.wat", ":wat::spawn::Bound<S,R>")` — the full
> declared spelling, even though the runtime `class` string at `src/runtime.rs:22091` / `:22130` is
> the bare `"wat::spawn::Bound"`. Exact-match is the *safe* direction: a wrong path fails loudly at
> compile time with the macro's own message rather than silently matching a neighbour. Leave the
> matching logic as it is.

---

## THE ROOMS — read in this order

1. **`src/value/value.rs:976-1004`** — the struct and the three constructors. The whole change starts
   here and every error downstream is caused by this edit. ~5 lines.
2. **`src/runtime.rs:15806-15850`** — the generic constructor. The class-A exemplar, and the site every
   user wat aggregate flows through. Do this second; it is the highest-value single site in the strike.
3. **`src/runtime.rs:548-650`** — six class-C sites in a tight cluster (`fault_from_runtime_error`,
   `stop_failure_value`, `fault_from_panic_payload`, …), all `Fault`/`StopFailure`/`Location`. Mint the
   consts here and reuse them; this cluster establishes the class-C pattern for everything after it.
4. **`src/rete/kernel.rs`** — 13 sites, a mix of A/B/C. The four `*.class.clone()` sites are class B.
5. **The rest, as rustc names them** — `src/edn_shim.rs` (7), `src/test_runner.rs` (3),
   `src/capability/registry.rs` (2), `src/channel/transfer.rs` (2), `src/rete/matcher.rs` (2),
   `src/freeze.rs`, `src/intrinsic/reflect.rs`, `src/rete/purity.rs` (1 each), and 11 sites under
   `tests/`.

**The compiler is the worklist, not this list.** A census by grep was wrong five separate times on
this arc — including the design's own file table, which claimed 54 sites in `value.rs` where there
are 3. Impose the change at room 1 and let `cargo build` enumerate. The list above is orientation so
you know *which class* each scream belongs to; it is not a checklist to complete.

---

## ⛔ TWO PREDICTED REDS — expected, not mysteries

Both were found before you flew. Neither is a defect you introduced.

1. **`tests/value/probe_arc234_stone1_wat_record_variant.rs:210`** pins the `Debug` rendering of an
   `AggregateValue` as a golden string. A new field changes that rendering. Update the golden to
   include `names: [...]` — the assertion's *intent* (the variant round-trips with its hologram) is
   unchanged and stays exercised.
2. **The fail-count will spike into the dozens on the first build.** That is the substrate teaching
   you the worklist (`docs/SUBSTRATE-AS-TEACHER.md`); the count is the progress meter. Watch it
   waterfall. It has run 848 → 0 on this repo before.

---

## STOP TRIGGERS — each is a rejection: ship nothing on that site, report it

- **STOP-1 — `:wat::kernel::ThreadPeer` has no wat declaration and cannot name its own fields.**
  `src/channel/transfer.rs:352` and `:359` construct it with two fields (a receiver and a sender);
  `grep -rn "ThreadPeer" wat/` returns nothing. Every other hardcoded class in the strike has a
  declaration; this one does not, so **no honest source of names exists for it today**. Do not invent
  a literal, and do not pass an empty vector. Report it and leave those two sites for the
  orchestrator — the resolution is a ruling about whether a substrate-internal test fixture
  (`make_thread_peer_pair_for_test`, explicitly *"NOT exposed to wat user code"*) gets a wat
  declaration like the other 13 did.
- **STOP-2 — a generic constructor reaches an unregistered class.** Raise. There is no positional
  fallback to reach for; that is the defect returning under a new name. (Already satisfied at
  `src/runtime.rs:15812` — if you find a *second* generic constructor without this guard, that site
  is the finding.)
- **STOP-3 — `names.len() != fields.len()` at any site.** The two are built together by construction.
  A disagreement means the site is assembling them from two different places, which is the whole
  class this stone removes. Report the site.
- **STOP-4 — a class-C type's wat declaration has a different field count than the site's `vec![]`.**
  The wat declaration is the source of truth and the site is wrong. Report it; do not "fix" it by
  padding either side.

---

## BLAST RADIUS

`src/**` and `tests/**` construction sites, plus the one golden named above. **No `.wat` file
changes** — every declaration this strike reads already exists (that was the previous stretch's
work). **No changes to `edn_shim.rs`'s `format!("field-{}", i)` fallbacks** — those are G-2, and they
must keep working until every site carries names. No new types, no signature changes beyond the three
constructors.

---

## HOW TO WORK

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — it does not suspend you,
and nothing will wake you. There is no notification coming. Run every build and test in the
**foreground** and block on it: your turn ends when the numbers are in your hands, not when the
command is launched.

- Work in `/home/watmin/work/holon/wat-rs`. Verify with `pwd` first.
- `cargo build` iteratively — it is your worklist generator. You hold the build lock alone.
- When the build is clean, weigh with `scripts/floor.sh` (it captures the whole run before you read
  it) and read the **Summary line**. Never a piped exit code — `… | tail` returns `tail`'s status.
- **On any red: do NOT re-run.** A re-run that goes green destroys the only evidence. Copy the failing
  test's entire stdout+stderr block into your report **verbatim** — never a summary, never a
  `| head` window — and name the exact assertion or match arm that fired. Then report.
- Leave the work in the tree for the orchestrator to weigh and commit.

Report: the site count per class (A/B/C) as rustc actually produced it, the floor Summary line
verbatim, every STOP you hit, and the honest deltas — anything that surprised you, especially
anywhere the design's description did not match the disk.

---

## PRIOR COMPARABLE

`0514498c` (296 step 2b) — the same shape one layer down: 13 registrations generated from wat, 126
lines of hand-written literal deleted, green in one flight. Its rider returned with a negative
control and caught a defect in the orchestrator's own brief. That is the bar.
