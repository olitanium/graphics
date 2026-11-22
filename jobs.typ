= Jobs

== Longer term
- Types use Self::Primitive as the inner value. Blocked on RustAnalyser failure,
- Yeet instead of Err(...into()) or Err(...)?

== Can Do Now
- Const derive for many types
- Environment config with default
- Texture Parameters
  - Depth buffer can choose to select the appropriate compare function.
- Deferred Shading SSAO.
  - blur
- VecOne periodically when appropriate, to remove unwraps
- Movement Vectors available in Shaders
- impl VertexHasTangent for others
- Renderbuffer Objects
- Multisample FBs
- `UniformName` trait work
  - Light uniform location cacheing without strings
- BoneAnimation don't store poses, store ScLERPS
- Camera ManualPose use Pose
- Camera ManualPose split into two structs
- ~ShaderProgram generics Vertex, Model and `Extra`~
  - Shadow Lights can then be generic over model type
- Import
  - Model auto-generated normals smooth option
- Bounding Box dyn
- Bounding Box Collision uses references, specifically for bone_index and unit_index
- Assess Vector<DIMS>: ExtendedVertex bound
- Collision generic over DIMS, but needs a "TransformBy" shenanigans to ensure Pose, Complex Number etc. in all dims