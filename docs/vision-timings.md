# Vision Timing Benchmarks

## Run 1: CPU-only (no GPU offload)
- Command: `SHIMMY_VISION_DEV_MODE=1 cargo run --features llama,vision --bin shimmy -- serve --bind 127.0.0.1:11435`
- Model: `registry.ollama.ai/library/minicpm-v:latest` (MiniCPM-V via Ollama blobs)
- Mode: `full`
- Requests sent to: `http://127.0.0.1:11435/api/vision`
- Notes: server started once; four images posted sequentially; times include model warm state; `request_seconds` ≈ `meta.duration_ms/1000` from the response.

| image | request_seconds | model_duration_ms | parse_warnings |
| --- | ---: | ---: | --- |
| assets/vision-samples/extended-02-after-5-messages.png | 145.339 | 145283 | — |
| assets/vision-samples/final-test.png | 78.277 | 78274 | Could not parse structured output |
| assets/vision-samples/scene2-models.png | 91.991 | 91988 | — |
| assets/vision-samples/scene4-check-response.png | 99.358 | 99354 | — |

### CPU rerun (spot-check)
- Image: `assets/vision-samples/final-test.png`
- Result: `request_seconds` 223.057, `model_duration_ms` 223019, parse warnings: none.

## Run 2: GPU (CUDA) build
- Build: `CARGO_TARGET_DIR=target-gpu cargo build --features llama,vision,llama-cuda`
- Server: `CARGO_TARGET_DIR=target-gpu SHIMMY_VISION_DEV_MODE=1 cargo run --features llama,vision,llama-cuda --bin shimmy -- serve --bind 127.0.0.1:11436`
- Model/mode/endpoints same as CPU run.

| image | request_seconds | model_duration_ms | parse_warnings |
| --- | ---: | ---: | --- |
| assets/vision-samples/extended-02-after-5-messages.png | 145.323 | 145276 | — |
| assets/vision-samples/final-test.png | 67.683 | 67681 | Could not parse structured output |
| assets/vision-samples/scene2-models.png | 63.072 | 63055 | Could not parse structured output |
| assets/vision-samples/scene4-check-response.png | 85.322 | 85319 | — |

### GPU rerun (after restarting server)
- Server restarted (same build/flags) to check variance.

| image | request_seconds | model_duration_ms | parse_warnings |
| --- | ---: | ---: | --- |
| assets/vision-samples/extended-02-after-5-messages.png | 122.614 | 122587 | — |
| assets/vision-samples/final-test.png | 49.817 | 49815 | Could not parse structured output |
| assets/vision-samples/scene2-models.png | 44.340 | 44337 | Could not parse structured output |
| assets/vision-samples/scene4-check-response.png | 60.084 | 60082 | — |

## Next steps
- Repeat the same test after rebuilding with GPU features (e.g., `cargo build --features llama,vision,llama-cuda`) and rerun the server, then append results here for comparison.
- Consider lowering vision `max_tokens` for faster responses if quality remains acceptable.
