# Shimmy Vision Specification

## Goals
- Add a vision capability to Shimmy (CLI + HTTP) that mirrors Seer's functionality (OCR, layout, visual, interaction) and outputs the same JSON schema.
- Ship behind a paid feature: build-time gated by a Cargo feature `vision` (off by default) and runtime-gated by Keygen licensing. No license → no vision execution.
- Default to a small, proven model (`minicpm-v` via shimmy/llama.cpp) for fast local runs; allow overriding via flag/env.
- Keep outputs strictly structured JSON; robust parsing and guardrails for malformed model output.
- Optimize for interoperability: Design as a standalone vision service integrable with any AI system (Cursor, KIRO, Copilot, etc.) via HTTP API, with minimal dependencies and high performance.
- Extend beyond static images: Support web-based vision (e.g., DOM mapping for interactive elements) to enable richer analysis of dynamic content.

## Build & Feature Gating
- Cargo feature: `vision` (not in default). All vision code (deps, endpoints, CLI) wrapped in `#[cfg(feature = "vision")]`.
- Binaries built without `vision` contain no vision code or deps.
- In code paths, add `cfg` shims returning 404/feature-disabled errors when `vision` is off.

## Licensing (Keygen + Stripe)
- Input: `SHIMMY_LICENSE_KEY` env or `--license <key>` CLI flag. Stored in-memory only (optional cache file with OS-appropriate permissions).
- Validate via Keygen `/licenses/actions/validate-key` on first use; cache signed token with expiry; revalidate on expiry. Short offline grace allowed (configurable, e.g., 24h) with cached token.
- Enforce per-request: vision endpoints/CLI require a valid license token before running the model. On failure: 402/403 with terse JSON error.
- Entitlements: Keygen metadata fields (e.g., `vision=true`, `monthly_cap=1000`). Shimmy tracks usage counters (in-memory + optional persisted file) and rejects over-cap with 402.
- Stripe: payment → webhook/script creates Keygen license; no third-party runtime service required.

## Models
- Default model: MiniCPM-V 2.6 GGUF, quant `Q4_K_M` (~4.68 GB) from `openbmb/MiniCPM-V-2_6-gguf`.
  - Model URL: `https://huggingface.co/openbmb/MiniCPM-V-2_6-gguf/resolve/main/ggml-model-Q4_K_M.gguf`
  - Model SHA256: `3a4078d53b46f22989adbf998ce5a3fd090b6541f112d7e936eb4204a04100b1`
  - Vision projector URL: `https://huggingface.co/openbmb/MiniCPM-V-2_6-gguf/resolve/main/mmproj-model-f16.gguf` (~1.04 GB)
  - Projector SHA256: `4485f68a0f1aa404c391e788ea88ea653c100d8e98fe572698f701e5809711fd`
  - Cache path convention: Use OS-standard data directories (Windows: `%LOCALAPPDATA%/shimmy/models/vision/`, macOS: `~/Library/Application Support/shimmy/models/vision/`, Linux: `~/.local/share/shimmy/models/vision/`). No Ollama dependency.
- Override: `--model <path-or-name>` CLI flag; HTTP body `model` field; env fallback `SHIMMY_VISION_MODEL`.
- Model requirements: vision-capable GGUF, supports image tokens. Validate at startup with a probe (metadata shows vision support) and return a clear error if incompatible.
- First-run behavior: if the GGUF or projector is missing, default to auto-download from the pinned URLs with a user prompt in CLI (yes/no); server mode auto-downloads unless disabled. Flags/env: `--auto-download` / `--no-auto-download` (or `SHIMMY_VISION_AUTO_DOWNLOAD=true/false`). If auto-download is disabled, fail fast with a clear message and the exact download URLs.

## CLI
- Command: `shimmy vision --image <path> [--mode full|ocr|layout|brief|web] [--output json|pretty] [--model <name>] [--timeout <ms>] [--license <key>] [--raw] [--url <url> for web mode]`
- Defaults: mode=full, output=json, model=minicpm-v, timeout=180000 ms.
- Behavior: load image (or URL for web), run prompt for mode, stream completion, parse JSON, emit structured output. On parse failure: return 502 and include raw text if `--raw`.
- Exit codes: 0 success, 2 invalid license/feature disabled, 3 model/load error, 4 JSON parse error, 5 timeout.
- Interactive first-run: if model/projector missing and auto-download enabled, prompt user yes/no; if yes, download into cache and proceed; if no, fail with instructions to set `--model` or `SHIMMY_VISION_MODEL` and the download URLs. Non-interactive CLI respects `--auto-download/--no-auto-download`.

## HTTP API
- Endpoint: `POST /api/vision` (behind `vision` feature).
- Request: multipart/form-data (`image` field) or JSON with `image_base64` and optional `filename` for type detection. For web mode: `url` field instead of image. Fields: `mode`, `model`, `timeout_ms`, `raw` (bool).
- Response 200: JSON schema (textBlocks, layout, visual, interaction, meta {model, backend, duration_ms}). For web mode: includes `dom_map`.
- Errors:
  - 400 bad input (missing image/mode), 415 unsupported image type
  - 402 license missing/invalid/over-cap; 403 forbidden/feature-disabled when `vision` off or license blocked
  - 422 parse failure (returns truncated `raw_model_output` when `raw=true`, sets `meta.parse_warnings`)
  - 502 model/backend failure; 504 timeout (with cancellation triggered)
  - 503 model missing and auto-download disabled/failed (includes download URLs)

## Prompting (port from Seer)
- Modes: `full`, `ocr`, `layout`, `brief`, `web` mapped from `vision-prompts.js` (extend for web).
- Base instructions: "Return ONLY valid JSON, no code fences, keys: textBlocks, layout, visual, interaction."
- Mode specifics:
  - ocr: focus on textBlocks only.
  - layout: focus on layout.theme/regions/keyUIelements.
  - brief: single description under visual.description.
  - full: include all fields plus example schema.
  - web: include dom_map with interactive elements (buttons, links, inputs) and their positions/attributes.
- Implementation: store prompts in Rust constants/templates; include system + user content. Keep output schema reminder verbatim.
- Inference defaults (tuned for structured JSON): temperature 0.7, top_p 0.9, top_k 50, repeat_penalty 1.05, max_tokens ~768 (configurable), stop tokens none by default.

## Schema (Rust types)
- `TextBlock { text: String, confidence: Option<f32> }`
- `Layout { theme: Option<String>, regions: Vec<Region>, key_ui_elements: Vec<UIElement> }`
- `Region { name: String, description: String }`
- `UIElement { name: String, element_type: String }`
- `Visual { background: Option<String>, accent_colors: Vec<String>, contrast: Option<Contrast>, description: Option<String> }`
- `Contrast { ratio: Option<f32>, compliant: Option<bool>, issues: Vec<String> }`
- `Interaction { description: Option<String> }`
- `DomElement { tag: String, id: Option<String>, class: Option<String>, text: Option<String>, position: Rect, attributes: HashMap<String, String> }`
- `Rect { x: f32, y: f32, width: f32, height: f32 }`
- `Meta { model: String, backend: String, duration_ms: u64, parse_warnings: Option<Vec<String>> }`
- `VisionResponse { image_path: Option<String>, url: Option<String>, mode: String, text_blocks, layout, visual, interaction, dom_map: Option<Vec<DomElement>>, meta, raw_model_output: Option<String> }`
- Parsing: strict serde; add a lenient fallback (similar to `vision-schema.js`) to recover when models emit Markdown/extra text; if recovered, mark `meta.parse_warnings`.

## Image handling
- Accept PNG/JPEG/WebP; reject others with 415. Detect type from magic bytes, not just extension.
- Size: no hard cap by default (buyer-beware). Optional soft limit flags (`--max-image-mb`, `--max-dim`) can downscale or reject; warn when images exceed a reasonable threshold (e.g., >12 MP) before processing. Downscale policy (when enabled): resize to max side ~2048px with a quality filter (e.g., `fast_image_resize` or `image` + Lanczos3).

## Advanced Features
- **Web-Based Vision**: Optional mode `web` for analyzing web pages. Input: URL (CLI/HTTP). Uses headless browser to capture screenshot + extract DOM map (elements with positions, types, attributes). Outputs enhanced schema with `dom_map` field (array of interactive elements). Requires additional dep (e.g., `chromiumoxide` or `playwright`). Buyer-beware for large/complex pages.
- **Interoperability**: HTTP API optimized for third-party AI integrations (Cursor, KIRO, Copilot). Standard JSON I/O, optional webhook callbacks for async results. Provide open-source SDKs (Rust, JS, Python) with examples for plugin development.
- **Performance Enhancements**: GPU-accelerated image preprocessing (via `wgpu` or CUDA), model quantization variants (Q2_K for speed), batch processing for multiple images/files. Optional model warm-up for low-latency startups.

## Performance
- Default model choice: `minicpm-v` to minimize load time and VRAM. Document memory footprint and expected latency.
- Caching: keep model loaded between requests; optional warm-start flag `--preload-vision-model` or auto-load on first vision call.
- Token limits: set reasonable max tokens for vision generation to avoid long runs (configurable; default ~512-1024 tokens).
- Timeouts: default 180s; enforce server-side cancellation and surface 504.

## Configuration
- Env: `SHIMMY_VISION_MODEL`, `SHIMMY_LICENSE_KEY`, `SHIMMY_VISION_TIMEOUT_MS`, `SHIMMY_VISION_MAX_IMAGE_MB`, `SHIMMY_VISION_MAX_DIM`, `SHIMMY_VISION_ALLOW_OFFLINE_SECONDS`.
- CLI flags override env.
- Server config file support (if present elsewhere) can add a `vision` section.
- Auto-download: `SHIMMY_VISION_AUTO_DOWNLOAD` (default true for server; CLI prompts). Cache dir configurable via env if desired.

## Logging & Observability
- Log at info: request start/end, model name, duration_ms, mode.
- Warn on parse recovery, near-cap usage, or license grace mode.
- No image bytes in logs; include image hash or size only.
- Warn if image exceeds soft thresholds; log when auto-download occurs or is skipped.

## Tests
- Unit: prompt builders per mode; schema serde; lenient parser recovery cases.
- Integration (feature `vision`): mock model returning canned JSON to verify CLI and HTTP; license validation stub to simulate valid/invalid licenses; image type validation; timeout paths; over-cap rejection.
- If hardware permits: golden test against `minicpm-v` on a small fixture image (optional, feature-gated and skipped in CI unless enabled).
- Mock backend shape: return JSON body matching schema and a malformed/markdown variant for parse recovery; fixture image (small PNG) in repo. Gate real-model tests behind an opt-in feature/env.
- Performance Benchmarks: Latency <5s for 1MP image on default model; accuracy >90% on OCR/layout tasks (measured against fixture datasets).
- Security: Input validation (no path traversal, size limits enforced); rate limiting on HTTP (configurable, e.g., 10 req/min per IP); no sensitive data in logs.

## Risks & Mitigations
- Model Download: Large files (~5.7 GB); mitigate with resumable downloads, progress bars, and offline mode.
- Web Mode Complexity: Headless browser adds deps/runtimes; mitigate by making it optional, with clear error if Chromium unavailable.
- Licensing Overhead: Keygen calls add latency; mitigate with caching and offline grace period.
- Interoperability: SDK maintenance; mitigate with community contributions and automated tests for API compatibility.

## Success Metrics
- Core: CLI/HTTP work end-to-end with real model; <1% parse failures; licensing enforces caps.
- Performance: 95% of requests <3s; VRAM usage <8GB on default model.
- Adoption: Easy integration (SDKs used in at least 3 external projects); positive third-party audit feedback.

## Backward/compat
- When `vision` is off: CLI subcommand and HTTP route are absent; attempts return a clear message (404 or feature-disabled JSON).
- When `vision` is on but license missing/invalid: refuse with 402/403 before model load.

## User-Facing Messages (Branded as "Shimmy Vision")
- License errors: "Shimmy Vision requires a valid license. Visit [pricing URL] to purchase."
- Feature disabled: "Shimmy Vision is not enabled in this build. Contact support for access."
- Auto-download prompt: "Shimmy Vision needs to download models (~5.7 GB). Proceed? (y/N)"
- CLI help: "Shimmy Vision: Analyze images and web pages with AI. Usage: shimmy vision [options]"
- HTTP errors: Include "Shimmy Vision" in JSON error messages for clarity.

## Delivery plan
- Phase 1: Add `vision` feature flag, Rust schema types, prompt templates, CLI/HTTP skeletons, license validation stub (no real model call yet). Tests for shape and feature gating.
- Phase 2: Wire model backend with `minicpm-v` default; add image loading/downscale; JSON parse + recovery; end-to-end with mock model. Add basic interoperability (HTTP API docs/SDK stubs).
- Phase 3: Keygen client implementation + usage metering + Stripe webhook script (out-of-binary) for provisioning. Add performance optimizations (GPU accel, batching).
- Phase 4: Web-based vision (DOM mapping with headless browser). Full interoperability SDKs for Cursor/KIRO/Copilot. Docs: usage, defaults, model pull instructions, licensing FAQ; optional VS Code extension update to call `/api/vision`.
