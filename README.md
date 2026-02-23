# AnimaMac
<video src="https://github-production-user-asset-6210df.s3.amazonaws.com/228816302/553474110-05f4ff94-03b7-45b1-ba9f-9516f48564d1.mp4?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAVCODYLSA53PQK4ZA%2F20260223%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20260223T123132Z&X-Amz-Expires=300&X-Amz-Signature=5450857fe33c546abfc522151491ce73fe43b85917042c39c0abf58b1eb78758&X-Amz-SignedHeaders=host" width="400" controls>
</video>

This is a fork of [AnimaTux](https://github.com/sethblocks/AnimaTux), renamed to AnimaMac. It targets macOS; behavior on other systems is unverified.

## Features
- Play animations or show images on your desktop or whatever app you are using
- Scale slider for image size
- Character library system for multiple characters
- If `ffmpeg` and `img2webp` are installed, selecting an APNG in the file picker will auto-convert to WebP
- If `steamcmd` and `ffmpeg` are installed, original AnimaEngine workshop items downloads are available
- Framerate slider for animations that need speed up or slowed down

## Dependencies
### Build
- Rust toolchain (stable) and `cargo`
