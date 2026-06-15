use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PairingMode {
    Sequential,
    Stem,
}

impl PairingMode {
    fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "stem" | "name" | "matching" => Self::Stem,
            _ => Self::Sequential,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::Stem => "stem",
        }
    }
}

#[derive(Clone, Debug)]
struct PipelineRequest {
    images_dir: PathBuf,
    audio_dir: PathBuf,
    output_dir: PathBuf,
    texts_dir: Option<PathBuf>,
    background_music: Option<PathBuf>,
    music_dir: PathBuf,
    music_mood: String,
    ml_command: Option<String>,
    pairing: PairingMode,
    render: bool,
    synthesize_voice: bool,
    language: String,
    recap_style: String,
    voice: String,
    width: u32,
    height: u32,
    fps: u32,
}

#[derive(Clone, Debug)]
struct MediaPair {
    index: usize,
    image: PathBuf,
    audio: PathBuf,
}

#[derive(Clone, Debug)]
struct OcrItem {
    index: usize,
    image: PathBuf,
    text: String,
    source: String,
}

#[derive(Clone, Debug)]
struct StageArtifact {
    stage: String,
    status: String,
    path: Option<PathBuf>,
    message: String,
}

#[derive(Clone, Debug)]
struct JobResult {
    job_id: String,
    job_dir: PathBuf,
    status: String,
    pairs_count: usize,
    output_video: Option<PathBuf>,
    message: String,
    artifacts: Vec<StageArtifact>,
}

#[derive(Clone)]
struct ServerState {
    default_request: PipelineRequest,
    jobs: Arc<Mutex<HashMap<String, PathBuf>>>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if has_flag(&args, "--help") || has_flag(&args, "-h") {
        print_help();
        return Ok(());
    }

    let mut request = default_request();
    apply_args_to_request(&args, &mut request);

    if has_flag(&args, "--once") {
        let result = run_pipeline(&request)?;
        println!("{}", job_result_json(&result));
        return Ok(());
    }

    let addr = arg_value(&args, "--addr").unwrap_or_else(|| "127.0.0.1:8000".to_string());
    serve(addr, request)
}

fn default_request() -> PipelineRequest {
    let images_dir =
        existing_or_parent(PathBuf::from("Tlok_Backend").join("\u{0444}\u{043e}\u{0442}\u{043e}"));
    let audio_dir = existing_or_parent(PathBuf::from("Tlok_Backend").join("zvyki"));
    let output_dir = if Path::new("backend-rust").is_dir() {
        PathBuf::from("output")
    } else {
        PathBuf::from("..").join("output")
    };
    let music_dir = existing_or_parent(PathBuf::from("assets").join("music"));

    PipelineRequest {
        images_dir,
        audio_dir,
        output_dir,
        texts_dir: None,
        background_music: None,
        music_dir,
        music_mood: "neutral".to_string(),
        ml_command: None,
        pairing: PairingMode::Sequential,
        render: false,
        synthesize_voice: false,
        language: "eng".to_string(),
        recap_style: "engaging".to_string(),
        voice: "default".to_string(),
        width: 1080,
        height: 1920,
        fps: 30,
    }
}

fn existing_or_parent(path: PathBuf) -> PathBuf {
    if path.exists() {
        path
    } else {
        PathBuf::from("..").join(path)
    }
}

fn apply_args_to_request(args: &[String], request: &mut PipelineRequest) {
    if let Some(value) = arg_value(args, "--images") {
        request.images_dir = PathBuf::from(value);
    }
    if let Some(value) = arg_value(args, "--audio") {
        request.audio_dir = PathBuf::from(value);
    }
    if let Some(value) = arg_value(args, "--output") {
        request.output_dir = PathBuf::from(value);
    }
    if let Some(value) = arg_value(args, "--texts") {
        request.texts_dir = Some(PathBuf::from(value));
    }
    if let Some(value) = arg_value(args, "--background-music") {
        request.background_music = Some(PathBuf::from(value));
    }
    if let Some(value) = arg_value(args, "--music-dir") {
        request.music_dir = PathBuf::from(value);
    }
    if let Some(value) = arg_value(args, "--music-mood") {
        request.music_mood = value;
    }
    if let Some(value) = arg_value(args, "--ml-command") {
        request.ml_command = Some(value);
    }
    if let Some(value) = arg_value(args, "--pairing") {
        request.pairing = PairingMode::parse(&value);
    }
    if let Some(value) = arg_value(args, "--language") {
        request.language = value;
    }
    if let Some(value) = arg_value(args, "--style") {
        request.recap_style = value;
    }
    if let Some(value) = arg_value(args, "--voice") {
        request.voice = value;
    }
    if let Some(value) = arg_value(args, "--width") {
        request.width = value.parse().unwrap_or(request.width);
    }
    if let Some(value) = arg_value(args, "--height") {
        request.height = value.parse().unwrap_or(request.height);
    }
    if let Some(value) = arg_value(args, "--fps") {
        request.fps = value.parse().unwrap_or(request.fps);
    }
    if has_flag(args, "--render") {
        request.render = true;
    }
    if has_flag(args, "--synthesize-voice") {
        request.synthesize_voice = true;
    }
}

fn run_pipeline(request: &PipelineRequest) -> Result<JobResult, String> {
    if !request.images_dir.is_dir() {
        return Err(format!(
            "images directory not found: {}",
            request.images_dir.display()
        ));
    }
    if !request.audio_dir.is_dir() {
        return Err(format!(
            "audio directory not found: {}",
            request.audio_dir.display()
        ));
    }

    let pairs = discover_pairs(&request.images_dir, &request.audio_dir, request.pairing)?;
    if pairs.is_empty() {
        return Err("no media pairs were found".to_string());
    }

    let job_id = new_job_id();
    let job_dir = request.output_dir.join("jobs").join(&job_id);
    fs::create_dir_all(&job_dir).map_err(|err| format!("cannot create job dir: {err}"))?;

    write_manifest(&job_dir, request, &pairs)?;
    let mut artifacts = vec![StageArtifact {
        stage: "input".to_string(),
        status: "completed".to_string(),
        path: Some(job_dir.join("manifest.json")),
        message: format!("{} media pairs discovered", pairs.len()),
    }];

    if request.ml_command.is_some() {
        artifacts.push(run_ml_worker_stage(&job_dir, request)?);
        artifacts.extend(collect_ml_artifacts(&job_dir));
    } else {
        let (ocr_items, ocr_artifact) = run_ocr_stage(&job_dir, request, &pairs)?;
        artifacts.push(ocr_artifact);

        let (recap_text, recap_artifact) = run_recap_stage(&job_dir, request, &ocr_items)?;
        artifacts.push(recap_artifact);

        let panels_artifact = run_panel_stage(&job_dir, &pairs)?;
        artifacts.push(panels_artifact);

        let tts_artifact = run_tts_stage(&job_dir, request, &recap_text)?;
        artifacts.push(tts_artifact);

        let (selected_music, music_artifact) = run_music_stage(&job_dir, request)?;
        artifacts.push(music_artifact);

        let audio_artifact = run_audio_mix_stage(&job_dir, &pairs, selected_music.as_deref())?;
        artifacts.push(audio_artifact);
    }

    let mut result = JobResult {
        job_id,
        job_dir,
        status: "pipeline_ready".to_string(),
        pairs_count: pairs.len(),
        output_video: None,
        message: "minimal pipeline artifacts created; render disabled".to_string(),
        artifacts,
    };

    if request.render {
        match render_video(request, &pairs, &result.job_dir) {
            Ok(video_path) => {
                result.status = "completed".to_string();
                result.output_video = Some(video_path);
                result.message = "video rendered".to_string();
                result.artifacts.push(StageArtifact {
                    stage: "video".to_string(),
                    status: "completed".to_string(),
                    path: result.output_video.clone(),
                    message: "final mp4 rendered with ffmpeg".to_string(),
                });
            }
            Err(err) => {
                result.status = "failed".to_string();
                result.message = err.clone();
                result.artifacts.push(StageArtifact {
                    stage: "video".to_string(),
                    status: "failed".to_string(),
                    path: None,
                    message: err,
                });
            }
        }
    } else {
        result.artifacts.push(StageArtifact {
            stage: "video".to_string(),
            status: "ready".to_string(),
            path: None,
            message: "render disabled; pass --render or render=true after installing ffmpeg"
                .to_string(),
        });
    }

    write_status(&result)?;
    Ok(result)
}

fn discover_pairs(
    images_dir: &Path,
    audio_dir: &Path,
    mode: PairingMode,
) -> Result<Vec<MediaPair>, String> {
    let images = collect_media_files(images_dir, &["png", "jpg", "jpeg", "webp", "bmp"])?;
    let audios = collect_media_files(audio_dir, &["mp3", "wav", "m4a", "aac", "flac", "ogg"])?;

    match mode {
        PairingMode::Sequential => {
            let total = images.len().min(audios.len());
            Ok((0..total)
                .map(|idx| MediaPair {
                    index: idx + 1,
                    image: images[idx].clone(),
                    audio: audios[idx].clone(),
                })
                .collect())
        }
        PairingMode::Stem => {
            let image_map = files_by_stem(images);
            let audio_map = files_by_stem(audios);
            let mut pairs = Vec::new();
            for (stem, image) in image_map {
                if let Some(audio) = audio_map.get(&stem) {
                    pairs.push(MediaPair {
                        index: pairs.len() + 1,
                        image,
                        audio: audio.clone(),
                    });
                }
            }
            Ok(pairs)
        }
    }
}

fn collect_media_files(dir: &Path, allowed_exts: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|err| format!("cannot read {}: {err}", dir.display()))? {
        let entry = entry.map_err(|err| format!("cannot read directory entry: {err}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if allowed_exts
            .iter()
            .any(|allowed| ext.eq_ignore_ascii_case(allowed))
        {
            files.push(path);
        }
    }
    files.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
    Ok(files)
}

fn files_by_stem(files: Vec<PathBuf>) -> BTreeMap<String, PathBuf> {
    let mut map = BTreeMap::new();
    for file in files {
        if let Some(stem) = file.file_stem().and_then(|value| value.to_str()) {
            map.insert(stem.to_string(), file);
        }
    }
    map
}

fn run_ml_worker_stage(job_dir: &Path, request: &PipelineRequest) -> Result<StageArtifact, String> {
    let command = request
        .ml_command
        .as_ref()
        .ok_or_else(|| "missing ML command".to_string())?;
    let manifest_path = job_dir
        .join("manifest.json")
        .canonicalize()
        .unwrap_or_else(|_| job_dir.join("manifest.json"));
    let manifest_path = strip_windows_verbatim(manifest_path);
    let absolute_job_dir = job_dir
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(job_dir));
    let absolute_job_dir = strip_windows_verbatim(absolute_job_dir);
    let Some((program, args)) = split_command(command) else {
        return Err("ML command is empty".to_string());
    };
    let mut worker_args = args;
    worker_args.push("run".to_string());
    worker_args.push("--manifest".to_string());
    worker_args.push(manifest_path.to_string_lossy().to_string());
    worker_args.push("--job-dir".to_string());
    worker_args.push(absolute_job_dir.to_string_lossy().to_string());
    if request.synthesize_voice {
        worker_args.push("--synthesize-voice".to_string());
    }

    let current_dir = project_root_from_job_dir(job_dir);
    let output = Command::new(program)
        .current_dir(&current_dir)
        .args(worker_args)
        .output()
        .map_err(|err| format!("failed to start ML worker: {err}"))?;

    fs::write(job_dir.join("ml_worker.stdout.log"), &output.stdout)
        .map_err(|err| format!("cannot write ML worker stdout: {err}"))?;
    fs::write(job_dir.join("ml_worker.stderr.log"), &output.stderr)
        .map_err(|err| format!("cannot write ML worker stderr: {err}"))?;

    if output.status.success() {
        Ok(StageArtifact {
            stage: "ml_worker".to_string(),
            status: "completed".to_string(),
            path: Some(job_dir.join("ml_worker_status.json")),
            message: "external Python ML worker completed".to_string(),
        })
    } else {
        Err(format!(
            "ML worker failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn split_command(command: &str) -> Option<(String, Vec<String>)> {
    let parts = command
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let (program, args) = parts.split_first()?;
    Some((program.clone(), args.to_vec()))
}

fn project_root_from_job_dir(job_dir: &Path) -> PathBuf {
    let absolute = job_dir
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(job_dir));
    let absolute = strip_windows_verbatim(absolute);
    if absolute
        .parent()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        == Some("jobs")
    {
        absolute
            .parent()
            .and_then(|jobs| jobs.parent())
            .and_then(|output| output.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    } else {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

fn strip_windows_verbatim(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(stripped) = text.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path
    }
}

fn collect_ml_artifacts(job_dir: &Path) -> Vec<StageArtifact> {
    [
        ("ocr", "completed", "ocr.json"),
        ("recap", "completed", "recap.txt"),
        ("panel_detection", "completed", "panels.json"),
        (
            "tts",
            if job_dir.join("narration.wav").is_file() {
                "completed"
            } else {
                "ready"
            },
            if job_dir.join("narration.wav").is_file() {
                "narration.wav"
            } else {
                "tts_request.json"
            },
        ),
        ("audio_mix", "ready", "audio_mix.json"),
        ("music", "ready", "music_selection.json"),
    ]
    .into_iter()
    .filter_map(|(stage, status, file)| {
        let path = job_dir.join(file);
        if path.exists() {
            Some(StageArtifact {
                stage: stage.to_string(),
                status: status.to_string(),
                path: Some(path),
                message: "generated by external ML worker".to_string(),
            })
        } else {
            None
        }
    })
    .collect()
}

fn run_ocr_stage(
    job_dir: &Path,
    request: &PipelineRequest,
    pairs: &[MediaPair],
) -> Result<(Vec<OcrItem>, StageArtifact), String> {
    let mut items = Vec::new();
    let mut used_tesseract = false;
    let mut used_sidecars = false;

    for pair in pairs {
        let (text, source) = read_sidecar_text(request, &pair.image)
            .map(|text| {
                used_sidecars = true;
                (text, "sidecar".to_string())
            })
            .or_else(|| {
                run_tesseract(&pair.image, &request.language).map(|text| {
                    used_tesseract = true;
                    (text, "tesseract".to_string())
                })
            })
            .unwrap_or_else(|| {
                (
                    String::new(),
                    "unavailable: add .txt sidecars or install tesseract".to_string(),
                )
            });

        items.push(OcrItem {
            index: pair.index,
            image: pair.image.clone(),
            text,
            source,
        });
    }

    let path = job_dir.join("ocr.json");
    write_ocr_json(&path, &items)?;

    let status = if used_tesseract || used_sidecars {
        "completed"
    } else {
        "fallback"
    };
    let message = if used_tesseract {
        "OCR completed with tesseract"
    } else if used_sidecars {
        "OCR loaded from text sidecar files"
    } else {
        "OCR fallback created empty records; tesseract/text sidecars not available"
    };

    Ok((
        items,
        StageArtifact {
            stage: "ocr".to_string(),
            status: status.to_string(),
            path: Some(path),
            message: message.to_string(),
        },
    ))
}

fn read_sidecar_text(request: &PipelineRequest, image: &Path) -> Option<String> {
    let stem = image.file_stem()?.to_str()?;
    let candidates = [
        request
            .texts_dir
            .as_ref()
            .map(|dir| dir.join(format!("{stem}.txt"))),
        Some(image.with_extension("txt")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.is_file() {
            if let Ok(text) = fs::read_to_string(candidate) {
                if !text.trim().is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn run_tesseract(image: &Path, language: &str) -> Option<String> {
    let output = Command::new("tesseract")
        .arg(image)
        .arg("stdout")
        .arg("-l")
        .arg(language)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn write_ocr_json(path: &Path, items: &[OcrItem]) -> Result<(), String> {
    let mut json = String::from("{\n  \"items\": [\n");
    for (idx, item) in items.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"index\": {},\n", item.index));
        json.push_str(&format!(
            "      \"image\": \"{}\",\n",
            json_escape_path(&item.image)
        ));
        json.push_str(&format!(
            "      \"source\": \"{}\",\n",
            json_escape(&item.source)
        ));
        json.push_str(&format!(
            "      \"text\": \"{}\"\n",
            json_escape(&item.text)
        ));
        json.push_str("    }");
        if idx + 1 != items.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ]\n}\n");
    fs::write(path, json).map_err(|err| format!("cannot write OCR JSON: {err}"))
}

fn run_recap_stage(
    job_dir: &Path,
    request: &PipelineRequest,
    items: &[OcrItem],
) -> Result<(String, StageArtifact), String> {
    let mut source = items
        .iter()
        .filter_map(|item| {
            let text = item.text.trim();
            if text.is_empty() {
                None
            } else {
                Some(format!(
                    "Page {}: {}",
                    item.index,
                    collapse_whitespace(text)
                ))
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if source.is_empty() {
        source = "No OCR text was available. Add .txt sidecar files or install Tesseract for real text extraction.".to_string();
    }

    let recap = local_recap(&source, &request.recap_style);
    let path = job_dir.join("recap.txt");
    fs::write(&path, &recap).map_err(|err| format!("cannot write recap: {err}"))?;

    Ok((
        recap,
        StageArtifact {
            stage: "recap".to_string(),
            status: "completed".to_string(),
            path: Some(path),
            message:
                "local extractive recap generated; OpenAI provider can replace this stage later"
                    .to_string(),
        },
    ))
}

fn local_recap(source: &str, style: &str) -> String {
    let mut selected = split_sentences(source)
        .into_iter()
        .take(12)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        selected.push(source.to_string());
    }

    format!(
        "Style: {style}\n\n{}\n\nProduction note: this is a local MVP recap generated from available OCR text.",
        selected.join(" ")
    )
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?' | '\n') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            current.clear();
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_string());
    }
    sentences
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn run_panel_stage(job_dir: &Path, pairs: &[MediaPair]) -> Result<StageArtifact, String> {
    let path = job_dir.join("panels.json");
    let mut json = String::from("{\n  \"method\": \"full-page-fallback\",\n  \"panels\": [\n");

    for (idx, pair) in pairs.iter().enumerate() {
        let (width, height) = image_dimensions(&pair.image).unwrap_or((0, 0));
        json.push_str("    {\n");
        json.push_str(&format!("      \"index\": {},\n", pair.index));
        json.push_str(&format!(
            "      \"image\": \"{}\",\n",
            json_escape_path(&pair.image)
        ));
        json.push_str("      \"bbox\": {\n");
        json.push_str("        \"x\": 0,\n");
        json.push_str("        \"y\": 0,\n");
        json.push_str(&format!("        \"width\": {},\n", width));
        json.push_str(&format!("        \"height\": {}\n", height));
        json.push_str("      },\n");
        json.push_str("      \"confidence\": 1.0,\n");
        json.push_str("      \"note\": \"full page used as one panel; OpenCV/YOLO can replace this stage later\"\n");
        json.push_str("    }");
        if idx + 1 != pairs.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ]\n}\n");
    fs::write(&path, json).map_err(|err| format!("cannot write panels JSON: {err}"))?;

    Ok(StageArtifact {
        stage: "panel_detection".to_string(),
        status: "fallback".to_string(),
        path: Some(path),
        message: "panel metadata generated using full-page fallback".to_string(),
    })
}

fn image_dimensions(path: &Path) -> Option<(u32, u32)> {
    let bytes = fs::read(path).ok()?;
    png_dimensions(&bytes).or_else(|| jpeg_dimensions(&bytes))
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let mut pos = 2;
    while pos + 9 < bytes.len() {
        if bytes[pos] != 0xFF {
            pos += 1;
            continue;
        }
        let marker = bytes[pos + 1];
        pos += 2;
        if marker == 0xD9 || marker == 0xDA {
            break;
        }
        if pos + 2 > bytes.len() {
            break;
        }
        let len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        if len < 2 || pos + len > bytes.len() {
            break;
        }
        if matches!(
            marker,
            0xC0 | 0xC1
                | 0xC2
                | 0xC3
                | 0xC5
                | 0xC6
                | 0xC7
                | 0xC9
                | 0xCA
                | 0xCB
                | 0xCD
                | 0xCE
                | 0xCF
        ) {
            let height = u16::from_be_bytes([bytes[pos + 3], bytes[pos + 4]]) as u32;
            let width = u16::from_be_bytes([bytes[pos + 5], bytes[pos + 6]]) as u32;
            return Some((width, height));
        }
        pos += len;
    }
    None
}

fn run_tts_stage(
    job_dir: &Path,
    request: &PipelineRequest,
    recap_text: &str,
) -> Result<StageArtifact, String> {
    let request_path = job_dir.join("tts_request.json");
    fs::write(
        &request_path,
        format!(
            "{{\n  \"provider\": \"windows-sapi-or-external\",\n  \"voice\": \"{}\",\n  \"text_path\": \"{}\"\n}}\n",
            json_escape(&request.voice),
            json_escape_path(&job_dir.join("recap.txt"))
        ),
    )
    .map_err(|err| format!("cannot write TTS request: {err}"))?;

    if !request.synthesize_voice {
        return Ok(StageArtifact {
            stage: "tts".to_string(),
            status: "ready".to_string(),
            path: Some(request_path),
            message:
                "TTS request prepared; pass --synthesize-voice to create narration.wav via Windows SAPI"
                    .to_string(),
        });
    }

    let wav_path = job_dir.join("narration.wav");
    match synthesize_windows_sapi(recap_text, &wav_path) {
        Ok(()) => Ok(StageArtifact {
            stage: "tts".to_string(),
            status: "completed".to_string(),
            path: Some(wav_path),
            message: "narration.wav generated with Windows SAPI".to_string(),
        }),
        Err(err) => Ok(StageArtifact {
            stage: "tts".to_string(),
            status: "failed".to_string(),
            path: Some(request_path),
            message: err,
        }),
    }
}

fn synthesize_windows_sapi(text: &str, wav_path: &Path) -> Result<(), String> {
    let escaped_text = text.replace('\'', "''");
    let escaped_path = wav_path.to_string_lossy().replace('\'', "''");
    let script = format!(
        "Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.SetOutputToWaveFile('{escaped_path}'); $s.Speak('{escaped_text}'); $s.Dispose()"
    );
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .output()
        .map_err(|err| format!("failed to start Windows SAPI synthesis: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Windows SAPI synthesis failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn run_music_stage(
    job_dir: &Path,
    request: &PipelineRequest,
) -> Result<(Option<PathBuf>, StageArtifact), String> {
    let selected = request
        .background_music
        .clone()
        .filter(|path| path.is_file())
        .or_else(|| select_music_track(&request.music_dir, &request.music_mood));
    let path = job_dir.join("music_selection.json");
    let selected_json = selected
        .as_ref()
        .map(|path| format!("\"{}\"", json_escape_path(path)))
        .unwrap_or_else(|| "null".to_string());
    let status = if selected.is_some() {
        "selected"
    } else {
        "empty"
    };
    let message = if selected.is_some() {
        "background music selected"
    } else {
        "no background music found; add tracks to assets/music or pass --background-music"
    };

    fs::write(
        &path,
        format!(
            "{{\n  \"music_dir\": \"{}\",\n  \"mood\": \"{}\",\n  \"selected\": {}\n}}\n",
            json_escape_path(&request.music_dir),
            json_escape(&request.music_mood),
            selected_json
        ),
    )
    .map_err(|err| format!("cannot write music selection: {err}"))?;

    Ok((
        selected,
        StageArtifact {
            stage: "music".to_string(),
            status: status.to_string(),
            path: Some(path),
            message: message.to_string(),
        },
    ))
}

fn select_music_track(music_dir: &Path, mood: &str) -> Option<PathBuf> {
    let mut tracks =
        collect_media_files(music_dir, &["mp3", "wav", "ogg", "m4a", "aac", "flac"]).ok()?;
    if tracks.is_empty() {
        return None;
    }
    tracks.sort_by_key(|path| {
        let name = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let mood_match = if name.contains(&mood.to_ascii_lowercase()) {
            0
        } else {
            1
        };
        (mood_match, name)
    });
    tracks.into_iter().next()
}

fn run_audio_mix_stage(
    job_dir: &Path,
    pairs: &[MediaPair],
    selected_music: Option<&Path>,
) -> Result<StageArtifact, String> {
    let path = job_dir.join("audio_mix.json");
    let narration = job_dir.join("narration.wav");
    let mut json = String::from("{\n");
    json.push_str("  \"strategy\": \"narration-plus-source-audio-plan\",\n");
    json.push_str(&format!(
        "  \"narration\": {},\n",
        if narration.is_file() {
            format!("\"{}\"", json_escape_path(&narration))
        } else {
            "null".to_string()
        }
    ));
    json.push_str(&format!(
        "  \"background_music\": {},\n",
        selected_music
            .map(|path| format!("\"{}\"", json_escape_path(path)))
            .unwrap_or_else(|| "null".to_string())
    ));
    json.push_str("  \"source_audio\": [\n");
    for (idx, pair) in pairs.iter().enumerate() {
        json.push_str(&format!(
            "    {{\"index\":{},\"path\":\"{}\",\"volume\":1.0}}",
            pair.index,
            json_escape_path(&pair.audio)
        ));
        if idx + 1 != pairs.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ],\n");
    json.push_str("  \"background_volume\": 0.25,\n");
    json.push_str("  \"narration_volume\": 1.0\n");
    json.push_str("}\n");

    fs::write(&path, json).map_err(|err| format!("cannot write audio mix plan: {err}"))?;

    Ok(StageArtifact {
        stage: "audio_mix".to_string(),
        status: "ready".to_string(),
        path: Some(path),
        message: "audio mix plan created; ffmpeg can render the final mix when installed"
            .to_string(),
    })
}

fn render_video(
    request: &PipelineRequest,
    pairs: &[MediaPair],
    job_dir: &Path,
) -> Result<PathBuf, String> {
    let segments_dir = job_dir.join("segments");
    fs::create_dir_all(&segments_dir)
        .map_err(|err| format!("cannot create segments dir: {err}"))?;

    let mut concat_list = String::new();
    for pair in pairs {
        let segment = segments_dir.join(format!("{:04}.mp4", pair.index));
        let vf = format!(
            "scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}",
            request.width, request.height, request.width, request.height
        );
        let output = Command::new("ffmpeg")
            .arg("-y")
            .arg("-loop")
            .arg("1")
            .arg("-i")
            .arg(&pair.image)
            .arg("-i")
            .arg(&pair.audio)
            .arg("-vf")
            .arg(vf)
            .arg("-shortest")
            .arg("-r")
            .arg(request.fps.to_string())
            .arg("-c:v")
            .arg("libx264")
            .arg("-tune")
            .arg("stillimage")
            .arg("-c:a")
            .arg("aac")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg(&segment)
            .output()
            .map_err(|err| {
                format!("failed to start ffmpeg: {err}. Install ffmpeg or run without --render.")
            })?;

        if !output.status.success() {
            return Err(format!(
                "ffmpeg segment render failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        concat_list.push_str(&format!("file '{}'\n", segment.display()));
    }

    let list_path = job_dir.join("concat.txt");
    fs::write(&list_path, concat_list).map_err(|err| format!("cannot write concat list: {err}"))?;

    let output_video = job_dir.join("result.mp4");
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_path)
        .arg("-c")
        .arg("copy")
        .arg(&output_video)
        .output()
        .map_err(|err| format!("failed to start ffmpeg concat: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "ffmpeg concat failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output_video)
}

fn write_manifest(
    job_dir: &Path,
    request: &PipelineRequest,
    pairs: &[MediaPair],
) -> Result<(), String> {
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!(
        "  \"pairing\": \"{}\",\n",
        request.pairing.as_str()
    ));
    json.push_str(&format!("  \"render\": {},\n", request.render));
    json.push_str(&format!("  \"width\": {},\n", request.width));
    json.push_str(&format!("  \"height\": {},\n", request.height));
    json.push_str(&format!("  \"fps\": {},\n", request.fps));
    json.push_str(&format!(
        "  \"language\": \"{}\",\n",
        json_escape(&request.language)
    ));
    json.push_str(&format!(
        "  \"recap_style\": \"{}\",\n",
        json_escape(&request.recap_style)
    ));
    json.push_str(&format!(
        "  \"voice\": \"{}\",\n",
        json_escape(&request.voice)
    ));
    json.push_str(&format!(
        "  \"synthesize_voice\": {},\n",
        request.synthesize_voice
    ));
    json.push_str(&format!(
        "  \"images_dir\": \"{}\",\n",
        json_escape_path(&request.images_dir)
    ));
    json.push_str(&format!(
        "  \"audio_dir\": \"{}\",\n",
        json_escape_path(&request.audio_dir)
    ));
    if let Some(texts_dir) = &request.texts_dir {
        json.push_str(&format!(
            "  \"texts_dir\": \"{}\",\n",
            json_escape_path(texts_dir)
        ));
    }
    if let Some(background_music) = &request.background_music {
        json.push_str(&format!(
            "  \"background_music\": \"{}\",\n",
            json_escape_path(background_music)
        ));
    }
    json.push_str(&format!(
        "  \"music_dir\": \"{}\",\n",
        json_escape_path(&request.music_dir)
    ));
    json.push_str(&format!(
        "  \"music_mood\": \"{}\",\n",
        json_escape(&request.music_mood)
    ));
    if let Some(ml_command) = &request.ml_command {
        json.push_str(&format!(
            "  \"ml_command\": \"{}\",\n",
            json_escape(ml_command)
        ));
    }
    json.push_str("  \"pairs\": [\n");
    for (idx, pair) in pairs.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"index\": {},\n", pair.index));
        json.push_str(&format!(
            "      \"image\": \"{}\",\n",
            json_escape_path(&pair.image)
        ));
        json.push_str(&format!(
            "      \"audio\": \"{}\"\n",
            json_escape_path(&pair.audio)
        ));
        json.push_str("    }");
        if idx + 1 != pairs.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ]\n");
    json.push_str("}\n");

    fs::write(job_dir.join("manifest.json"), json)
        .map_err(|err| format!("cannot write manifest: {err}"))
}

fn write_status(result: &JobResult) -> Result<(), String> {
    fs::write(result.job_dir.join("status.json"), job_result_json(result))
        .map_err(|err| format!("cannot write status: {err}"))
}

fn job_result_json(result: &JobResult) -> String {
    let output_video = result
        .output_video
        .as_ref()
        .map(|path| format!("\"{}\"", json_escape_path(path)))
        .unwrap_or_else(|| "null".to_string());

    let mut json = format!(
        "{{\n  \"job_id\": \"{}\",\n  \"status\": \"{}\",\n  \"pairs_count\": {},\n  \"job_dir\": \"{}\",\n  \"output_video\": {},\n  \"message\": \"{}\"",
        json_escape(&result.job_id),
        json_escape(&result.status),
        result.pairs_count,
        json_escape_path(&result.job_dir),
        output_video,
        json_escape(&result.message)
    );
    json.push_str(",\n  \"artifacts\": [\n");
    for (idx, artifact) in result.artifacts.iter().enumerate() {
        let path = artifact
            .path
            .as_ref()
            .map(|path| format!("\"{}\"", json_escape_path(path)))
            .unwrap_or_else(|| "null".to_string());
        json.push_str(&format!(
            "    {{\"stage\":\"{}\",\"status\":\"{}\",\"path\":{},\"message\":\"{}\"}}",
            json_escape(&artifact.stage),
            json_escape(&artifact.status),
            path,
            json_escape(&artifact.message)
        ));
        if idx + 1 != result.artifacts.len() {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ]\n}\n");
    json
}

fn serve(addr: String, request: PipelineRequest) -> Result<(), String> {
    let listener = TcpListener::bind(&addr).map_err(|err| format!("cannot bind {addr}: {err}"))?;
    let state = ServerState {
        default_request: request,
        jobs: Arc::new(Mutex::new(HashMap::new())),
    };
    println!("backend listening on http://{addr}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = state.clone();
                std::thread::spawn(move || {
                    if let Err(err) = handle_connection(stream, state) {
                        eprintln!("request error: {err}");
                    }
                });
            }
            Err(err) => eprintln!("connection failed: {err}"),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, state: ServerState) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|err| format!("cannot set read timeout: {err}"))?;
    let request = read_http_request(&mut stream)?;
    let (status, body) = route_request(&request, state);
    write_http_response(&mut stream, status, &body)
}

struct HttpRequest {
    method: String,
    path: String,
    body: String,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut raw = Vec::new();
    let mut buffer = [0_u8; 4096];

    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|err| format!("cannot read request: {err}"))?;
        if read == 0 {
            break;
        }
        raw.extend_from_slice(&buffer[..read]);
        if let Some(header_end) = find_header_end(&raw) {
            let header_text = String::from_utf8_lossy(&raw[..header_end]).to_string();
            let content_length = parse_content_length(&header_text).unwrap_or(0);
            let total_needed = header_end + 4 + content_length;
            while raw.len() < total_needed {
                let read = stream
                    .read(&mut buffer)
                    .map_err(|err| format!("cannot read request body: {err}"))?;
                if read == 0 {
                    break;
                }
                raw.extend_from_slice(&buffer[..read]);
            }
            break;
        }
        if raw.len() > 1024 * 1024 {
            return Err("request too large".to_string());
        }
    }

    let Some(header_end) = find_header_end(&raw) else {
        return Err("invalid HTTP request".to_string());
    };
    let header_text = String::from_utf8_lossy(&raw[..header_end]).to_string();
    let mut lines = header_text.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or("/").to_string();
    let body_start = header_end + 4;
    let body = String::from_utf8_lossy(raw.get(body_start..).unwrap_or_default()).to_string();

    Ok(HttpRequest { method, path, body })
}

fn route_request(request: &HttpRequest, state: ServerState) -> (u16, String) {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/health") | ("GET", "/api/v1/health") => {
            (200, "{\"status\":\"ok\"}\n".to_string())
        }
        ("POST", "/api/v1/dubbing/start") => {
            let mut pipeline_request = state.default_request.clone();
            apply_json_to_request(&request.body, &mut pipeline_request);
            match run_pipeline(&pipeline_request) {
                Ok(result) => {
                    if let Ok(mut jobs) = state.jobs.lock() {
                        jobs.insert(result.job_id.clone(), result.job_dir.clone());
                    }
                    (200, job_result_json(&result))
                }
                Err(err) => (
                    400,
                    format!(
                        "{{\"status\":\"failed\",\"message\":\"{}\"}}\n",
                        json_escape(&err)
                    ),
                ),
            }
        }
        _ if request.method == "GET" && request.path.starts_with("/api/v1/status/") => {
            let job_id = request.path.trim_start_matches("/api/v1/status/");
            read_job_file(&state, job_id, "status.json")
        }
        _ if request.method == "GET" && request.path.starts_with("/api/v1/result/") => {
            let job_id = request.path.trim_start_matches("/api/v1/result/");
            read_job_file(&state, job_id, "manifest.json")
        }
        _ => (404, "{\"status\":\"not_found\"}\n".to_string()),
    }
}

fn read_job_file(state: &ServerState, job_id: &str, file_name: &str) -> (u16, String) {
    let job_dir = state
        .jobs
        .lock()
        .ok()
        .and_then(|jobs| jobs.get(job_id).cloned())
        .unwrap_or_else(|| state.default_request.output_dir.join("jobs").join(job_id));
    match fs::read_to_string(job_dir.join(file_name)) {
        Ok(body) => (200, body),
        Err(_) => (404, "{\"status\":\"not_found\"}\n".to_string()),
    }
}

fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.as_bytes().len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|err| format!("cannot write response: {err}"))
}

fn apply_json_to_request(body: &str, request: &mut PipelineRequest) {
    if let Some(value) = json_string(body, "images_dir") {
        request.images_dir = PathBuf::from(value);
    }
    if let Some(value) = json_string(body, "audio_dir") {
        request.audio_dir = PathBuf::from(value);
    }
    if let Some(value) = json_string(body, "output_dir") {
        request.output_dir = PathBuf::from(value);
    }
    if let Some(value) = json_string(body, "texts_dir") {
        request.texts_dir = Some(PathBuf::from(value));
    }
    if let Some(value) = json_string(body, "background_music") {
        request.background_music = Some(PathBuf::from(value));
    }
    if let Some(value) = json_string(body, "music_dir") {
        request.music_dir = PathBuf::from(value);
    }
    if let Some(value) = json_string(body, "music_mood") {
        request.music_mood = value;
    }
    if let Some(value) = json_string(body, "ml_command") {
        request.ml_command = Some(value);
    }
    if let Some(value) = json_string(body, "pairing") {
        request.pairing = PairingMode::parse(&value);
    }
    if let Some(value) = json_string(body, "language") {
        request.language = value;
    }
    if let Some(value) = json_string(body, "recap_style") {
        request.recap_style = value;
    }
    if let Some(value) = json_string(body, "voice") {
        request.voice = value;
    }
    if let Some(value) = json_bool(body, "render") {
        request.render = value;
    }
    if let Some(value) = json_bool(body, "synthesize_voice") {
        request.synthesize_voice = value;
    }
    if let Some(value) = json_u32(body, "width") {
        request.width = value;
    }
    if let Some(value) = json_u32(body, "height") {
        request.height = value;
    }
    if let Some(value) = json_u32(body, "fps") {
        request.fps = value;
    }
}

fn json_string(body: &str, key: &str) -> Option<String> {
    let value = json_value_start(body, key)?;
    let mut chars = value.chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            other => out.push(other),
        }
    }
    None
}

fn json_bool(body: &str, key: &str) -> Option<bool> {
    let value = json_value_start(body, key)?;
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn json_u32(body: &str, key: &str) -> Option<u32> {
    let value = json_value_start(body, key)?;
    let digits: String = value.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn json_value_start<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let key_pos = body.find(&needle)?;
    let after_key = &body[key_pos + needle.len()..];
    let colon_pos = after_key.find(':')?;
    Some(after_key[colon_pos + 1..].trim_start())
}

fn parse_content_length(headers: &str) -> Option<usize> {
    for line in headers.lines() {
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                return value.trim().parse().ok();
            }
        }
    }
    None
}

fn find_header_end(raw: &[u8]) -> Option<usize> {
    raw.windows(4).position(|window| window == b"\r\n\r\n")
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    for idx in 0..args.len() {
        if args[idx] == name {
            return args.get(idx + 1).cloned();
        }
        if let Some(value) = args[idx].strip_prefix(&format!("{name}=")) {
            return Some(value.to_string());
        }
    }
    None
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn new_job_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("job-{millis}")
}

fn json_escape_path(path: &Path) -> String {
    json_escape(&path.to_string_lossy())
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn print_help() {
    println!(
        "manhwa-dubbing-backend

USAGE:
  cargo run -- --addr 127.0.0.1:8000
  cargo run -- --once
  cargo run -- --once --images <images-dir> --audio <audio-dir> --output <output-dir>
  cargo run -- --once --render

HTTP:
  GET  /health
  POST /api/v1/dubbing/start
  GET  /api/v1/status/<job_id>
  GET  /api/v1/result/<job_id>

JSON body fields:
  images_dir, audio_dir, texts_dir, background_music, music_dir, music_mood, ml_command
  output_dir, pairing=sequential|stem
  language, recap_style, voice, synthesize_voice=true|false
  render=true|false, width, height, fps
"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_pairing_uses_sorted_order() {
        let root = test_root("sequential_pairing");
        let images = root.join("images");
        let audio = root.join("audio");
        fs::create_dir_all(&images).unwrap();
        fs::create_dir_all(&audio).unwrap();
        fs::write(images.join("b.png"), "").unwrap();
        fs::write(images.join("a.png"), "").unwrap();
        fs::write(audio.join("2.ogg"), "").unwrap();
        fs::write(audio.join("1.ogg"), "").unwrap();

        let pairs = discover_pairs(&images, &audio, PairingMode::Sequential).unwrap();

        assert_eq!(pairs.len(), 2);
        assert!(pairs[0].image.ends_with("a.png"));
        assert!(pairs[0].audio.ends_with("1.ogg"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stem_pairing_requires_matching_names() {
        let root = test_root("stem_pairing");
        let images = root.join("images");
        let audio = root.join("audio");
        fs::create_dir_all(&images).unwrap();
        fs::create_dir_all(&audio).unwrap();
        fs::write(images.join("same.png"), "").unwrap();
        fs::write(images.join("only-image.png"), "").unwrap();
        fs::write(audio.join("same.ogg"), "").unwrap();
        fs::write(audio.join("only-audio.ogg"), "").unwrap();

        let pairs = discover_pairs(&images, &audio, PairingMode::Stem).unwrap();

        assert_eq!(pairs.len(), 1);
        assert!(pairs[0].image.ends_with("same.png"));
        assert!(pairs[0].audio.ends_with("same.ogg"));

        let _ = fs::remove_dir_all(root);
    }

    fn test_root(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("manhwa_backend_test_{}_{}", name, new_job_id()));
        if root.exists() {
            let _ = fs::remove_dir_all(&root);
        }
        root
    }
}
