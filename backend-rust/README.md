# Minimal Rust Backend

This backend is the first runnable MVP slice of the manhwa dubbing system.

It currently supports:

- scanning image and audio folders;
- pairing files by sorted order or by matching file stem;
- OCR stage via `.txt` sidecars or installed `tesseract`;
- local extractive recap generation;
- panel metadata generation with a full-page fallback;
- TTS request generation, with optional Windows SAPI `narration.wav`;
- audio mix planning with optional background music;
- creating `manifest.json`, `ocr.json`, `recap.txt`, `panels.json`, `tts_request.json`, `audio_mix.json`, and `status.json`;
- exposing minimal HTTP endpoints;
- optional `ffmpeg` rendering when `ffmpeg` is installed.

## Run as API

```powershell
cd backend-rust
cargo run -- --addr 127.0.0.1:8000
```

Endpoints:

- `GET /health`
- `POST /api/v1/dubbing/start`
- `GET /api/v1/status/<job_id>`
- `GET /api/v1/result/<job_id>`

Example body:

```json
{
  "images_dir": "../Tlok_Backend/<images-dir>",
  "audio_dir": "../Tlok_Backend/zvyki",
  "texts_dir": "../data/texts",
  "background_music": "../data/music/theme.mp3",
  "output_dir": "../output",
  "pairing": "sequential",
  "language": "eng",
  "recap_style": "engaging",
  "voice": "default",
  "synthesize_voice": false,
  "render": false
}
```

## Run once

```powershell
cd backend-rust
cargo run -- --once
```

Add `--render` to generate `result.mp4`. Rendering requires `ffmpeg` in PATH.

Add `--synthesize-voice` to generate `narration.wav` through Windows SAPI.

Use `--texts <dir>` when OCR text has already been prepared as sidecar `.txt` files with names matching the images.

## Python ML Worker

The Rust backend can delegate ML stages to the Poetry-managed Python worker:

```powershell
cd backend-rust
cargo run -- --once --ml-command "poetry run manhwa-ml" --synthesize-voice
```

The worker writes:

- `ocr.json`
- `recap.txt`
- `panels.json`
- `tts_request.json` or `narration.wav`
- `audio_mix.json`
- `ml_worker_status.json`
