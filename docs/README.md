# 🎬 AI Manhwa Dubbing System for YouTube

[![Python](https://img.shields.io/badge/Python-3.9+-blue.svg)](https://www.python.org/)
[![AI](https://img.shields.io/badge/AI-Powered-purple.svg)](https://www.openai.com/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> 🚀 A fully automated **AI pipeline** that transforms manhwa chapters into professional YouTube recap videos — complete with emotional voice narration, intelligent summaries, and cinematic sound design.

---

This README.md file was generated with Perplexity Pro and Research mode, Best Model:
https://www.perplexity.ai/search/write-a-readme-and-docs-specif-wmW0601GTjCJfr6fpGftug?preview=1#0

Documentations & Specification (/docs):

- README.md [click me.](README.md)
- ARCHITECTURE.md [click me.](ARCHITECTURE.md)
- COMPONENTS-SPEC.md [click me.](COMPONENTS-SPEC.md)

---

![AI Manhwa Dubbing System - Complete Pipeline Architecture](ai_manhwa_pipeline.png)

## 🧩 Overview

The **AI Manhwa Dubbing System** automates the entire process of creating long-form manhwa recap videos for YouTube.  
It extracts text from manhwa chapters, summarizes the story using GPT-4, detects and crops panels, generates expressive TTS narration, and composes the final video — ready for upload.

**Example Reference:** [🎥 Manhwa Recap Sample](https://youtu.be/3Wjqr9wL7B0)

---

## ✨ Key Features

| Category | Feature | Description |
|-----------|----------|-------------|
| 🖼️ OCR | Automated text extraction | Extracts dialogue and text bubbles using Tesseract OCR |
| 🤖 AI Recap | GPT-4 story summarization | Builds coherent, engaging storylines |
| ✂️ Vision | Smart panel cropping | Uses OpenCV + YOLO for precise panel segmentation |
| 🎙️ Audio | Emotional text-to-speech | Natural, expressive voice narration (male/female) |
| 🎬 Video | Automated video assembly | Combines visuals, narration, and music into YouTube-ready format |
| 🎵 Sound | Dynamic background soundtrack | Adds mood-matched background music |
| 🧠 Scalability | Parallel processing | Multi-chapter batch processing supported |

---

## 🧱 System Architecture

```
Input: Manhwa Chapters (Images / PDF)
    ↓
[ OCR Module ] → Extract Text + Dialogues
    ↓
[ AI Recap Engine ] → Generate Narrative Summary
    ↓
[ Panel Detector ] → Crop & Sequence Panels
    ↓
[ TTS Engine ] → Generate Voice Narration
    ↓
[ Video Assembler ] → Merge Panels, Audio & Music
    ↓
Output: 2–3 Hour Dubbed Recap Video → YouTube Upload
```

📄 See [`/docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for a detailed architecture breakdown.

---

## ⚙️ Technology Stack

| Layer | Technology | Purpose |
|--------|-------------|----------|
| Core | Python 3.9+ | Primary language |
| OCR | Tesseract, OpenCV | Text detection & extraction |
| NLP | GPT-4 API | Story summarization |
| Vision | YOLOv8 / U-Net | Panel detection |
| TTS | ElevenLabs / Coqui | Emotional voice synthesis |
| Video | MoviePy + FFmpeg | Video assembly & encoding |
| Audio | Pydub / Librosa | Music synchronization |
| Config | YAML / .env | Configuration management |

---

## 📦 Prerequisites

```bash
# Python
Python >= 3.9

# System Dependencies
sudo apt install tesseract-ocr ffmpeg  # Linux
brew install tesseract ffmpeg          # macOS
# Windows: Install from official websites

# Required API Keys
OPENAI_API_KEY=your_openai_api_key
ELEVENLABS_API_KEY=your_elevenlabs_api_key
```

---

## 🚀 Installation

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/ai-manhwa-dubbing.git
cd ai-manhwa-dubbing
```

### 2. Setup Virtual Environment

```bash
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
```

### 3. Install Dependencies

```bash
pip install -r requirements.txt
```

### 4. Configure Environment

Create a `.env` file in the project root:

```env
OPENAI_API_KEY=your_openai_api_key_here
ELEVENLABS_API_KEY=your_elevenlabs_api_key_here
```

---

## 🧠 Usage

### Python Interface

```python
from src.pipeline.main_pipeline import ManhwaDubbingPipeline

pipeline = ManhwaDubbingPipeline(
    input_dir="data/input/manhwa_chapters",
    output_dir="data/output"
)

pipeline.run(
    chapters_range=(1, 30),
    voice_type="female",
    voice_emotion="expressive",
    target_duration=7200  # 2 hours
)
```

### CLI Example

```bash
python src/pipeline/main_pipeline.py     --input data/input/solo_leveling     --chapters 1-30     --voice female     --emotion dramatic     --duration 7200     --output data/output/recap.mp4
```

---

## 🧩 Project Structure

```
manhwa-dubbing-ai/
├── src/
│   ├── ocr/                 # Text extraction
│   ├── recap/               # GPT-4 summarization logic
│   ├── tts/                 # Voice generation & emotional tone
│   ├── video/               # Video assembly
│   ├── audio/               # Music mixing & audio sync
│   └── pipeline/            # Orchestration & CLI
├── data/
│   ├── input/               # Input manhwa images
│   ├── output/              # Final videos
│   └── temp/                # Temp files
├── config/                  # Configuration files
├── docs/                    # Project documentation
├── tests/                   # Unit tests
├── requirements.txt
└── README.md
```

---

## 🎛️ Configuration Example

`config/config.yaml`
```yaml
ocr:
  engine: tesseract
  language: eng
  preprocessing: true

recap:
  model: gpt-4
  style: engaging
  max_tokens: 8000

tts:
  provider: elevenlabs
  voice_id: Rachel
  emotion: expressive

video:
  fps: 24
  resolution: 1920x1080
  codec: h264

audio:
  background_music: true
  music_volume: 0.3
```

---

## 📊 Performance

| Stage | Time per Chapter | Description |
|--------|------------------|-------------|
| OCR | 30–60 sec | Depends on image quality |
| Recap Generation | 20–40 sec | GPT-4 summarization |
| Panel Detection | 10–20 sec | GPU-accelerated |
| TTS | 2–5 min / 1000 words | Emotional synthesis |
| Video Assembly | 3–5 min | Final rendering |

🕒 **Total:** ~30–40 minutes for 30 chapters (2–3 hour recap video)

---

## 🧱 Deployment

### Docker

```bash
docker build -t manhwa-dubbing:latest .
docker run -v $(pwd)/data:/app/data     -e OPENAI_API_KEY=$OPENAI_API_KEY     -e ELEVENLABS_API_KEY=$ELEVENLABS_API_KEY     manhwa-dubbing:latest
```

### GitHub Actions (CI/CD)

See workflow: `.github/workflows/process_video.yml`

---

## 🧪 Testing

```bash
pytest tests/
pytest --cov=src
```

---

## 🧑‍💻 Roadmap

- [ ] Multi-language OCR (Korean, Japanese, Chinese)
- [ ] Real-time streaming mode
- [ ] Web dashboard for editing
- [ ] Character-consistent TTS voice memory
- [ ] Fully local mode (offline inference)
- [ ] Voice cloning dataset builder

---

## ⚖️ License & Fair Use Notice

This software is provided under the **MIT License**.  
It is intended for **transformative and educational use only**.

Please ensure:
- Use excerpts, not full chapters
- Add commentary or analysis
- Credit original creators
- Respect YouTube copyright claims

---

## 🙌 Credits

- **Tesseract OCR** – Optical character recognition  
- **OpenAI GPT-4** – Story summarization  
- **ElevenLabs** – Text-to-speech  
- **MoviePy & FFmpeg** – Video editing  
- **OpenCV** – Panel detection  

---

## 📚 Documentation

Full technical documentation available in `/docs/`:
- [Architecture](docs/ARCHITECTURE.md)
- [API Reference](docs/API_REFERENCE.md)
- [Config Guide](docs/CONFIG_GUIDE.md)
- [Deployment Guide](docs/DEPLOYMENT.md)

---

> “Let the AI tell your favorite stories — one chapter at a time.” 🎙️  
> _Created with ❤️ by the AI Manhwa Dubbing Team_
