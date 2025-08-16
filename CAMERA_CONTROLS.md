# Camera Controls

The camera system supports keyboard-based movement controls:

## Movement Controls
- **W** or **Up Arrow**: Move forward in the direction the camera is pointing
- **S** or **Down Arrow**: Move backward in the direction the camera is pointing  
- **A** or **Left Arrow**: Strafe left (move perpendicular to camera direction)
- **D** or **Right Arrow**: Strafe right (move perpendicular to camera direction)

## Reset
- **R**: Reset camera orientation to default (looking along negative z-axis)

## Technical Details
- Camera uses yaw and pitch angles for orientation
- Movement is relative to camera direction (FPS-style)
- Camera speed can be adjusted in the `CameraController::new()` constructor
- Mouse controls have been temporarily removed for simplification

## Implementation Notes
- The camera system uses cgmath for 3D mathematics
- Camera orientation is fixed at startup and can be reset with R key
- For future enhancement, mouse look functionality can be re-implemented
