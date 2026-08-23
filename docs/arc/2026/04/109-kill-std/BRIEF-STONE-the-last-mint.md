# BRIEF — the last mint, and then the wall

Two classes of angle-name minting survive. Close them, put the wall up at all three doors, and `<K,V>`
is unexpressible everywhere in wat — not written, not minted, not rendered.

Read `DESIGN-STONE-the-last-mint.md` first. The tree is CLEAN and the floor is green at 4903/4903.
Copy the report shape of `SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

## STEP 1 — class 1: the client-fn DECLARATION name (~9000 mints/floor)

`wat/service.wat:1996`:

```clojure
method-name (:wat::core::keyword/from-string
              (:wat::core::string::interpolate "{b}/{op-str}{p}" :b fqdn-base :op-str op-str :p fqdn-tp))
```

`{p}` is `fqdn-tp`, the `<K,V>` suffix built at `service.wat:303` from `fqdn-tp-syms`. The code's own
comment says what this is: *"the client fn's SIGNATURE … the **DECLARATION** carries the service's own
binders."* That is **position 1**, the declaration binder — γ-i's, working since the campaign opened.

★ **The precedent is in the same file and one stone old.** `proto-tp` was exactly this shape and was
killed by emitting the bare name with `:- [syms]` as siblings in the emitted `defn`. `fqdn-tp` is its
twin; read that change (commit `c6c614fe2`, `wat/service.wat`) and mirror it.

`fqdn-tp` should die with its last consumer, as `proto-tp` did. `fqdn-parametric?` and any
mono-vs-parametric `if` that only existed because two spellings did go unconditional — the macro always
emits `:- [syms]`, empty or not.

⚠ A stdlib `.wat` edit is INVISIBLE until you rebuild.

## STEP 2 — class 2: two fixtures feeding `keyword-node` angle strings (~8 mints)

```
tests/resolve/probe_arc251_keyword_to_type_form.wat
tests/resolve/probe_arc251_type_namespace_fix.wat
```

Both deliberately pass `":wat::core::Vector<wat::core::i64>"`-style strings to `keyword-node` to
exercise `keyword/to-type-form`. With the wall up, `keyword-node` refuses that input.

⛔ **Do NOT retire `keyword/to-type-form` or `to-type-form-colon`.** `wat/service.wat:434` calls the
colon variant, and its comment records that it accepts EITHER `:S<K,V>` **or** `(S :- [K V])` — it is a
transition shim. Its angle half dies with the purge, once a green floor proves it unreachable. Not here.

For each fixture, choose and SAY WHICH: it becomes a `.wat.bad` negative control proving the refusal, or
it moves to the form spelling and keeps testing the half that survives. Their owning `.rs` assertions
move with them — **assert the MECHANISM, not the whole diagnostic**.

## STEP 3 — the wall, at all THREE doors

```bash
git apply docs/arc/2026/04/109-kill-std/STONE-the-last-mint.wall.patch
```

That patch walls `keyword/from-string` (`src/runtime.rs`) and `keyword-node` (`src/edn_shim.rs`) with
`angle_type_head_in_name` — the lexer's own predicate — sharing one message,
`angle_minted_name_reason`. It applied cleanly as of `c6c614fe2`.

**Then wall the third door yourself: `symbol-node` in `src/edn_shim.rs`.** An earlier rider found it
genuinely unwalled; it is harmless today only because the checker's surface arm keys on `Keyword`
rather than `Symbol`. Same predicate, same message. An unwalled door is an unwalled door.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★★ | `(keyword/from-string "my::Map<K,V>")` | ⛔ refused, message names `:-` |
| 2★★★ | `(keyword-node ":Vec<T>")` | ⛔ refused |
| 3★★★ | `(symbol-node "Vec<T>")` | ⛔ refused — the third door |
| 4★★★ | `(keyword/from-string "wat::core::i64::<")` | ✅ minted — the OPERATOR survives |
| 5★★★ | `(keyword/from-string "foo/bar")` · `"wat::kernel::Peer'"` | ✅ minted |
| 6★★ | a parametric `defservice` round-trips | lru-svc / hologram-svc |
| 7★★ | the imposed census | flip the wall to log-and-continue, floor, expect **0** |

**Rows 4 and 5 decide it.** Rows 1–3 go green for a wall that refuses every minted name — which takes
the whole stdlib with it. Only operator and ordinary names still minting proves the predicate matched a
type-head and nothing else.

Row 7 is the census and it must be run the way the DESIGN's was: flip the wall's `return Err` to an
append-to-file, run the full floor, read the file, then restore the refusal. Do not grep for it.

## STOP triggers

- **STOP-1 — a minting site cannot emit the bare name + binder.** Report the site and what needs the
  suffix. `proto-tp`'s twin should not hit this, and if it does that is the finding.
- **STOP-2 — row 7 finds a class the DESIGN's census did not.** Report every distinct name. That is the
  finding, not an obstacle.
- **STOP-3 — an operator or ordinary name stops minting** (rows 4-5). The predicate is wrong; report the
  exact string. Do NOT widen it to get past this — it is the lexer's own predicate, so a disagreement
  between them is a real finding.
- **STOP-4 — walling `symbol-node` breaks something.** It has no known consumer for angle names; if it
  does, report what.

## Boundaries

- `wat/service.wat`, the two `tests/resolve/` fixtures and their `.rs`, the wall patch, and
  `symbol-node`.
- **Do NOT retire `keyword/to-type-form` / `to-type-form-colon`.** Transition shims with a live caller.
- **Do NOT delete the angle PARSERS** (`split_type_params`, `canonical_callable_name`, `check.rs`'s
  explicit-suffix arm). The purge is the NEXT stone and needs this one green first.
- **Do NOT touch `keyword/from-string`'s NAME.** Its own NOTE, decided with verb-equals-type.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- Goldens: **KEEP PINNING THE SPAN** and recapture; verify each is the same call site, only moved.
- The orchestrator runs the full floor and clippy centrally. Use `./target/release/wat --check <file>`
  and scoped `cargo nextest run --release -E '...'`.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.
`cargo wat` uses the STALE installed binary; always `./target/release/wat`.

## Your report

Rows 1-5 verbatim in ONE run, refusals and survivals together — that pairing is the whole proof. Then
row 7's census output. What each class-2 fixture became and why. Any STOP that fired, with the arm
captured verbatim BEFORE you diagnosed it. What surprised you.
