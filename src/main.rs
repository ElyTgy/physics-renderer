use physicsrenderer::App;

fn main() -> anyhow::Result<()> {
    println!("Physics Renderer");
    println!("Controls:");
    println!("  WASD - Move camera");
    println!("  R - Reset camera to default");
    println!("  Escape - Exit");
    println!();
    
    physicsrenderer::run()
} 