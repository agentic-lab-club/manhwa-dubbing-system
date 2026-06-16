FROM rust:1.83-bookworm AS rust-builder

WORKDIR /build
COPY backend-rust/Cargo.toml backend-rust/Cargo.lock ./backend-rust/
COPY backend-rust/src ./backend-rust/src
WORKDIR /build/backend-rust
RUN cargo build --release --locked

FROM python:3.11-slim-bookworm AS runtime

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    APP_HOST=0.0.0.0 \
    APP_PORT=8000

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        ffmpeg \
        tesseract-ocr \
        tesseract-ocr-eng \
        tesseract-ocr-rus \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=rust-builder /build/backend-rust/target/release/manhwa-dubbing-backend /usr/local/bin/manhwa-dubbing-backend
COPY frontend ./frontend
COPY assets ./assets
COPY manhwa_ml ./manhwa_ml
COPY Tlok_Backend ./Tlok_Backend
COPY pyproject.toml README.md ./

RUN mkdir -p /app/output /app/data/input /app/data/texts /app/assets/music

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD python -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:8000/api/v1/health', timeout=3).read()"

CMD ["sh", "-c", "manhwa-dubbing-backend --addr ${APP_HOST}:${APP_PORT} --ml-command 'python -m manhwa_ml.cli'"]
