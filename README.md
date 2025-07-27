# Physics Renderer

A WebGPU-based 3D renderer built with Rust, supporting both native and web platforms.

## Project Structure

The project has been refactored into a modular structure for better organization and maintainability:

### Modules

- **`src/lib.rs`** - Main library entry point and module declarations
- **`src/camera.rs`** - Camera system including:
  - `Camera` - 3D camera with view/projection matrix calculations
  - `CameraController` - Input handling for camera movement
  - `CameraUniform` - GPU buffer representation of camera data
- **`src/geometry.rs`** - Geometry and vertex data:
  - `Vertex` - Vertex structure with position and color
  - `VERTICES` - Triangle vertex data
  - `INDICES` - Index buffer data for efficient rendering
- **`src/renderer.rs`** - Core rendering system:
  - `State` - Main renderer state and WebGPU setup
  - Rendering pipeline configuration
  - Buffer management and draw calls
- **`src/app.rs`** - Application lifecycle management:
  - `App` - Application handler for winit events
  - Platform-specific initialization (native vs web)

### Key Improvements

1. **Separation of Concerns**: Each module has a specific responsibility
2. **Better Encapsulation**: Private fields and public interfaces are clearly defined
3. **Cleaner Dependencies**: Dependencies are properly organized and available for all targets
4. **Improved Maintainability**: Code is easier to understand and modify
5. **Platform Support**: Maintains support for both native and web platforms

## Building and Running

### Native
```bash
cargo run
```

### Web
```bash
wasm-pack build --target web
```

## Controls

- **WASD** or **Arrow Keys** - Move camera
- **R** - Reset camera position
- **Escape** - Exit application

## Dependencies

- `wgpu` - WebGPU API for graphics
- `winit` - Cross-platform window creation and event handling
- `cgmath` - Mathematics library for 3D graphics
- `bytemuck` - Safe transmutation of data types
- `anyhow` - Error handling
- `pollster` - Async runtime for native builds
- `wasm-bindgen` - WebAssembly bindings (web only) 