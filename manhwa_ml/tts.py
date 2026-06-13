from __future__ import annotations

import subprocess
from pathlib import Path


def synthesize_windows_sapi(text: str, output_path: Path) -> tuple[str, str]:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    escaped_text = text.replace("'", "''")
    escaped_path = str(output_path).replace("'", "''")
    script = (
        "Add-Type -AssemblyName System.Speech; "
        "$s = New-Object System.Speech.Synthesis.SpeechSynthesizer; "
        f"$s.SetOutputToWaveFile('{escaped_path}'); "
        f"$s.Speak('{escaped_text}'); "
        "$s.Dispose()"
    )
    try:
        result = subprocess.run(
            ["powershell", "-NoProfile", "-Command", script],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError as exc:
        return "failed", f"Windows SAPI unavailable: {exc}"
    if result.returncode != 0:
        return "failed", result.stderr.strip() or "Windows SAPI synthesis failed"
    return "completed", "narration.wav generated with Windows SAPI"
