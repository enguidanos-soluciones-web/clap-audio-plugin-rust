#!/usr/bin/env bash
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"

INCLUDES=(
  "-Isrc"
  "-Itarget/cxxbridge"
  "-Iexternal/neural-amp-modeler-0.5.1/NAM"
  "-Iexternal/neural-amp-modeler-0.5.1/Dependencies/eigen"
  "-Iexternal/neural-amp-modeler-0.5.1/Dependencies/nlohmann"
  "-Iexternal/sdl-3.4.4/include"
)
FLAGS="-std=c++20 ${INCLUDES[*]}"

files=(
  "src/nam.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/activations.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/container.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/conv1d.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/convnet.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/dsp.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/get_dsp.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/lstm.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/ring_buffer.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/util.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/wavenet/model.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/wavenet/a2_fast.cpp"
  "external/neural-amp-modeler-0.5.1/NAM/wavenet/slimmable.cpp"
)

entries=()
for f in "${files[@]}"; do
  entries+=("  {\"directory\": \"$ROOT\", \"file\": \"$f\", \"command\": \"clang++ $FLAGS $f\"}")
done

printf '[\n%s\n]\n' "$(IFS=$',\n'; echo "${entries[*]}")" > compile_commands.json
echo "Generated compile_commands.json"
