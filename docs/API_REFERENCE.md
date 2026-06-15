# ⚙️ API Reference — AI Manhwa Dubbing System

## Base URL
```
http://localhost:8000/api/v1
```

## Endpoints

### GET /
Serves the browser console UI.

### GET /metrics
Returns runtime status for jobs, ML/AI stages, and local tool capabilities.

### GET /music/library
Returns music catalog metadata and scanned audio files.

### POST /music/register
Registers a music file path in `assets/music/library.json`.

### POST /dubbing/start
Starts a dubbing process.
```json
{
  "project_name": "solo_leveling",
  "language": "en",
  "voice": "female_expressive",
  "chapters": [1, 2, 3],
  "images_dir": "Tlok_Backend/<images-dir>",
  "audio_dir": "Tlok_Backend/zvyki",
  "texts_dir": "data/texts",
  "background_music": "data/music/theme.mp3",
  "music_dir": "assets/music",
  "music_mood": "dramatic",
  "ml_command": "poetry run manhwa-ml",
  "output_dir": "output",
  "pairing": "sequential",
  "recap_style": "engaging",
  "synthesize_voice": false,
  "render": false
}
```
**Response**
```json
{
  "job_id": "job-1781388039172",
  "status": "pipeline_ready",
  "pairs_count": 5,
  "job_dir": "output/jobs/job-1781388039172",
  "output_video": null,
  "message": "minimal pipeline artifacts created; render disabled",
  "artifacts": []
}
```

### GET /status/{task_id}
Retrieves job status.
```json
{
  "task_id": "b3d91e12",
  "status": "processing",
  "progress": 72,
  "current_stage": "tts_generation"
}
```

### GET /result/{task_id}
Returns final video URLs and logs.
```json
{
  "video_url": "https://cdn.manhwa.ai/output/solo_leveling.mp4",
  "logs": "https://cdn.manhwa.ai/logs/b3d91e12.log"
}
```

### GET /health
Health check endpoint.
```json
{"status": "ok"}
```
