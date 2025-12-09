# ⚠️ CRITICAL SERVER RULE: NEVER cancel background servers with Ctrl+C! Use `&` or separate terminals!
# If you start a server (shimmy serve, python -m http.server, etc.) and then cancel it, IT WON'T RUN ANYMORE.
# Either use trailing `&` for background OR use different terminal tabs. You've done this mistake 12+ times today!

# 📋 CURRENT STATUS - December 9, 2025

## Active Work: Shimmy Vision Development 🎯
**Focus**: Implement Shimmy Vision as a feature-gated, Keygen-licensed CLI + HTTP for image/web analysis, mirroring Seer functionality.
**Workflow**:
1. **Phase 1**: Add `vision` feature, Rust schemas, CLI/HTTP skeletons, license stubs.
2. **Phase 2**: Wire model backend (MiniCPM-V), image processing, JSON parsing.
3. **Phase 3**: Keygen licensing, usage metering, performance opts.
4. **Phase 4**: Web/DOM mode, SDKs for Cursor/KIRO/Copilot integration.
**Branch**: `feature/shimmy-vision-phase1` – Work here only; test with `./scripts/dev-test.sh`; release gates with `./scripts/dry-run-release.sh`.

### Development Workflow Rules:
- **NEVER work on main**: Always use feature branches
- **Test before commit**: `./scripts/dev-test.sh` or `cargo test`
- **Release gates mandatory**: `./scripts/dry-run-release.sh` before PR
- **Clean commits**: `cargo fmt`, `cargo clippy -- -D warnings`
- **Detailed PRs**: Include issue link, reproduction steps, test results

---

# Copilot / AI Assistant Operating Guide for Shimmy Vision

This file teaches AI assistants how to work on Shimmy Vision. Keep replies lean, perform actions directly, focus on Vision implementation.

## CRITICAL RULES - NEVER VIOLATE

### 1. NEVER Print Fake Validation
**WRONG**: `echo "✅ Build successful"`
**RIGHT**: Actually check: `ls -lh target/release/shimmy.exe && echo $? && ./shimmy --version`

### 2. NEVER Use `!` in Bash Commands
**WRONG**: `echo "Build finished!"` or `rg "println!" src/`
**RIGHT**: `printf "%s\n" "Build finished"` or `rg 'println\!' src/`

### 3. ALWAYS Use `&` for Background Processes
**WRONG**: Long-running commands without `&`
**RIGHT**: `command args &`

### 4. ZERO TOLERANCE FOR WARNINGS
Fix ALL warnings immediately.

### 5. Python Command is `py` NOT `python3`
**WRONG**: `python3 script.py`
**RIGHT**: `py script.py`

### 6. Read Documentation BEFORE Trial-and-Error
Use `fetch_webpage` for official docs.

## Mission
Shimmy is a single-binary local inference shim with optional Vision feature: CLI + HTTP for image/web analysis (OCR, layout, visual, interaction, DOM mapping). Paid via Keygen, interoperable with Cursor/KIRO/Copilot.

## Core Components
- `src/engine/llama.rs`: llama.cpp backend (feature `llama`).
- `src/api.rs`: `/api/generate` + `/api/vision` (POST, JSON) with SSE/WebSocket.
- `src/server.rs`: axum server.
- `src/templates.rs`: prompt templates.
- `src/model_registry.rs`: model registry.
- `src/cli.rs` + `src/main.rs`: CLI (serve, list, probe, bench, generate, vision).
- `src/vision.rs`: Vision schemas, processing (feature `vision`).

## Build & Run
- Non-vision: `cargo run -- list`
- Vision: `cargo run --features llama,vision -- vision --image <path> --mode full`
- Serve: `cargo run --features llama,vision -- serve --bind 127.0.0.1:11435`
- Vision HTTP: `POST /api/vision {"image_base64": "...", "mode": "full"}`

Environment variables:
- `SHIMMY_LICENSE_KEY` (for vision)
- `SHIMMY_VISION_MODEL`, etc.

## Conventions
- Keep public API minimal & stable.
- Use owned `String` in callbacks.
- Unsafe limited; prefer additive changes.
- After Rust edits: `cargo build --features llama,vision`.

## Adding Features (Playbook)
1. Outline contract in commit/PR.
2. Add types/endpoints skeletons.
3. Add tests.
4. Build + test; fix warnings.
5. Update docs.

## Error Handling
HTTP codes: 400/402/403/422/502/504.

## Streaming Patterns
SSE/WebSocket for generation/vision.

## Performance Notes
Model latency dominates; optimize for <3s responses.

## Planned Enhancements (Vision-Focused)
- Web mode with DOM mapping.
- Interoperability SDKs.
- GPU accel, batching.

## Interaction Rules for AI Assistants
- Do work directly.
- Pause after 3–5 edits to summarize.
- Avoid speculative refactors.
- Request missing info once.

## Quality Gate
- Build success (`cargo build --features llama,vision`).
- CLI/HTTP work.
- No warnings.

## Upstream Contribution Protocol
**CRITICAL**: For llama-cpp-rs, etc.:
1. **NO AI SHORTCUTS** - Real code only
2. **NO STUBBING** - No placeholders
3. **VERIFY EVERYTHING** - Test in Shimmy first
4. **ACCURATE COMMIT MESSAGES**
5. **REVIEW BEFORE PUSH**
6. **PATIENCE** - Get it right

---
Keep this file focused on Vision; prune when features land.
