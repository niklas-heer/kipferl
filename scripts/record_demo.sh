#!/usr/bin/env bash
# Record genuine stable CLI output; no terminal output is simulated or replaced.
set -euo pipefail
repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"
for tool in vhs ttyd ffmpeg ffprobe cwebp; do
    command -v "$tool" >/dev/null || { echo "Install $tool before recording." >&2; exit 1; }
done
demo_cli=${KIPFERL_DEMO_CLI:-$repo_root/target/published-stable-verification/kipferl-macos-aarch64}
[[ -x "$demo_cli" ]] || { echo "Set KIPFERL_DEMO_CLI to a published v0.7.1 executable for this host." >&2; exit 1; }
case "$demo_cli" in /*) ;; *) demo_cli="$repo_root/$demo_cli" ;; esac
version=$(NO_COLOR=1 "$demo_cli" --version | sed $'s/\033\\[[0-9;]*m//g')
[[ "$version" == 'Kipferl v0.7.1' ]] || { echo "Expected published Kipferl v0.7.1, got: $version" >&2; exit 1; }
DEMO_ROOT=$(mktemp -d /tmp/kipferl-demo.XXXXXX)
export DEMO_ROOT
trap 'rm -rf "$DEMO_ROOT"' EXIT
mkdir -p "$DEMO_ROOT"/{bin,fixtures,dist,state/home,state/cache,state/config,state/runtime,state/tmp}
ln -s "$demo_cli" "$DEMO_ROOT/bin/kipferl"
cp scripts/demo/zones.py "$DEMO_ROOT/fixtures/zones.py"
cat > "$DEMO_ROOT/environment" <<'ENVIRONMENT'
export PATH="$DEMO_ROOT/bin:/usr/bin:/bin"
export HOME="$DEMO_ROOT/state/home" TMPDIR="$DEMO_ROOT/state/tmp"
export XDG_CACHE_HOME="$DEMO_ROOT/state/cache" XDG_CONFIG_HOME="$DEMO_ROOT/state/config"
export KIPFERL_CACHE_DIR="$DEMO_ROOT/state/runtime"
unset PYTHONPATH PYTHONHOME KIPFERL_PACKAGE_INDEX KIPFERL_NO_COLOR NO_COLOR
export LANG=C LC_ALL=C PS1='$ '
unset PROMPT_COMMAND
set -E
trap 'printf "%s\n" "$BASH_COMMAND" >> "$DEMO_ROOT/failed"' ERR
cd "$DEMO_ROOT"
clear
ENVIRONMENT
mkdir -p website/public/demos
# Stage all outputs so a failed command never overwrites the published demo.
sed "s|website/public/demos/kipferl-0.7.1|$DEMO_ROOT/kipferl-0.7.1|g" demo.tape > "$DEMO_ROOT/demo.tape"
VHS_NO_SANDBOX=1 vhs "$DEMO_ROOT/demo.tape"
if [[ -e "$DEMO_ROOT/failed" || ! -e "$DEMO_ROOT/complete" ]]; then
    echo 'Recording failed; do not publish its media.' >&2
    [[ ! -e "$DEMO_ROOT/failed" ]] || cat "$DEMO_ROOT/failed" >&2
    exit 1
fi
# Extract from the finished video: VHS screenshots can race the following clear.
# Adjust this timestamp after a replay if network latency shifts the source view.
ffmpeg -hide_banner -loglevel error -y -ss "${KIPFERL_DEMO_POSTER_SECOND:-27}" \
    -i "$DEMO_ROOT/kipferl-0.7.1.mp4" -frames:v 1 "$DEMO_ROOT/kipferl-0.7.1.png"
cwebp -quiet -lossless "$DEMO_ROOT/kipferl-0.7.1.png" -o "$DEMO_ROOT/kipferl-0.7.1.webp"
ffmpeg -hide_banner -loglevel error -y -i "$DEMO_ROOT/kipferl-0.7.1.mp4" \
    -c copy -movflags +faststart "$DEMO_ROOT/kipferl-0.7.1-stream.mp4"
# Refresh the README/blog GIF from the same verified recording.
ffmpeg -hide_banner -loglevel error -y -i "$DEMO_ROOT/kipferl-0.7.1.mp4" \
    -filter_complex '[0:v]fps=10,scale=960:-1:flags=lanczos,split[a][b];[a]palettegen[p];[b][p]paletteuse' \
    -loop 0 "$DEMO_ROOT/demo.gif"
cp "$DEMO_ROOT/demo.gif" demo.gif
cp "$DEMO_ROOT/demo.gif" website/public/demo.gif
mv "$DEMO_ROOT/kipferl-0.7.1-stream.mp4" website/public/demos/kipferl-0.7.1.mp4
mv "$DEMO_ROOT/kipferl-0.7.1.webp" website/public/demos/kipferl-0.7.1.webp
ffprobe -v error -show_entries format=duration,size -show_entries stream=width,height \
    -of json website/public/demos/kipferl-0.7.1.mp4
