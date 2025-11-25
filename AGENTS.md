<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Repository Guidelines

## Project Structure & Modules
- Rust workspace in `crates/`: `industrydb-core` (traits/config/errors), database connectors per crate (`industrydb-postgres`, `industrydb-sqlite`, `industrydb-mssql`), and `industrydb-py` for PyO3 bindings. Workspace manifest: `Cargo.toml`.
- Python package in `python/industrydb` with config helpers, type stubs (`industrydb.pyi`), and PEP 561 marker. Python config: `pyproject.toml`, locks: `uv.lock`.
- Integration tests live in `tests/`; runnable examples in `examples/`; architecture notes in `ARCHITECTURE.md`; additional docs in `docs/` and specs under `openspec/`. Example settings: `example.config.toml`.

## Build, Test, and Development Commands
- Install tooling once: `just setup` (creates venv & syncs deps), `just install-sys-deps` if prompted for system libs.
- Fast checks: `just check` (Rust), `just lint` (Rust+Python), `just fmt` (format all), `just dev` (format+check+build loop).
- Build: `just build` (debug), `just build-release` (optimized), `just develop` (PyO3 dev install), `just wheel` / `just wheel-all` for distributable artifacts.
- Test: `just test` (all), `just test-rust`, `just test-python`, `just test-coverage` (reports coverage). Targeted Python: `just test-file <path>`. Generate docs: `just doc` or `just doc-all`.

## Coding Style & Naming Conventions
- Rust: follow `rustfmt` defaults (`just fmt-rust`), idiomatic 4-space indent, snake_case for functions/vars, UpperCamelCase for types; `clippy` via `just lint-rust`.
- Python: `ruff format` + `black` alternative (`just fmt-python`), 4-space indent, snake_case for modules/functions, PascalCase for classes, UPPER_SNAKE for constants. Static checks: `ruff check`, `mypy` via `just lint-python`.
- Keep modules small and single-purpose; prefer trait interfaces in Rust and protocol-style abstractions in Python to honor SRP and DI.

## Testing Guidelines
- Frameworks: `cargo test` for Rust crates; `pytest` for Python (runs against built extension). Aim to cover connectors, config parsing, and error paths; add regression tests alongside fixes.
- Naming: mirror target module with `_test.rs` or `test_*.py`; keep fixtures under `tests/` or module-adjacent. Use the sample DB via `just create-example-db` instead of production instances.
- Run `just test` or the language-specific targets before opening a PR; include coverage-sensitive cases when touching query planning or IO layers.

## Commit & Pull Request Guidelines
- Follow Conventional Commits as seen in history (`docs:`, `chore:`, `style:`, `ci:`, etc.). Keep scopes meaningful (e.g., `core`, `py`, `postgres`).
- Each PR should describe intent, key changes, and testing performed (Rust/Python commands). Link issues, attach logs or screenshots when relevant (e.g., docs site, example outputs). Avoid committing generated artifacts (`target/`, `.venv/`, `__pycache__/`); use `just clean`/`just clean-python` before publishing.
