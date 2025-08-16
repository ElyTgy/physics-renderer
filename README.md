# Physics Renderer

A 3D physics renderer built with Rust and WebGPU for rendering instanced 3D objects.

## Features

### 3D Rendering
- WebGPU-based rendering with instanced objects
- Camera controls with WASD movement
- Depth testing and proper 3D perspective
- Texture support for models
- Configurable instance grid with adjustable spacing

## Technical Implementation

### Architecture
- **Rust Backend**: Core rendering logic using WebGPU
- **WASM**: WebAssembly compilation for web deployment

### Key Components

#### Instance Management
```rust
pub struct ModelInstance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: f32,
    pub texture_index: usize,
    pub model_index: usize,
}
```

#### WASM Bindings
The application exposes JavaScript functions for:
- Basic camera controls
- Resetting camera to default position

## Usage

### Building
```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build for web
wasm-pack build --target web

# Serve the application
python3 -m http.server 8000
# or
npx serve .
```

### Controls
- **WASD**: Move camera
- **R**: Reset camera
- **Escape**: Exit application

## File Structure
```
physicsrenderer/
├── src/
│   ├── lib.rs          # Main library with WASM bindings
│   ├── app.rs          # Application event handling
│   ├── renderer.rs     # WebGPU rendering logic
│   ├── camera.rs       # Camera and controller
│   ├── model.rs        # 3D model loading and rendering
│   ├── texture.rs      # Texture loading and management
│   ├── geometry.rs     # Geometric primitives
│   └── resources.rs    # Resource management
├── assets/
│   └── texture.jpg     # Default texture
├── res/
│   ├── cube.obj        # Default 3D model
│   └── textures/
│       └── cube_texture.png
├── demo.html           # Demo page (if needed)
└── Cargo.toml          # Rust dependencies
```

## Future Enhancements

### Planned Features
- **Advanced Camera Controls**: Orbit, pan, zoom controls
- **Material System**: Support for different material types
- **Lighting**: Dynamic lighting and shadows
- **Animation**: Keyframe animation system
- **Physics**: Basic physics simulation
- **Export**: Scene export to common formats

### Technical Improvements
- **Performance**: Optimize rendering for large instance counts
- **Memory Management**: Better resource cleanup
- **Error Handling**: Comprehensive error reporting
- **Testing**: Unit and integration tests
- **Documentation**: API documentation and examples

## Dependencies

### Rust Dependencies
- `wgpu`: WebGPU rendering
- `winit`: Window management
- `cgmath`: Mathematics library
- `bytemuck`: Memory utilities
- `image`: Image processing
- `tobj`: OBJ model loading
- `wasm-bindgen`: WASM bindings

### Web Dependencies
- `web-sys`: Web APIs
- `wasm-bindgen-futures`: Async WASM support
- `console_log`: Logging for web

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 