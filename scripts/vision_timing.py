import argparse
import base64
import json
import os
import time
import urllib.request


DEFAULT_IMAGES = [
	"assets/vision-samples/extended-02-after-5-messages.png",
	"assets/vision-samples/final-test.png",
	"assets/vision-samples/scene2-models.png",
	"assets/vision-samples/scene4-check-response.png",
]


def post_vision(url: str, image_path: str, mode: str, timeout_ms: int, socket_timeout_s: int) -> dict:
	with open(image_path, "rb") as f:
		image_bytes = f.read()

	body = {
		"mode": mode,
		"timeout_ms": timeout_ms,
		"filename": os.path.basename(image_path),
		"image_base64": base64.b64encode(image_bytes).decode("ascii"),
	}

	req = urllib.request.Request(
		url,
		data=json.dumps(body).encode("utf-8"),
		headers={"Content-Type": "application/json"},
		method="POST",
	)

	with urllib.request.urlopen(req, timeout=socket_timeout_s) as resp:
		return json.loads(resp.read().decode("utf-8"))


def main() -> int:
	parser = argparse.ArgumentParser(description="Shimmy vision timing runner")
	parser.add_argument(
		"--url",
		default="http://127.0.0.1:11435/api/vision",
		help="Vision endpoint URL (default: http://127.0.0.1:11435/api/vision)",
	)
	parser.add_argument("--mode", default="full", help="Vision mode (default: full)")
	parser.add_argument(
		"--timeout-ms",
		type=int,
		default=600000,
		help="Server-side timeout_ms to send (default: 600000)",
	)
	parser.add_argument(
		"--socket-timeout-s",
		type=int,
		default=900,
		help="Client socket timeout in seconds (default: 900)",
	)
	parser.add_argument(
		"images",
		nargs="*",
		help="Image paths (default: assets/vision-samples/* from docs/vision-timings.md)",
	)
	args = parser.parse_args()

	images = args.images or DEFAULT_IMAGES

	rows = []
	for image_path in images:
		start = time.perf_counter()
		data = post_vision(args.url, image_path, args.mode, args.timeout_ms, args.socket_timeout_s)
		elapsed = time.perf_counter() - start
		meta = data.get("meta") or {}
		parse_warnings = meta.get("parse_warnings")
		if isinstance(parse_warnings, list):
			parse_warnings = "; ".join(parse_warnings)
		rows.append(
			{
				"image": image_path,
				"request_seconds": round(elapsed, 3),
				"model_duration_ms": meta.get("duration_ms"),
				"backend": meta.get("backend"),
				"parse_warnings": parse_warnings or "—",
			}
		)

	print("image,request_seconds,model_duration_ms,backend,parse_warnings")
	for row in rows:
		print(
			f"{row['image']},{row['request_seconds']},{row['model_duration_ms']},{row['backend']},{row['parse_warnings']}"
		)

	return 0


if __name__ == "__main__":
	raise SystemExit(main())
