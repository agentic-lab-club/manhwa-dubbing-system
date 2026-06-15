from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .io_utils import read_json, write_json

SUPPORTED_AUDIO_EXTENSIONS = {".mp3", ".wav", ".ogg", ".m4a", ".aac", ".flac"}


@dataclass(frozen=True)
class MusicTrack:
    id: str
    path: Path
    mood: str = "neutral"
    tags: tuple[str, ...] = ()
    title: str = ""
    license: str = "unknown"
    volume: float = 0.25


class MusicLibrary:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.catalog_path = root / "library.json"

    def tracks(self) -> list[MusicTrack]:
        catalog_tracks = self._catalog_tracks()
        scanned_tracks = self._scan_tracks()
        known_paths = {track.path.resolve() for track in catalog_tracks if track.path.exists()}
        merged = list(catalog_tracks)
        merged.extend(track for track in scanned_tracks if track.path.resolve() not in known_paths)
        return merged

    def select(self, mood: str = "neutral", tags: list[str] | None = None) -> MusicTrack | None:
        tags = tags or []
        tracks = [track for track in self.tracks() if track.path.is_file()]
        if not tracks:
            return None

        def score(track: MusicTrack) -> tuple[int, str]:
            value = 0
            if track.mood.lower() == mood.lower():
                value += 10
            value += len(set(tag.lower() for tag in tags) & set(tag.lower() for tag in track.tags))
            return value, track.id

        return sorted(tracks, key=score, reverse=True)[0]

    def write_selection(self, output_path: Path, mood: str = "neutral", tags: list[str] | None = None) -> MusicTrack | None:
        track = self.select(mood, tags)
        payload: dict[str, Any] = {
            "music_dir": str(self.root),
            "mood": mood,
            "tags": tags or [],
            "selected": None,
        }
        if track:
            payload["selected"] = {
                "id": track.id,
                "title": track.title,
                "path": str(track.path),
                "mood": track.mood,
                "tags": list(track.tags),
                "license": track.license,
                "volume": track.volume,
            }
        write_json(output_path, payload)
        return track

    def _catalog_tracks(self) -> list[MusicTrack]:
        if not self.catalog_path.is_file():
            return []
        data = read_json(self.catalog_path)
        tracks = []
        for raw in data.get("tracks", []):
            rel_path = Path(str(raw.get("path", "")))
            path = rel_path if rel_path.is_absolute() else self.root / rel_path
            tracks.append(
                MusicTrack(
                    id=str(raw.get("id") or path.stem),
                    title=str(raw.get("title") or path.stem),
                    path=path,
                    mood=str(raw.get("mood") or data.get("default_mood") or "neutral"),
                    tags=tuple(str(tag) for tag in raw.get("tags", [])),
                    license=str(raw.get("license") or "unknown"),
                    volume=float(raw.get("volume", 0.25)),
                )
            )
        return tracks

    def _scan_tracks(self) -> list[MusicTrack]:
        if not self.root.is_dir():
            return []
        tracks = []
        for path in sorted(self.root.rglob("*")):
            if path.is_file() and path.suffix.lower() in SUPPORTED_AUDIO_EXTENSIONS:
                tracks.append(
                    MusicTrack(
                        id=path.stem,
                        title=path.stem.replace("_", " ").replace("-", " "),
                        path=path,
                        mood=_mood_from_name(path.stem),
                    )
                )
        return tracks


def _mood_from_name(name: str) -> str:
    lowered = name.lower()
    for mood in ("dramatic", "sad", "happy", "action", "calm", "mystery", "neutral"):
        if mood in lowered:
            return mood
    return "neutral"
