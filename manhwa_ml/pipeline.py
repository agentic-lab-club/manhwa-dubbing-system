from __future__ import annotations

from pathlib import Path
from typing import Any

from .io_utils import as_path, read_json, write_json, write_text
from .ocr import run_ocr
from .panels import detect_panels
from .recap import generate_recap
from .tts import synthesize_windows_sapi


def run_worker(manifest_path: Path, job_dir: Path, synthesize_voice: bool = False) -> dict[str, Any]:
    manifest_path = manifest_path.resolve()
    job_dir = job_dir.resolve()
    manifest = read_json(manifest_path)
    project_root = _project_root_from_job_dir(job_dir)
    pairs = _resolve_pairs(manifest.get("pairs", []), project_root)
    language = str(manifest.get("language", "eng"))
    style = str(manifest.get("recap_style", "engaging"))
    texts_dir = _resolve_optional_path(as_path(manifest.get("texts_dir")), project_root)

    artifacts: list[dict[str, Any]] = []

    ocr_items, ocr_status, ocr_message = run_ocr(pairs, language, texts_dir)
    ocr_path = job_dir / "ocr.json"
    write_json(ocr_path, {"items": ocr_items})
    artifacts.append(_artifact("ocr", ocr_status, ocr_path, ocr_message))

    recap = generate_recap(ocr_items, style)
    recap_path = job_dir / "recap.txt"
    write_text(recap_path, recap)
    artifacts.append(_artifact("recap", "completed", recap_path, "local extractive recap generated"))

    panels_payload, panel_status, panel_message = detect_panels(pairs)
    panels_path = job_dir / "panels.json"
    write_json(panels_path, panels_payload)
    artifacts.append(_artifact("panel_detection", panel_status, panels_path, panel_message))

    tts_request_path = job_dir / "tts_request.json"
    write_json(
        tts_request_path,
        {
            "provider": "windows-sapi-or-external",
            "voice": manifest.get("voice", "default"),
            "text_path": str(recap_path),
        },
    )
    if synthesize_voice or bool(manifest.get("synthesize_voice", False)):
        wav_path = job_dir / "narration.wav"
        tts_status, tts_message = synthesize_windows_sapi(recap, wav_path)
        artifacts.append(_artifact("tts", tts_status, wav_path if wav_path.exists() else tts_request_path, tts_message))
    else:
        artifacts.append(
            _artifact(
                "tts",
                "ready",
                tts_request_path,
                "TTS request prepared; enable synthesize_voice to generate narration.wav",
            )
        )

    audio_mix_path = job_dir / "audio_mix.json"
    write_json(audio_mix_path, _audio_mix_payload(manifest, pairs, job_dir))
    artifacts.append(_artifact("audio_mix", "ready", audio_mix_path, "audio mix plan created"))

    worker_status = {"status": "completed", "artifacts": artifacts}
    write_json(job_dir / "ml_worker_status.json", worker_status)
    return worker_status


def _audio_mix_payload(manifest: dict[str, Any], pairs: list[dict[str, Any]], job_dir: Path) -> dict[str, Any]:
    narration = job_dir / "narration.wav"
    return {
        "strategy": "narration-plus-source-audio-plan",
        "narration": str(narration) if narration.exists() else None,
        "background_music": manifest.get("background_music"),
        "source_audio": [{"index": pair["index"], "path": pair["audio"], "volume": 1.0} for pair in pairs],
        "background_volume": 0.25,
        "narration_volume": 1.0,
    }


def _project_root_from_job_dir(job_dir: Path) -> Path:
    # Expected layout: <project>/output/jobs/<job-id>.
    if len(job_dir.parents) >= 3 and job_dir.parent.name == "jobs":
        return job_dir.parents[2]
    return Path.cwd()


def _resolve_pairs(pairs: list[dict[str, Any]], project_root: Path) -> list[dict[str, Any]]:
    resolved = []
    for pair in pairs:
        item = dict(pair)
        item["image"] = str(_resolve_path(Path(str(pair["image"])), project_root))
        item["audio"] = str(_resolve_path(Path(str(pair["audio"])), project_root))
        resolved.append(item)
    return resolved


def _resolve_optional_path(path: Path | None, project_root: Path) -> Path | None:
    if path is None:
        return None
    return _resolve_path(path, project_root)


def _resolve_path(path: Path, project_root: Path) -> Path:
    if path.exists():
        return path
    candidate = project_root / path
    if candidate.exists():
        return candidate
    stripped = _strip_leading_parents(path)
    candidate = project_root / stripped
    if candidate.exists():
        return candidate
    return path


def _strip_leading_parents(path: Path) -> Path:
    parts = list(path.parts)
    while parts and parts[0] == "..":
        parts.pop(0)
    return Path(*parts) if parts else path


def _artifact(stage: str, status: str, path: Path, message: str) -> dict[str, Any]:
    return {"stage": stage, "status": status, "path": str(path), "message": message}
