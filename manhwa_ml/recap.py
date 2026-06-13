from __future__ import annotations

from typing import Any


def generate_recap(ocr_items: list[dict[str, Any]], style: str, max_sentences: int = 14) -> str:
    source_lines = []
    for item in ocr_items:
        text = " ".join(str(item.get("text", "")).split())
        if text:
            source_lines.append(f"Page {item['index']}: {text}")

    if not source_lines:
        return (
            f"Style: {style}\n\n"
            "No OCR text was available. Add text sidecars or install Tesseract to generate a real story recap.\n\n"
            "Production note: local ML worker fallback."
        )

    source = "\n".join(source_lines)
    sentences = split_sentences(source)
    selected = sentences[:max_sentences] or [source]
    return (
        f"Style: {style}\n\n"
        + " ".join(selected)
        + "\n\nProduction note: local extractive recap from OCR text."
    )


def split_sentences(text: str) -> list[str]:
    sentences: list[str] = []
    current: list[str] = []
    for char in text:
        current.append(char)
        if char in ".!?\n":
            sentence = "".join(current).strip()
            if sentence:
                sentences.append(sentence)
            current = []
    tail = "".join(current).strip()
    if tail:
        sentences.append(tail)
    return sentences
