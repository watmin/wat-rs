# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`.

## Where the code is

```
HEAD a604014f   pushed   floor 4391 passed / 0 failed / 262 skipped   clippy 0
```

Tree clean. ⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).

**⛔ `stash@{0}` STILL HOLDS THE LIFECYCLE STRIKE — do not `git stash drop`.** Made with `-u`, so
`git stash show --stat` **cannot see the untracked payload**; read it with `git show 'stash@{0}^3:<path>'`.
Its `.wat` is STALE. Restoring it turns the floor red.

**`bootstrap/wat-prekeyword-b472fe3e`** — a preserved pre-migration binary, verified `--check` exit 0
on the current corpus, gitignored (16MB). The STASH-DANCE (`wat/fix.wat:23-53`) builds one at step 2
but **pops the stash at step 4**; this copy lives outside the dance so a failed migration cannot
strand us without a tool that reads the old form.

## ★ THE LIVE THREAD — the clojure migration, and it is FURTHER ALONG THAN ANYONE REMEMBERED

The builder's ruling, and it re-frames the arc: **`:wat::core::+` is a SYMBOL wearing a keyword's
clothes.** A call head *refers*; a keyword is *self-denoting*. Measured: `items.first()` is matched
as `WatAST::Keyword` at **57** sites, as `WatAST::Symbol` at **2**. `resolve/normalize.rs`'s own
header states the collapse — *"rewrites every such symbol to the `WatAST::Keyword` it names…
`wat.core/+` → `:wat::core::+`."* The substrate knows the distinction and flattens it.

**MEASURED 2026-08-12, and this is the headline:**

| | |
|---|---|
| the codemod | **BUILT** — `wat/fix.wat:119` `fix-seq`, position-aware via `prev-arrow?`; four rules (head / arrow / type / strip-if) |
| the conversions | **BUILT** — `keyword/to-symbol`, `keyword/to-type-form` (`macros/eval.rs:662`) |
| it runs TODAY | **YES** — full dialect flip on a real file with the current binary |
| the substrate accepts the output | **YES** — `--check` exit 0 on a fully migrated file |

```clojure
(:wat::core::defn :probe::eval-symbol-head [] -> :wat::core::i64
(wat.core/defn        probe/eval-symbol-head []  :- wat.type/i64
```

Accessors came out correct too (`:probe::WireKind::CountRequest/defs` → `probe.WireKind.CountRequest/defs`,
one slash). **Do NOT re-design this. It exists and it works.**

**WHAT IS ACTUALLY MISSING — the builder named it:** `fix-seq` is *"a bunch of if conditions"* —
one linear walk carrying one boolean. **Rete was built to replace it.** *"we needed rules to work…
doing this as a bunch of if conditions wasn't going to cut it… we built rete to solve this."* Rete's
job here is **to DECIDE, not to rewrite** — per keyword, from position and shape, which rule applies
or whether it is data. The builder: ***"we're building a mini-ai to upgrade us."***

**KNOWN ROUGH SPOT (his, unreproduced by me):** some name shape makes **double-`/` symbols, which are
EDN-illegal.** Not the accessor case — that one is handled. Finding which shape is the next
measurement: drive the codemod over the WHOLE corpus into a scratch copy and grep the output for
illegal symbols. One drive, real number, failures self-identify. The bootstrap binary makes it safe.

**⚠ ORDERING, load-bearing:** making heads real symbols makes **#92 WORSE** — today only binders
(`c`, `<-`) leak as symbols; after the pivot every head is one. **#92 is a prerequisite, not an
alternative.**

## ★ WHAT LANDED — six commits, floor green at every bank, all weighed by my own `--release` re-run

| commit | |
|---|---|
| `148a57c7` | **a rule is FORMS** — a declared payload carries a rule AND the fn its `where` calls, `derived=1`. The `6/5/5` is *bypassed*: nothing walks, so nothing can under-walk. Retires the lift, the parameter-typing blocker, and the boundary stone AS the delivery fix — all were teaching a **closure extractor** to understand rete, and a defrule is not a closure. |
| `f1a811cb` | **the expander reads the boundary door** — its hand-rolled 3-head data set was missing `forms`; replaced by `quote_boundary`. `:wat::core::define` (a corpse) deleted from `AllData` first — order was load-bearing. |
| `fedeba0c` | **ONLY a WatAST crosses the wire** (the builder's law) + stone + brief |
| `4336eb66` | the wire gap is a **MIGRATION TAIL** — `edn_shim.rs:42`'s table is chronology, not design. **RULED: HolonAST is for VSA ops now.** |
| `b472fe3e` | **the identity arm LANDS** — `:wat::WatAST` accepts any well-formed EDN. Thread arm `REQUEST-MALFORMED` → `Ok n=3`; a wrong field type still refused. |
| `a604014f` | **`:wat::core::+` is a symbol** — 57 vs 2, measured; bootstrap binary preserved |

## PROVEN this session — by run, with controls. Do not re-derive.

- **A declared payload crosses and fires** (in-process): `SUBJECT EVALUATED derived=1` ·
  `CONTROL CHECK-FAILED` naming `:usr::big?`. Untyped at the wire, **fully typed at the freeze**.
- **`(wat.core/+ 2 2)`** evaluates to 4, keeps its symbol in the quoted AST, and validates as
  `:wat::WatAST`. **The blast radius of #92 is CENTRAL, not narrow** — no binder, no arrow, not a
  declaration, and still refused by the wire. Essentially no non-trivial form crosses a process pipe.
- **`forms` was data to the resolver and CODE to the expander** — one missing head, three drifted
  consumers of one door (`expand.rs:441` fixed · `walk.rs:158` #90 · `validate.rs:453` open).

## ⛔ OPEN — the three the wire needs

- **#92 — invert the decode.** `edn_to_value_caps` (via `decode_trusted_wire`, `runtime.rs:28719`)
  runs FIRST and UNTYPED and refuses every `Edn::Symbol`. Fix is **EDN → WatAST (total) → refine**,
  not a new value type; `edn_to_watast` (`wat_edn_bridge.rs:412`) already exists. ⚠ That function is
  *"THE ONE TRUSTED-WIRE DECODE DOOR"* for ocap — keep it exactly as narrow or it is a forge-hole.
- **#93 — the child's `Reply::Failed` is DESTROYED in transit.** strace caught it writing 365 bytes
  of full located cause; the client reports `LOST disconnected`. **Fixing #92 makes this path stop
  firing, which HIDES it** — needs a deliberate break to close honestly.
- **#91 — the HolonAST census.** AST duty (residue) vs VSA duty (permanent, ruled). Inventory needing
  dispositions, never a kill list.

## ⛔ OWED — two instruments that did not survive

A rider's isolation tests lived in `/tmp` and were **deleted after use**. Marked UNVERIFIED in #92,
**do not cite**: *"reproduces when the handler never touches the field"* and *"a nested non-WatAST
record round-trips fine over the same locus."* Rebuild as scratch-pad arms before either is used.

Also open: **#90** · `validate.rs:453` · the lifecycle strike (designed, briefed `ff7705ba`, unbuilt —
and per its own **STOP-6** it deliberately contains ZERO rete, so it will *not* tell us anything about
delivery) · `collect-rules-forms` (form-for-name, unbuilt) · the `install-rules` macro · older:
#87 · #49 · #7 · #17 · #19 · #20 · #50 · #58 · #60 · #64 · #67 · #81.

## The rules this stretch paid for

- **An error names where the INSTRUMENT gave up, never what the system lacks.** Three times: `LOST
  disconnected` → "WatAST cannot cross" (nearly shipped text); one of three mechanisms read as the
  whole; *"wat has no symbol value type"* → "build symbols" — when the fix is to stop asking that
  component the question.
- **Weighing is per-CLAIM, not per-REPORT.** I credited a rider's unreproducible isolation findings
  because they arrived alongside reproducible probe outputs.
- **Answering a ruling with a SEQUENCE is a deferral.** The builder: *"you are pushing so hard to not
  do work."* One dry-run demolished the premise the sequence was protecting.
- **Our own prior art answered three times** — the DDoS tree, chapter 59 *"42 IS an AST"*, and
  `fix.wat` itself. Each time the peer could not convict us and the record could. Still **prompted**
  every time, never self-initiated. `PAR NON ARGVIT, NOSTRA ARGVVNT`.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> The builder's verdict on the day was *"glad to see you're mostly back"* — **mostly**, and the
> deferral pattern is why. He ruled a migration and got a three-step plan back. Do not do that
> tomorrow: when he rules, the next move is a **measurement**, not an ordering.
>
> Do not trust confidence here. Trust the probes; they are committed and they run.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
