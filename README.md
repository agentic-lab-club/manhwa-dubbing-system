# manhwa-dubbing-system

An automated AI-powered pipeline for creating professional manhwa recap videos with emotional voice-over narration and background music for YouTube content creation.

## Current runnable slice

- Rust backend: `backend-rust`
- Python environment manager: Poetry (`pyproject.toml`)

Run the minimal Rust pipeline:

```powershell
cd backend-rust
cargo run -- --once
```

The Rust MVP creates these job artifacts under `output/jobs/<job_id>`:

- `manifest.json` for input pairing
- `ocr.json` from Tesseract or `.txt` sidecars
- `recap.txt` from the local recap stage
- `panels.json` from panel detection metadata
- `tts_request.json`, or `narration.wav` with `--synthesize-voice`
- `audio_mix.json` for narration/source/background audio planning
- `status.json`

Run the Rust HTTP API:

```powershell
cd backend-rust
cargo run -- --addr 127.0.0.1:8000
```

Delegate model stages to the Python ML worker:

```powershell
cd backend-rust
cargo run -- --once --ml-command "poetry run manhwa-ml" --synthesize-voice
```

Prepare Python utilities with Poetry:

```powershell
poetry install
poetry run build-video
poetry run manhwa-ml --help
```
