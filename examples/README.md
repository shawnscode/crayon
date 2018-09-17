# Examples

Pre-compiled assets are placed under `resources` folder for convenience, so you could run examples without having `crayon-cli` tools. And notes that assets/resources are stored with `LFS`, please makes sure you have [git-lfs](https://git-lfs.github.com/) installed.

## Core

1. Texture: ```cargo run --bin texture```
2. RenderTexture: ```cargo run --bin render_texture```
3. Input: ```cargo run --bin input```

![RenderTexture](./screenshots/render_texture.png)

## 3D

1. Cube: ```cargo run --bin cube```
2. Prefab: ```cargo run --bin prefab```
3. Saturn ```cargo run --bin saturn```

![Cube](./screenshots/cube.png)
![Prefab](./screenshots/prefab.png)
![Saturn](./screenshots/saturn.png)

## ImGui

1. ImGui windows: ```cargo run --bin imgui```

![ImGui](./screenshots/imgui.png)

## Audio

1. Audio: ```cargo run --bin audio```
2. Audio3D: ```cargo run --bin audio_3d```