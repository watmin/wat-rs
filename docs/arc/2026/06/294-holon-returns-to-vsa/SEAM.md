# SEAM — the ONE live breadcrumb. As of 2026-08-20 (the angle-bracket crusade). Replaced in place.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `0599f6750`.** Run **`git log --oneline 0599f6750..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 4855/4855, 0 FAIL, 19 skipped, ~70s   (own invocation, scripts/floor.sh)
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba, ~390 commits back.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …`. Read exit codes directly, never through a pipe.
⚠ **`mcp__wat__eval` runs `~/.cargo/bin/wat`, NOT `target/release/wat`.** `cargo install --path .
--bin wat --force` after any substrate change or the MCP is a time machine.

## ★★ THE WORK: ARC 109 — ANNIHILATE ANGLE BRACKETS

> Builder: *"all parametrics must be expressed as constructor who receives a vec-of-types for its
> instance… no inference — no ambiguity — you say what it is. **the verbosity is our shield** — we are
> optimizing this for LLMs not humans… same reason why everything is fqdn all the time."*
> And: *"they are gone, completely… they die this day."*

```
(:wat::core::Vector [:wat::core::i64])        TYPE after <- / ->  ·  EMPTY INSTANCE elsewhere
(:wat::core::Vector [:wat::core::i64] 1 2 2)  INSTANCE with values
```

Near-term keeps the rust-ish `:wat::core::` head. The `wat.type/` flip is LATER and separate — that is
the **second** hard problem (illegal keywords), explicitly not this one.

### Where it stands

```
①   bracket ACCEPTED, all six ctors, checker + runtime   ✅ f454c465 · df90b990
②-i  renderer brackets + COLON head mode                 ✅ 0422b67ff   (closed 300's blocker note)
②-ii the codemod, written and PROVEN on /tmp             ✅ 0599f6750   wat-scripts/fixes/parametrics-take-a-type-vector.wat
⛔ NEXT — the Tuple arm. ②-iii IS BLOCKED ON IT.
②-iii apply to wat/ ALONE (~470 sites), floor, commit
②-iv  tests/ + wat-scripts/ (~2,070)
②-v   the 692 .rs string literals — separate strike
③    angle form ILLEGAL + vec MANDATORY — the checker WRITES each fix as a `remedy`
④    Fn types are brackets (141 sites)  ·  ⑤ type-name casing (keyword→Keyword, ~572)
```

## ⛔⛔ THE NEXT STRIKE — the Tuple arm, and the builder RULED its shape

`TypeExpr::Tuple`'s arm in `type_expr_to_clojure_form` (`src/edn_shim.rs`) is **mode-blind and
unbracketed**. Measured:

```
:wat::core::nil            → (wat.type/Tuple)                          wrong spelling
Result<nil,String>         → (:wat::core::Result [(wat.type/Tuple) …])  MIXED in ONE form
:(i64,i64,String)          → (wat.type/Tuple :i64 :i64 :String)         wrong spelling AND flat args
```

**Builder's ruling, 2026-08-20 — this specifies the fix:**
> *"nil is rust's unit… but **`nil != ()` in wat. nil is not an empty list**. `(wat.type/Tuple)` is
> illegal, it'd be `(wat.type/Tuple [])` to be an empty tuple."*

So: **(a)** stop canonicalizing — `types.rs:4728` collapses `:wat::core::nil` → `Tuple(vec![])`, which
is why the renderer cannot say `nil`. A `canonicalize: bool` already exists (`types.rs:4625`); the verb
calls `parse_type_expr`, which hardcodes the canonicalizing path. **(b)** bracket the Tuple arm and
honour the mode, so an empty tuple is `(:wat::core::Tuple [])` — head always takes a bracket, even empty.

Verified distinct at the surface: `-> :()` with a `nil` body is a type error.

⚠ The codemod SKIPS these rather than corrupting them (a rendered-output guard refusing any
replacement containing `wat.type/`): **30 standalone tuples · 66 nested-`nil`**. And dropping
`fix.wat`'s post-arrow rule is what saved **1,031** bare `-> :wat::core::nil` returns.

## THE CODEMOD — proven, and the property that matters

`printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/parametrics-take-a-type-vector.wat`

Verified by own hand on `/tmp` copies of `wat/sqlite.wat` + `wat/fix.wat`:
**`<-` 173/173 · `->` 139/139 · `<` 6/6 · `>` 2/2 · `<=` 1/1 · `>=` 1/1 — nothing moved.**
Nesting nests · primed `HashMap'<K,V>` survives · idempotent.

⚠ **9,912 arrow/operator sites must never move.** The discriminator is `wat/fix.wat`'s
`type-shaped-keyword?` — *requires a MATCHING close*. Reuse it; never write another.
⚠ **`ast->source`, NEVER `write-forms`.** The latter re-spells every `::`-keyword to EDN-dotted form.
This cost real time; see the memory entry.

## ALSO LANDED TODAY (not 109)

- **io home #12 CLOSED** — `src/intrinsic/io/{mod,reader,writer,fs}.rs`, 29 verbs, literal dispatch 0.
- **The Persistent family is under the type checker** — 13 schemes registered (`9c82f157`). It was
  entirely blanket-accepted; a declared `V` was INERT before this.
- **#110's INVENTORY**: 520 verbs probed, **487 checked / 32 not**. ★ And the blanket is the
  **RESERVED PREFIX**, not a missing scheme — `(:wat::core::totally-bogus 1 2)` check-PASSES at any
  arity while `:user::` is rejected. `--check` certifies calls to functions that do not exist.
- **②a v2 ring 1** (`46058fe51`) — 16 rete memory annotations, each copied from a caller. Ring 2 next;
  `network`/`facts` stay bare because nothing upstream decides them.

## THE STILL-OPEN

- **②a ring 2** — `append-token`'s `beta-mem` is decided three ways over by ring-1 callers.
- **`bindings`' V is UNRESOLVED.** Not `Value` (rete compares with `<`/`>`, keys on them, conjes them).
- **255 #110 · 285 Map half · 296 OPEN · 295 not started.**
- Unmeasured, and it sizes 255.1b-iv: **does removing the blanket turn the corpus red, and by how much?**
- Dead reference: `holon_type_ast_to_wat_type_form` (`runtime.rs:14462`, `check.rs:3538`) exists nowhere.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **AN ANSWER LABELLED "SETTLED" DISABLES THE ONE CHECK THAT WOULD CATCH YOU.** I handed a rider
> `bindings <- [String Value]` as "builder-ruled, settled". It was wrong, it went to 24 sites, and the
> label is why nobody questioned it. Cite the derivation, never the verdict.
> `[[feedback_the_authority_you_cite_decides_who_can_catch_you]]`
>
> ⚠ **SAME FILE, SAME RIDER, TWICE: 2620 FAILURES vs GREEN FIRST TRY.** The only variable was the
> AUTHORITY. Prose the compiler cannot check → wrong and unfalsifiable. A caller's declared type →
> the compiler validates every step. Prefer sources that can fail loudly.
>
> ⚠ **A PRINTER IS A FUNCTION, NOT A WINDOW.** I read `write-forms` output as the value and sent a
> rider a wrong correction. **It refused and measured, and it was right.** A rider that had complied
> would have corrupted the renderer to match my broken probe — and shipped GREEN, because the wrong
> spelling parses.
>
> ⚠ **NEVER `git add -A` WHILE A RIDER IS IN THE FIELD.**
>
> `NON BIS IN IDEM FLVMEN.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `NISI FRANGAS, NIHIL PROBAS.`
