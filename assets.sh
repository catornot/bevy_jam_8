#!/usr/bin/env sh

build_glb() {
  file="$1"
  arg="${arg%.blend}"
  blender -b "$file" --python-expr "import bpy; bpy.ops.export_scene.gltf(filepath='$arg.glb')"
}

inotifywait -r -m -e close_write -e moved_to --format '%w%f' ./assets | while IFS= read -r file; do
    [[ "$file" == *.blend ]] || continue

    echo "change in: $file"
    build_glb "$file"
done
