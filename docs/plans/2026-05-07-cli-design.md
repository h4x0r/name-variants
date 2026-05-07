# name-variants CLI Design

**Date:** 2026-05-07

## Goal

Add an `nv` command-line tool to the `name-variants` package — both Python (pip) and Rust (cargo install) — with identical command surface on both runtimes.

## Commands

### `nv lookup [NAME]`

Resolves a name to its canonical native-script key + all known variants.

- If NAME is given: resolve and print one line
- If no NAME: read stdin line by line, print one result per input line
- Known name: `KEY\tvariant1, variant2, ...` (tab-separated)
- Unknown name: `-`
- Always exits 0

```
$ nv lookup Chan
陈	chen, chan, tan, chin, zen, chern, ching

$ nv lookup Smith
-

$ printf "Chan\nPark\nSmith\n" | nv lookup
陈	chen, chan, tan, chin, zen, chern, ching
박	park, bak, pak
-
```

**Output format rationale:** tab separator between key and variants makes the output friendly to `cut -f1` / `cut -f2`. Native script character is self-identifying — no language label is applied (attaching labels like "chinese/" to a character used across multiple communities, scripts, and political entities would be contested and wrong).

### `nv match NAME1 NAME2`

Tests whether two names are variants of the same entity.

- Both resolve to the same key → prints `match`, exits 0
- Different keys, or either name unknown → prints `no match`, exits 1

```
$ nv match Chan Chen
match

$ nv match Chan Smith
no match
```

Designed for shell scripting: `if nv match "$a" "$b"; then ...`

### `nv --version`

Prints the package version.

## Architecture

Two independent implementations, identical UX:

| | Python | Rust |
|--|--|--|
| Source | `name_variants/cli.py` | `name-variants-rs/src/main.rs` |
| Argument parser | Click | Clap (derive API) |
| Installed via | `pip install name-variants` | `cargo install name-variants` |
| Binary | `nv` | `nv` |
| Backing lib | `name_variants.lookup_key`, `ALL_TABLES` | `name_variants::lookup_key`, `INDEX` |

## README Badges (following blazehash template)

```
[![PyPI](https://img.shields.io/pypi/v/name-variants.svg)](https://pypi.org/project/name-variants/)
[![Crates.io](https://img.shields.io/crates/v/name-variants.svg)](https://crates.io/crates/name-variants)
[![CI](https://github.com/h4x0r/name-variants/actions/workflows/ci.yml/badge.svg)](...)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/sponsor-h4x0r-ea4aaa?logo=github-sponsors)](https://github.com/sponsors/h4x0r)
```

## Script-Form Aliases in Output

`nv lookup Chan` should show `陈  陳, chen, chan, tan, ...` — the Traditional form `陳` must appear alongside the romanizations.

The fix is in the data, not the CLI: Traditional (and other script-form aliases) are added as **variants** inside the relevant table entries. For Chinese, each Simplified key gets its Traditional equivalent(s) prepended to the variants list:

```python
"陈": ["陳", "chen", "chan", "tan", "chin", "zen", "chern"],
```

This also makes `lookup_key("陳")` work directly — Traditional input resolves to the Simplified canonical key without needing opencc. The `test_all_romanizations_are_lowercase` test is unaffected because `"陳".lower() == "陳"` (CJK has no case).

The same applies to Japanese (kyūjitai forms), Korean (Hanja), and Arabic (alternate Unicode encodings) where relevant.

For the implementation plan, this is a data-layer task separate from the CLI task.

## Out of Scope

- Language group labels on output (politically contested)
- `--json` output (YAGNI — tab-separated is already scriptable)
- `stats` / `list` subcommands (YAGNI)
- Shell completions (future)
