# Component Specification Document

## 1. OCR Module Specification

### Overview
The OCR (Optical Character Recognition) module extracts text from manhwa panel images using Tesseract OCR with custom preprocessing optimizations.

### Class: `ManhwaTextExtractor`

**Location:** `src/ocr/text_extractor.py`

#### Constructor
```python
def __init__(self, 
             language: str = 'eng',
             tesseract_config: str = '--psm 6',
             preprocessing: bool = True):
    """
    Initialize the text extractor.
    
    Args:
        language: Tesseract language code ('eng', 'jpn', 'kor', etc.)
        tesseract_config: Tesseract configuration parameters
        preprocessing: Enable image preprocessing for better accuracy
    """
```

#### Methods

##### `extract_from_image(image_path: str) -> List[TextRegion]`
```python
def extract_from_image(self, image_path: str) -> List[TextRegion]:
    """
    Extract all text regions from a manhwa page image.
    
    Args:
        image_path: Path to input image file
        
    Returns:
        List of TextRegion objects containing extracted text and metadata
        
    Raises:
        FileNotFoundError: If image file doesn't exist
        OCRException: If OCR processing fails
    """
```

##### `preprocess_image(image: np.ndarray) -> np.ndarray`
```python
def preprocess_image(self, image: np.ndarray) -> np.ndarray:
    """
    Apply preprocessing to improve OCR accuracy.
    
    Steps:
    1. Convert to grayscale
    2. Apply Gaussian blur for noise reduction
    3. Adaptive thresholding
    4. Morphological operations to enhance text
    
    Args:
        image: Input image as numpy array
        
    Returns:
        Preprocessed image
    """
```

##### `detect_text_regions(image: np.ndarray) -> List[BoundingBox]`
```python
def detect_text_regions(self, image: np.ndarray) -> List[BoundingBox]:
    """
    Detect speech bubbles and text boxes in image.
    
    Uses contour detection and shape analysis to identify text regions.
    
    Args:
        image: Preprocessed image
        
    Returns:
        List of bounding boxes for detected text regions
    """
```

#### Data Models

```python
@dataclass
class TextRegion:
    text: str                    # Extracted text content
    bbox: Tuple[int, int, int, int]  # Bounding box (x, y, w, h)
    confidence: float            # OCR confidence score (0-1)
    region_type: str            # 'speech_bubble', 'narration', 'sfx'
    page_id: str                # Source page identifier
    
@dataclass
class BoundingBox:
    x: int
    y: int
    width: int
    height: int
```

#### Configuration

**config/ocr_config.yaml:**
```yaml
ocr:
  tesseract:
    path: /usr/bin/tesseract
    language: eng
    config: --psm 6 --oem 3
    
  preprocessing:
    enabled: true
    gaussian_blur_kernel: [5, 5]
    adaptive_threshold_block_size: 11
    adaptive_threshold_c: 2
    
  text_detection:
    min_region_area: 100
    max_region_area: 50000
    aspect_ratio_range: [0.1, 10.0]
```

---

## 2. AI Recap Generation Specification

### Overview
Generates coherent narrative summaries from extracted manhwa text using GPT-4 API with custom prompt engineering.

### Class: `ManhwaRecapGenerator`

**Location:** `src/recap/summarizer.py`

#### Constructor
```python
def __init__(self,
             api_key: str,
             model: str = "gpt-4-turbo",
             max_tokens: int = 8000,
             temperature: float = 0.7):
    """
    Initialize the recap generator.
    
    Args:
        api_key: OpenAI API key
        model: Model name (gpt-4, gpt-4-turbo, etc.)
        max_tokens: Maximum tokens for generation
        temperature: Sampling temperature (0-1)
    """
```

#### Methods

##### `create_recap(texts: List[str], style: str = 'engaging') -> str`
```python
def create_recap(self, 
                 texts: List[str], 
                 style: str = 'engaging',
                 max_length: int = 5000) -> str:
    """
    Generate a comprehensive recap from extracted texts.
    
    Args:
        texts: List of text strings from manhwa chapters
        style: Narration style ('engaging', 'dramatic', 'neutral')
        max_length: Target word count for recap
        
    Returns:
        Generated recap text
        
    Raises:
        APIException: If API call fails
        TokenLimitException: If input exceeds token limit
    """
```

##### `chunk_text(text: str, chunk_size: int = 4000) -> List[str]`
```python
def chunk_text(self, text: str, chunk_size: int = 4000) -> List[str]:
    """
    Split large text into chunks for processing.
    
    Implements smart chunking that preserves sentence boundaries.
    
    Args:
        text: Input text to chunk
        chunk_size: Maximum tokens per chunk
        
    Returns:
        List of text chunks
    """
```

##### `summarize_chunk(chunk: str) -> str`
```python
def summarize_chunk(self, chunk: str) -> str:
    """
    Summarize a single text chunk.
    
    Args:
        chunk: Text chunk to summarize
        
    Returns:
        Summary of chunk
    """
```

##### `combine_summaries(summaries: List[str]) -> str`
```python
def combine_summaries(self, summaries: List[str]) -> str:
    """
    Combine multiple chunk summaries into coherent narrative.
    
    Args:
        summaries: List of chunk summaries
        
    Returns:
        Final combined recap
    """
```

#### Prompt Templates

**src/recap/prompt_templates.py:**
```python
RECAP_SYSTEM_PROMPT = """
You are an expert storyteller specializing in creating engaging YouTube video recaps 
of manhwa series. Your recaps are:
- Compelling and dramatic
- Chronologically coherent
- Suitable for voice-over narration
- Engaging for YouTube audiences
"""

RECAP_USER_PROMPT = """
Create a {style} recap of the following manhwa content in approximately {word_count} words.

Focus on:
1. Main plot developments
2. Character growth and relationships
3. Dramatic moments and conflicts
4. Cliffhangers and hooks

Content to recap:
{content}

Generate a narrative suitable for YouTube voice-over that will keep viewers engaged.
"""

CHUNK_SUMMARY_PROMPT = """
Summarize the following section of a manhwa story, preserving key plot points 
and character developments:

{chunk}

Provide a concise summary that captures the essence of this section.
"""
```

#### Configuration

**config/recap_config.yaml:**
```yaml
recap:
  openai:
    model: gpt-4-turbo
    max_tokens: 8000
    temperature: 0.7
    top_p: 0.9
    frequency_penalty: 0.0
    presence_penalty: 0.0
    
  chunking:
    chunk_size: 4000
    overlap: 200
    preserve_sentences: true
    
  styles:
    engaging:
      temperature: 0.7
      emphasis: "dramatic moments and character emotions"
    dramatic:
      temperature: 0.8
      emphasis: "tension, conflict, and high-stakes situations"
    neutral:
      temperature: 0.5
      emphasis: "factual plot summary"
```

---

## 3. Panel Detection & Cropping Specification

### Overview
Automatically detects manhwa panel boundaries and extracts individual panels in reading order.

### Class: `PanelDetector`

**Location:** `src/video/panel_detector.py`

#### Constructor
```python
def __init__(self,
             method: str = 'opencv',
             model_path: str = None,
             min_panel_area: int = 10000):
    """
    Initialize panel detector.
    
    Args:
        method: Detection method ('opencv', 'yolo', 'hybrid')
        model_path: Path to trained model (for YOLO method)
        min_panel_area: Minimum area for valid panels (pixels²)
    """
```

#### Methods

##### `detect_panels(image_path: str) -> List[Panel]`
```python
def detect_panels(self, image_path: str) -> List[Panel]:
    """
    Detect all panels in a manhwa page.
    
    Args:
        image_path: Path to input manhwa page image
        
    Returns:
        List of Panel objects with bounding boxes
    """
```

##### `order_panels(panels: List[Panel]) -> List[Panel]`
```python
def order_panels(self, panels: List[Panel]) -> List[Panel]:
    """
    Order panels in reading sequence (top-to-bottom, left-to-right).
    
    Args:
        panels: Unordered list of detected panels
        
    Returns:
        Ordered list of panels
    """
```

##### `crop_panel(image: np.ndarray, bbox: BoundingBox) -> np.ndarray`
```python
def crop_panel(self, 
               image: np.ndarray, 
               bbox: BoundingBox,
               margin: int = 10) -> np.ndarray:
    """
    Extract panel from image with optional margin.
    
    Args:
        image: Source image
        bbox: Panel bounding box
        margin: Extra pixels around panel
        
    Returns:
        Cropped panel image
    """
```

#### Data Models

```python
@dataclass
class Panel:
    bbox: BoundingBox
    sequence_number: int
    confidence: float
    image_data: Optional[np.ndarray] = None
```

#### Detection Algorithms

**OpenCV Method:**
```python
def detect_opencv(self, image: np.ndarray) -> List[Panel]:
    """
    Contour-based panel detection.
    
    Steps:
    1. Convert to grayscale
    2. Apply binary threshold
    3. Find contours
    4. Filter by size and shape
    5. Extract bounding rectangles
    """
```

**YOLO Method:**
```python
def detect_yolo(self, image_path: str) -> List[Panel]:
    """
    Deep learning-based detection using YOLOv8.
    
    Requires pre-trained model on manhwa panels.
    """
```

#### Configuration

**config/panel_detection_config.yaml:**
```yaml
panel_detection:
  method: opencv  # opencv, yolo, or hybrid
  
  opencv:
    threshold_value: 240
    threshold_type: binary_inv
    min_panel_area: 10000
    max_panel_area: 500000
    aspect_ratio_tolerance: 0.1
    
  yolo:
    model_path: models/panel_detection/yolov8_panels.pt
    confidence_threshold: 0.5
    iou_threshold: 0.4
    
  ordering:
    row_threshold: 50  # pixels
    direction: lr-tb   # left-right, top-bottom
    
  cropping:
    margin: 10
    resize_target: [1920, 1080]
    maintain_aspect: true
```

---

## 4. Text-to-Speech Specification

### Overview
Generates emotional, human-like voice narration using ElevenLabs API with emotion detection.

### Class: `EmotionalTTS`

**Location:** `src/tts/voice_generator.py`

#### Constructor
```python
def __init__(self,
             api_key: str,
             voice_name: str = "Rachel",
             model: str = "eleven_multilingual_v2"):
    """
    Initialize TTS engine.
    
    Args:
        api_key: ElevenLabs API key
        voice_name: Voice identifier
        model: ElevenLabs model version
    """
```

#### Methods

##### `generate(text: str, emotion: str = 'neutral') -> bytes`
```python
def generate(self, 
             text: str, 
             emotion: str = 'neutral',
             output_format: str = 'mp3') -> bytes:
    """
    Generate voice audio from text.
    
    Args:
        text: Text to convert to speech
        emotion: Emotion tag ('neutral', 'dramatic', 'sad', 'happy', 'angry')
        output_format: Audio format ('mp3' or 'wav')
        
    Returns:
        Audio data as bytes
    """
```

##### `generate_with_emotions(emotional_segments: List[EmotionalSegment]) -> bytes`
```python
def generate_with_emotions(self, 
                           emotional_segments: List[EmotionalSegment]) -> bytes:
    """
    Generate audio with emotion changes throughout.
    
    Args:
        emotional_segments: List of text segments with emotion labels
        
    Returns:
        Combined audio with emotional variation
    """
```

##### `analyze_emotions(text: str) -> List[EmotionalSegment]`
```python
def analyze_emotions(self, text: str) -> List[EmotionalSegment]:
    """
    Detect emotions in text using NLP model.
    
    Args:
        text: Input text to analyze
        
    Returns:
        List of segments with detected emotions
    """
```

#### Data Models

```python
@dataclass
class EmotionalSegment:
    text: str
    emotion: str
    confidence: float
    start_time: Optional[float] = None
    end_time: Optional[float] = None

@dataclass
class VoiceSettings:
    stability: float = 0.75      # 0-1
    similarity_boost: float = 0.85
    style: float = 0.5
    use_speaker_boost: bool = True
```

#### Available Voices

```python
VOICES = {
    "female": {
        "Rachel": "voice_id_rachel",
        "Bella": "voice_id_bella",
        "Elli": "voice_id_elli"
    },
    "male": {
        "Adam": "voice_id_adam",
        "Antoni": "voice_id_antoni",
        "Josh": "voice_id_josh"
    }
}

EMOTION_SETTINGS = {
    "neutral": VoiceSettings(stability=0.75, style=0.5),
    "dramatic": VoiceSettings(stability=0.5, style=0.8),
    "sad": VoiceSettings(stability=0.7, style=0.3),
    "happy": VoiceSettings(stability=0.6, style=0.7),
    "angry": VoiceSettings(stability=0.4, style=0.9)
}
```

#### Configuration

**config/tts_config.yaml:**
```yaml
tts:
  provider: elevenlabs
  
  elevenlabs:
    model: eleven_multilingual_v2
    output_format: mp3_44100_128
    
  voices:
    default_female: Rachel
    default_male: Adam
    
  emotion_detection:
    enabled: true
    model: j-hartmann/emotion-english-distilroberta-base
    segment_length: 500
    
  audio_processing:
    normalize: true
    remove_silence: true
    silence_threshold: -50  # dB
```

---

## 5. Video Assembly Specification

### Overview
Combines panels, narration, and background music into final video with effects and transitions.

### Class: `VideoAssembler`

**Location:** `src/video/video_editor.py`

#### Constructor
```python
def __init__(self,
             fps: int = 24,
             resolution: Tuple[int, int] = (1920, 1080),
             codec: str = 'libx264'):
    """
    Initialize video assembler.
    
    Args:
        fps: Frames per second
        resolution: Video resolution (width, height)
        codec: Video codec for encoding
    """
```

#### Methods

##### `create_video(panels, audio, background_music=None) -> VideoFileClip`
```python
def create_video(self,
                 panels: List[str],
                 audio: str,
                 background_music: Optional[str] = None,
                 transitions: str = 'fade') -> VideoFileClip:
    """
    Create final video from components.
    
    Args:
        panels: List of panel image paths in order
        audio: Path to narration audio file
        background_music: Optional background music path
        transitions: Transition type ('fade', 'none', 'slide')
        
    Returns:
        Assembled video clip
    """
```

##### `apply_ken_burns(clip: ImageClip, duration: float) -> ImageClip`
```python
def apply_ken_burns(self, 
                    clip: ImageClip, 
                    duration: float,
                    zoom_factor: float = 0.1) -> ImageClip:
    """
    Apply Ken Burns effect (zoom and pan).
    
    Args:
        clip: Input image clip
        duration: Effect duration
        zoom_factor: Amount of zoom (0-1)
        
    Returns:
        Clip with Ken Burns effect
    """
```

##### `sync_audio_video(video_clip, audio_clip) -> VideoFileClip`
```python
def sync_audio_video(self, 
                     video_clip: VideoFileClip, 
                     audio_clip: AudioFileClip) -> VideoFileClip:
    """
    Synchronize video length with audio duration.
    
    Args:
        video_clip: Video clip to adjust
        audio_clip: Reference audio clip
        
    Returns:
        Synchronized video clip
    """
```

##### `mix_audio(narration, background_music, volumes) -> AudioClip`
```python
def mix_audio(self,
              narration: str,
              background_music: str,
              volumes: Tuple[float, float] = (1.0, 0.3)) -> AudioClip:
    """
    Mix narration with background music.
    
    Args:
        narration: Narration audio path
        background_music: Background music path
        volumes: (narration_volume, music_volume)
        
    Returns:
        Mixed audio clip
    """
```

##### `export(video_clip, output_path, quality='high')`
```python
def export(self,
           video_clip: VideoFileClip,
           output_path: str,
           quality: str = 'high'):
    """
    Export final video to file.
    
    Args:
        video_clip: Video to export
        output_path: Output file path
        quality: Quality preset ('high', 'medium', 'low')
    """
```

#### Effects

```python
class VideoEffects:
    @staticmethod
    def fade_transition(duration: float = 0.5):
        """Crossfade between clips."""
        
    @staticmethod
    def zoom_effect(start_zoom: float = 1.0, end_zoom: float = 1.1):
        """Gradual zoom effect."""
        
    @staticmethod
    def pan_effect(start_pos: Tuple, end_pos: Tuple):
        """Pan across image."""
        
    @staticmethod
    def color_grade(brightness: float = 1.0, contrast: float = 1.0):
        """Adjust color properties."""
```

#### Configuration

**config/video_config.yaml:**
```yaml
video:
  output:
    fps: 24
    resolution: [1920, 1080]
    codec: libx264
    format: mp4
    
  encoding:
    quality: high
    bitrate: 8000k
    preset: medium
    crf: 23
    
  audio:
    codec: aac
    bitrate: 192k
    sample_rate: 44100
    
  effects:
    ken_burns:
      enabled: true
      zoom_factor: 0.05
      duration_per_panel: 3.0
      
    transitions:
      type: fade
      duration: 0.5
      
  audio_mix:
    narration_volume: 1.0
    music_volume: 0.3
    normalize: true
```

---

## 6. Main Pipeline Orchestration

### Class: `ManhwaDubbingPipeline`

**Location:** `src/pipeline/main_pipeline.py`

#### Constructor
```python
def __init__(self, config_path: str = "config/config.yaml"):
    """
    Initialize the complete pipeline.
    
    Args:
        config_path: Path to configuration file
    """
```

#### Methods

##### `run(input_dir, chapters_range, output_path, **kwargs)`
```python
def run(self,
        input_dir: str,
        chapters_range: Tuple[int, int],
        output_path: str,
        voice_type: str = "female",
        voice_emotion: str = "expressive",
        background_music: Optional[str] = None,
        target_duration: Optional[int] = None) -> str:
    """
    Execute complete pipeline.
    
    Args:
        input_dir: Directory containing manhwa chapter images
        chapters_range: Tuple of (start_chapter, end_chapter)
        output_path: Output video file path
        voice_type: 'male' or 'female'
        voice_emotion: Emotion style
        background_music: Background music path (optional)
        target_duration: Target video duration in seconds
        
    Returns:
        Path to generated video file
    """
```

---

This specification provides the detailed component interfaces and data models for implementing the AI Manhwa Dubbing System.
