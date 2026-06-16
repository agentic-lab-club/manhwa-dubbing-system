# 🚀 Deployment Guide — AI Manhwa Dubbing System

## Local Development
```bash
poetry install
poetry run build-video
```

Minimal Rust backend:

```bash
cd backend-rust
cargo run -- --addr 127.0.0.1:8000
```

## Docker Deployment
```bash
docker compose up --build
```

Open the browser UI:

```text
http://127.0.0.1:8000/
```

Mounted directories:

```text
./output       -> /app/output
./assets/music -> /app/assets/music
./data         -> /app/data
./Tlok_Backend -> /app/Tlok_Backend
```

The image includes the Rust backend binary, static frontend, Python ML worker, `ffmpeg`, and Tesseract OCR.
