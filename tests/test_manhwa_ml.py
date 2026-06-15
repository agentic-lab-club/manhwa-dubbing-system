from __future__ import annotations

from manhwa_ml.panels import _split_long_page
from manhwa_ml.recap import split_sentences
from manhwa_ml.music import MusicLibrary


def test_split_sentences() -> None:
    assert split_sentences("One. Two!") == ["One.", "Two!"]


def test_split_long_page_keeps_short_page_whole() -> None:
    assert _split_long_page(1000, 700, 16 / 9) == [(0, 0, 1000, 700)]


def test_split_long_page_segments_tall_page() -> None:
    boxes = _split_long_page(1000, 4000, 16 / 9)
    assert len(boxes) > 1
    assert boxes[0][0] == 0


def test_music_library_scans_audio(tmp_path) -> None:
    music_dir = tmp_path / "music"
    music_dir.mkdir()
    (music_dir / "dramatic_theme.mp3").write_bytes(b"")

    track = MusicLibrary(music_dir).select("dramatic")

    assert track is not None
    assert track.mood == "dramatic"
