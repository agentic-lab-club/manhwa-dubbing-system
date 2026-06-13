from __future__ import annotations

import argparse
import json
from pathlib import Path

from .pipeline import run_worker


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the manhwa dubbing ML worker.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_parser = subparsers.add_parser("run", help="Run ML stages for an existing job manifest.")
    run_parser.add_argument("--manifest", required=True, type=Path)
    run_parser.add_argument("--job-dir", required=True, type=Path)
    run_parser.add_argument("--synthesize-voice", action="store_true")

    args = parser.parse_args()

    if args.command == "run":
        result = run_worker(args.manifest, args.job_dir, args.synthesize_voice)
        print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
