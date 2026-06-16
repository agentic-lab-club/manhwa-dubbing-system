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
- `music_selection.json` from the local `assets/music` library
- `status.json`

Run the Rust HTTP API:

```powershell
cd backend-rust
cargo run -- --addr 127.0.0.1:8000
```

Open the browser UI:

```text
http://127.0.0.1:8000/
```

The UI lets users configure input folders, translation language, voice, ML worker command, music mood/library, rendering, and live job metrics.

Delegate model stages to the Python ML worker:

```powershell
cd backend-rust
cargo run -- --once --ml-command "poetry run manhwa-ml" --synthesize-voice
```

Use the local music library for background overlay:

```powershell
poetry run manhwa-ml music --library assets/music --list
cd backend-rust
cargo run -- --once --music-dir ../assets/music --music-mood dramatic
```

Prepare Python utilities with Poetry:

```powershell
poetry install
poetry run build-video
poetry run manhwa-ml --help
```

## Docker

Run the whole project with one command:

```powershell
docker compose up --build
```

Open:

```text
http://127.0.0.1:8000/
```

The compose setup persists generated jobs in `output/`, mounts `assets/music/` for background tracks, and runs the Rust backend with the Python ML worker enabled.
