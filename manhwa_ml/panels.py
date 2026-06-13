from __future__ import annotations

from pathlib import Path
from typing import Any

from .image_info import image_dimensions


def detect_panels(pairs: list[dict[str, Any]], target_ratio: float = 16 / 9) -> tuple[dict[str, Any], str, str]:
    panels: list[dict[str, Any]] = []

    for pair in pairs:
        image = Path(pair["image"])
        width, height = image_dimensions(image)
        boxes = _split_long_page(width, height, target_ratio)
        for seq, (x, y, w, h) in enumerate(boxes, start=1):
            panels.append(
                {
                    "page_index": pair["index"],
                    "panel_index": seq,
                    "image": str(image),
                    "bbox": {"x": x, "y": y, "width": w, "height": h},
                    "confidence": 0.55 if len(boxes) > 1 else 1.0,
                    "method": "aspect-ratio-split" if len(boxes) > 1 else "full-page",
                }
            )

    payload = {"method": "heuristic-ml-worker", "panels": panels}
    return payload, "fallback", "panel metadata generated with deterministic heuristics"


def _split_long_page(width: int, height: int, target_ratio: float) -> list[tuple[int, int, int, int]]:
    if width <= 0 or height <= 0:
        return [(0, 0, width, height)]
    target_height = max(1, int(width / target_ratio))
    if height <= target_height * 1.5:
        return [(0, 0, width, height)]

    boxes: list[tuple[int, int, int, int]] = []
    y = 0
    while y < height:
        h = min(target_height, height - y)
        if h < target_height * 0.35 and boxes:
            x0, y0, w0, h0 = boxes[-1]
            boxes[-1] = (x0, y0, w0, h0 + h)
        else:
            boxes.append((0, y, width, h))
        y += h
    return boxes
