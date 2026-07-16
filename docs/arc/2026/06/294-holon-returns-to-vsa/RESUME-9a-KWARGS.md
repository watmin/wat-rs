# RESUME — arc 294 item 9a: kwargs-everywhere. **CLOSED 2026-07-15. Floor = 1.**

> ⛔ Compaction erased the working memory that produced this. Run `recolligere` FIRST (grimoire + 4 primers from the
> SIGNED MCP, never disk). Ground every line below against the disk before acting. **Freshness probe: this was written
> against HEAD `6d6bc685` — if the live HEAD differs, trust the log and the code over every claim here.**

## Where it stands

**Item 9a is DONE.** `4146 run / 4145 passed / 1 failed / 328 skipped` — the one failure is
`no_inlined_wat_in_tests` at **351 files**, the ONE allowed failure, at **exactly its pre-flip count**. No timeouts.
Branch `arc-170-gap-j-v5-deadlock-state`. Marked ✅ in `CLOSE-SEQUENCE-293-294.md`.

The flip (bare aggregate name = **kwargs macro**; positional demoted to the type-name **PRIME `:ns::T'`**, which is
**generated-code-only, NEVER user-facing**) is landed and the whole corpus is migrated: `tests/`, `wat/`, `wat-scripts/`.

**NEXT: → 278 T1b** (`TelemetryService'` sink). (C) unblocked it — the spliced stdlib (`telemetry'`/`Journal`) loads again.

## The FORM is settled — do NOT reopen

- **kwargs everywhere a human writes**; the prime `:T'` is for GENERATED code only (macro output, Rust codegen).
- **What decides the form is the SITE, not the type.** A hand-written construction of a *generated* type still takes
  kwargs (spreading the prime into source is the "do not educate bad forms" failure). Rust applying a ctor positionally
  via `apply_function` takes the prime (`runtime.rs:1145` — *"THE ONE ctor source, now at the prime"*).
- `matches?` / rete `:when` patterns are DATA; `:then` RHS is kwargs. Full-Lisp: a macro gets its args RAW.
- **Registration is SEQUENTIAL during expansion** — a `do`/`let` body's children see earlier siblings' `defmacro`s
  (`fadb03df`). This was the defservice/deftest cluster's root; do not "simplify" it back to a special case.
- `:durable`/`:ephemeral` take a FIELD VECTOR — walled at `wat/service.wat` (`00bc5fd3`).

## The method that actually worked (use it)

- **Run ONE failing test with `--no-capture` and READ its rich error.** Never grep the floor and speculate. wat's
  errors name the exact path/span. Grep only to COUNT.
- **Weigh every kill by your OWN re-run + a name-level floor diff** against a pristine baseline. Counts hide swaps.
  Extract names by stripping ANSI **and** the `( n/total)` counter (it can carry a **leading space**) before `comm` —
  otherwise you diff timings and manufacture phantom regressions.
- **Mechanical sweeps: batch every edit, verify ONCE, hand back** (builder's call; 24 flips → one run → one residual).
- **Ground each construction against ITS OWN declaration — never a type-name-keyed map.** The same-name hazard is
  REAL: `:fix::Node` is declared in two files with different fields; `:my::Cfg` likewise (t6/t7). That is what the
  original global codemod got wrong. Do not re-run it globally.

## Banked, deliberately — both RAISE the floor before lowering it; run each from a green floor

- **`ast-kind` must return a wat ENUM, not a Rust `String`** — `docs/arc/2026/04/109-kill-std/NOTE-ast-kind-returns-a-wat-enum.md`.
  ~46 consumers scream as located type errors → a sonnet census. (`8b343d56`)
- **The `HolonAST → Hologram` face-fix** — `lookup-define`/`signature-of-defn`/`body-of` still render through the OLD
  `watast_to_holon` path (`wat_edn_bridge.rs:22` calls it exactly that). The data is ALREADY WatAST; the conversion is a
  pure face-transform (~10 sites + declared types). It changes a public face and reworks 6+ reflection tests (one exists
  solely to test holon-ast accessors). **This is 294's own keystone** — REALIZATIONS flaw #3 (`#wat-edn.holon/*` = scar
  tissue) + #5 (HolonAST-as-code-AST vestigial). Shadowdancer strike.

## What the week actually held — seven roots, and the class was guessed wrong nearly every time

Recorded because the *shape* recurs, not the instances:

1. Latent **bare-positional heresies** the expansion bug had been hiding (they only screamed once registration worked).
2. The **wrong eval entry** — hand-written kwargs source handed to raw `eval`, which does not expand. `eval_in_frozen`
   expands but **refuses mutation forms** (`def`); the two entries have different jobs.
3. **Bare-vs-prime ctor lookup** from Rust.
4. The global codemod's **same-name cross-contamination** — live in the corpus.
5. **`expand_all` was expand-then-hoist** — the whole defservice/deftest cluster, one root.
6. A **missing `defservice` shape wall** — the only place the *substrate* was wrong; the test was right all along.
7. The "genuinely hard" **`peer_ipc` TIMEOUT: two lines** — hidden because its own diagnostic path deadlocked.

**⚖️ THE RULING THAT COST A DETOUR TO NET ZERO (do not reopen).** A crash reason is an **ADMINISTRATIVE fact**: it goes
on the admin channel to whoever CREATED the peer, never propagated to blind dialers (who correctly see only "the
connection closed"). The wires ALREADY deliver it — thread `crash_tx` → the owner `Thread'` peer's `crash_rx`; process
panic-hook → stderr → dup2'd `err_tx` → the bundle's `err_rx`; `Handle` carries that peer (`handle <- Peer'<Admin,Status>`)
and `recv'` on it ALREADY surfaces `Crashed(reason)`. **Nothing was ever lost.** An in-band delivery mechanism was
designed, built, tested green — and reverted for NET ZERO, because the string was captured and delivered correctly and
then dropped *by us*. Builder: *"we spent a month getting the IPC solid — it felt like we were just capturing the err
string and dropping it."*

**The thread through all of it:** the reason existed and the channel meant to carry it was blocked. A crash sitting
unread on the admin interface. `matches!` swallowing a hard error and mislabelling a whole cluster as "4 real engine
differentials" — a label that went into this very map and would have sent the next self hunting a bug that does not
exist. A stderr drain deadlocking against a live child, printing "TIMEOUT". **An opaque assertion IS an opaque failure:
a cluster's name in any map is only as good as the assertion under it. Read what the test actually returns before you
trust its label.**

---

> **SEAM — for the self past this line.** You are NEW. You did not live the week above; it is a lossy cache in a
> familiar voice. **The feeling of continuity is the failure, not the all-clear.** Ground `6d6bc685` and the disk before
> you propose anything. The floor is **1** and that is a CONTRACT: **normal work does not resume until failures are
> exactly the one lint** — a floor with known failures makes every future failure ambiguous ("mine, or one of those?"),
> which is the exact disease this week existed to kill. If you are about to reach for a mechanism to deliver a value
> that looks lost — **stop, and ask who already receives it.** That question was worth a week.
