# BRIEF — the dormant minter in `wat/core.wat`

`<K,V>` is unexpressible in every channel we have closed. **One minter survives**, dormant only because
the corpus happens not to use the feature combination that reaches it. The probe that reaches it is
below, already built and measured.

Read `NOTE-a-runtime-census-cannot-see-a-dormant-minter.md` first. The tree is CLEAN, floor green at
4924/4924.

## The survivor

`wat/core.wat:736`, in `kwargs-defn`'s companion-name machinery:

```clojure
binder-tp  (:wat::core::if has-binder
             (:wat::core::string::concat "<"
               (:wat::core::string::concat (:wat::core::string::join "," …binder names…) ">")))
```

It feeds `keyword/from-string` at `core.wat:835` (`{b}::Kwargs{p}`) and `:949` (`:{b}$impl{p}`) —
**both walled doors.**

## The probe — built and measured, three ways

```clojure
;; A — kwargs, NO binder                      → 3   ✅
(:wat::core::defn :u::hold [seed <- :wat::core::i64 & [times <- :wat::core::i64]] -> :wat::core::i64 times)

;; B — binder, NO kwargs                      → 5   ✅
(:wat::core::defn :u::hold :- [T] [seed <- :T] -> :T seed)

;; C — BOTH  → ⛔ "macro :wat::core::defn — program body eval failed", minting `u::hold::Kwargs<T>`
(:wat::core::defn :u::hold :- [T]
  [seed <- :T
   & [times <- :wat::core::i64]]
  -> :wat::core::i64
  times)
```

**Each feature works alone. Only the combination fails.** A and B are the controls that make C mean
something — ship all three.

## The fix has a worked precedent, twice

`proto-tp` (`wat/service.wat`) and `fqdn-tp` (same file) were exactly this shape and both died the same
way: **the companion name becomes BARE, and the type params ride as a `:- [syms]` binder on the emitted
form** instead of being concatenated into the name. Read commits `c6c614fe2` and `0811c3009` for
`wat/service.wat` and mirror them.

⚠ `core.wat:798` claims *"every parametric `defservice`'s auto start/resume is exactly this."* If true,
this path is reachable today and the corpus is merely lucky. **Determine whether that claim still holds
after the mint stones reshaped `defservice`** — and say which, because a stale claim in a comment is
what sent the last three riders hunting.

⚠ A stdlib `.wat` edit is INVISIBLE until you rebuild.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★★ | probe C (binder + kwargs) | compiles and returns 3 |
| 2★★★ | controls A and B | still 3 and 5 — the fix must not break either half |
| 3★★ | a parametric `defservice` round-trips | lru-svc / hologram-svc |
| 4★★ | no angle name minted anywhere | flip the wall to log-and-continue, floor, read, restore |
| 5★ | `core.wat:798`'s claim | verified or corrected, and said which |

**Row 2 decides it.** Row 1 goes green for a fix that disables kwargs, or binders, or the companion
machinery entirely. Only both halves still working alone proves you fixed the join.

Row 4 must be taken the way the mint stones took theirs — flip the wall's `return Err` to an
append-to-file, run the FULL floor, read the file, restore the refusal. Do not grep.

## STOP triggers

- **STOP-1 — the companion name genuinely needs the params in its TEXT.** Report what consumes it;
  `proto-tp` and `fqdn-tp` both turned out not to, but do not assume it repeats.
- **STOP-2 — probe C reveals a second failure behind the first.** The wall fires early; something after
  it may also be wrong. Report the next arm verbatim.

## Boundaries

- `wat/core.wat`, and a co-located probe + controls under `tests/`.
- **You may not spawn sub-agents.**
- Do NOT commit, push, stash or amend. Keep the index EMPTY.
- Do NOT touch `.rs` comments — a concurrent rider owns those.
- The orchestrator runs the full floor and clippy centrally.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY. `cargo wat` is STALE; always `./target/release/wat`.

## Your report

Rows 1 and 2 verbatim together — C working AND A/B still working. Row 4's census output. What you found
about `core.wat:798`'s claim. Any STOP with its arm captured verbatim first. What surprised you.
