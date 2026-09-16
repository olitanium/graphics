= Jobs

== Longer term
- Types use Self::Primitive as the inner value. Blocked on RustAnalyser failure,
- Yeet instead of Err(...into()) or Err(...)?

== Can Do Now
- impl VertexHasTangent for others
- Model auto-generated normals smooth option
- Renderbuffer Objects (like textures for Framebuffer attachments. Might cause issues for shadows because they are unreadable)
- Multisample FBs
- Light uniform location cacheing without strings
- BoneAnimation don't store poses, store ScLERPS
- Camera ManualPose use Pose
- Camera ManualPose split into two structs
- ~ShaderProgram generics Vertex, Model and `Extra`~
  - Shadow Lights can then be generic over model type
- Import
- Bounding Box dyn
- Bounding Box Collision uses refrerences, specifically for bone_index and unit_index
- Assess Vector<DIMS>: ExtendedVertex bound
- Collision generic over DIMS, but needs a "TransformBy" shenanigans to ensure Pose, Complex Number etc. in all dims