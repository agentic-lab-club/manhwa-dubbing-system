from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any


def run_ocr(pairs: list[dict[str, Any]], language: str, texts_dir: Path | None) -> tuple[list[dict[str, Any]], str, str]:
    items: list[dict[str, Any]] = []
    used_tesseract = False
    used_sidecar = False

    for pair in pairs:
        image = Path(pair["image"])
        text, source = _read_sidecar(image, texts_dir)
        if text:
            used_sidecar = True
        else:
            text = _run_tesseract(image, language)
            if text:
                source = "tesseract"
                used_tesseract = True
            else:
                source = "unavailable"
                text = ""

        items.append(
            {
                "index": pair["index"],
                "image": str(image),
                "source": source,
                "text": text,
            }
        )

    if used_tesseract:
        return items, "completed", "OCR completed with tesseract CLI"
    if used_sidecar:
        return items, "completed", "OCR loaded from text sidecar files"
    return items, "fallback", "OCR fallback created empty records; install tesseract or add sidecar text files"


def _read_sidecar(image: Path, texts_dir: Path | None) -> tuple[str, str]:
    candidates: list[Path] = []
    if texts_dir:
        candidates.append(texts_dir / f"{image.stem}.txt")
    candidates.append(image.with_suffix(".txt"))

    for candidate in candidates:
        if candidate.is_file():
            text = candidate.read_text(encoding="utf-8").strip()
            if text:
                return text, "sidecar"
    return "", "unavailable"


def _run_tesseract(image: Path, language: str) -> str:
    try:
        result = subprocess.run(
            ["tesseract", str(image), "stdout", "-l", language],
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
    except OSError:
        return ""
    if result.returncode != 0:
        return ""
    return result.stdout.strip()
