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
docker-compose up --build -d
```
Example `docker-compose.yml`:
```yaml
version: '3.8'
services:
  backend:
    build: ./backend
    ports: ["8000:8000"]
    volumes:
      - ./data:/app/data
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - ELEVENLABS_API_KEY=${ELEVENLABS_API_KEY}
  frontend:
    build: ./frontend
    ports: ["3000:3000"]
```
