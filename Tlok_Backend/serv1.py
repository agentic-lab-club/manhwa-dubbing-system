import os
import sys
from pathlib import Path
from typing import List, Optional, Tuple

from moviepy import (
	AudioFileClip,
	CompositeVideoClip,
	ImageClip,
	concatenate_videoclips,
)


def find_matching_pairs(images_dir: Path, sounds_dir: Path) -> List[Tuple[Path, Path]]:
	image_exts = {".png", ".jpg", ".jpeg", ".webp", ".bmp"}
	sound_exts = {".mp3", ".wav", ".m4a", ".aac", ".flac", ".ogg"}

	images = {}
	for p in images_dir.iterdir():
		if p.is_file() and p.suffix.lower() in image_exts:
			images[p.stem] = p

	sounds = {}
	for p in sounds_dir.iterdir():
		if p.is_file() and p.suffix.lower() in sound_exts:
			sounds[p.stem] = p

	common_keys = sorted(set(images.keys()) & set(sounds.keys()))
	return [(images[k], sounds[k]) for k in common_keys]


def create_vertical_pan_clip(
	image_path: Path,
	duration: float,
	final_size: Tuple[int, int],
	bg_color: Tuple[int, int, int] = (0, 0, 0),
) -> ImageClip:
	base = ImageClip(str(image_path)).set_duration(duration)
	W, H = final_size

	img_w, img_h = base.size
	scale = max(W / img_w, H / img_h)
	scaled = base.resize(scale)

	sw, sh = scaled.size
	max_y_offset = max(0, sh - H)

	def position_at_time(t: float) -> Tuple[int, int]:
		if duration <= 0:
			return int((W - sw) / 2), int((H - sh) / 2)
		progress = min(max(t / duration, 0.0), 1.0)
		y_offset = int(max_y_offset - progress * max_y_offset)
		return int((W - sw) / 2), -y_offset

	return scaled.set_position(position_at_time).on_color(size=(W, H), color=bg_color)


def build_video(
	images_dir: Path,
	sounds_dir: Path,
	output_path: Path,
	final_size: Tuple[int, int] = (1080, 1920),
	fps: int = 30,
	bg_color: Tuple[int, int, int] = (0, 0, 0),
) -> Optional[Path]:
	pairs = find_matching_pairs(images_dir, sounds_dir)
	if not pairs:
		print("Не найдено совпадающих пар по имени файла между папками 'foto' и 'zvyki'.")
		return None

	clips: List[CompositeVideoClip] = []
	audios: List[AudioFileClip] = []
	try:
		for img_path, snd_path in pairs:
			audio = AudioFileClip(str(snd_path))
			dur = float(audio.duration)
			img_clip = create_vertical_pan_clip(img_path, dur, final_size, bg_color)
			clip = CompositeVideoClip([img_clip]).set_audio(audio).set_duration(dur)
			clips.append(clip)
			audios.append(audio)

		final = concatenate_videoclips(clips, method="compose")
		output_path.parent.mkdir(parents=True, exist_ok=True)
		final.write_videofile(
			str(output_path),
			fps=fps,
			codec="libx264",
			preset="medium",
			audio_codec="aac",
			threads=os.cpu_count() or 4,
		)
		return output_path
	finally:
		# Cleanup to avoid Windows file locks
		for c in clips:
			try:
				c.close()
			except Exception:
				pass
		for a in audios:
			try:
				a.close()
			except Exception:
				pass


def main():
	base = Path(__file__).resolve().parent.parent
	images_dir = base / "foto"
	sounds_dir = base / "zvyki"
	output = base / "end" / "result.mp4"

	args = sys.argv[1:]
	if len(args) >= 1:
		images_dir = Path(args[0])
	if len(args) >= 2:
		sounds_dir = Path(args[1])
	if len(args) >= 3:
		output = Path(args[2])
	if len(args) >= 5:
		try:
			final_size = (int(args[3]), int(args[4]))
		except ValueError:
			print("Ширина/высота должны быть целыми числами.")
			return
	else:
		final_size = (1080, 1920)
	if len(args) >= 6:
		try:
			fps = int(args[5])
		except ValueError:
			print("FPS должно быть целым числом.")
			return
	else:
		fps = 30

	if not images_dir.exists() or not images_dir.is_dir():
		print(f"Папка с фото не найдена: {images_dir}")
		return
	if not sounds_dir.exists() or not sounds_dir.is_dir():
		print(f"Папка со звуками не найдена: {sounds_dir}")
		return

	res = build_video(images_dir, sounds_dir, output, final_size=final_size, fps=fps)
	if res is None:
		return
	print(f"Готово: {res}")


if __name__ == "__main__":
	main()


