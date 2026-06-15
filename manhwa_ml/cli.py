from __future__ import annotations

import argparse
import json
from pathlib import Path

from .music import MusicLibrary
from .pipeline import run_worker


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the manhwa dubbing ML worker.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_parser = subparsers.add_parser("run", help="Run ML stages for an existing job manifest.")
    run_parser.add_argument("--manifest", required=True, type=Path)
    run_parser.add_argument("--job-dir", required=True, type=Path)
    run_parser.add_argument("--synthesize-voice", action="store_true")

    music_parser = subparsers.add_parser("music", help="Inspect or select background music.")
    music_parser.add_argument("--library", type=Path, default=Path("assets/music"))
    music_parser.add_argument("--mood", default="neutral")
    music_parser.add_argument("--list", action="store_true")

    args = parser.parse_args()

    if args.command == "run":
        result = run_worker(args.manifest, args.job_dir, args.synthesize_voice)
        print(json.dumps(result, ensure_ascii=False, indent=2))
    elif args.command == "music":
        library = MusicLibrary(args.library)
        if args.list:
            result = [
                {
                    "id": track.id,
                    "title": track.title,
                    "path": str(track.path),
                    "mood": track.mood,
                    "tags": list(track.tags),
                    "license": track.license,
                    "volume": track.volume,
                }
                for track in library.tracks()
            ]
        else:
            track = library.select(args.mood)
            result = None if track is None else {"id": track.id, "path": str(track.path), "mood": track.mood}
        print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
