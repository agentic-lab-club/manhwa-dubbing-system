from __future__ import annotations


def translate_text(text: str, target_language: str) -> tuple[str, str, str]:
    translated = (
        f"Target language: {target_language}\n\n"
        f"{text}\n\n"
        "Translation note: external translation provider is not configured; source text is preserved."
    )
    return translated, "fallback", "translation fallback generated"
