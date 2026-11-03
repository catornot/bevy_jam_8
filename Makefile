# Configuration: Defaults to 'blender' in PATH, overridden by shell environment
BLENDER ?= blender
ASSET_DIR ?= assets

# Find all .blend files recursively inside the assets directory
BLEND_FILES := $(shell find $(ASSET_DIR) -name "*.blend")

# Automatically map each .blend file to a corresponding .glb file path
GLB_FILES := $(BLEND_FILES:.blend=.glb)

.PHONY: all run assets clean

# Default target: updates assets, then runs your Rust application
all: run

# Run target depends on the assets being up-to-date
run: assets
	cargo run

# Target that triggers the pattern rule for all discovered .glb files
assets: $(GLB_FILES)

# Pattern rule: passes the .blend file directly to Blender before executing the export script
%.glb: %.blend
	@echo "Converting $< -> $@"
	@$(BLENDER) -b "$<" --python-expr "import bpy; bpy.ops.export_scene.gltf(filepath='$@', export_format='GLB')"

# Optional: Clean up generated .glb files
clean:
	@echo "Cleaning generated .glb files..."
	@rm -f $(GLB_FILES)
