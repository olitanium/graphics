= Jobs

== Longer term
- Types use Self::Primitive as the inner value. Blocked on RustAnalyser failure,
- Yeet instead of Err(...into()) or Err(...)?

== Can Do Now
- Texture Parameters
  - Min and Mag only for `Image`.
  - gen_tex_set_parameters no longer looks at the `Image`, so image needs to specify the filtering.
  - Depth buffer can choose to select the appropriate compare function.
- Deferred Shading SSAO.
- VecOne periodically when appropriate, to remove unwraps
- Movement Vectors available in Shaders
- impl VertexHasTangent for others
- Model auto-generated normals smooth option
- Renderbuffer Objects (like textures for Framebuffer attachments. Might cause issues for shadows because they are unreadable)
- Multisample FBs
- Light uniform location cacheing without strings
  - `UniformName` trait available
- BoneAnimation don't store poses, store ScLERPS
- Camera ManualPose use Pose
- Camera ManualPose split into two structs
- ~ShaderProgram generics Vertex, Model and `Extra`~
  - Shadow Lights can then be generic over model type
- Import
- Bounding Box dyn
- Bounding Box Collision uses references, specifically for bone_index and unit_index
- Assess Vector<DIMS>: ExtendedVertex bound
- Collision generic over DIMS, but needs a "TransformBy" shenanigans to ensure Pose, Complex Number etc. in all dims