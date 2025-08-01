# Jobs

* Vertex Array auto-generated normals smooth option
* Renderbuffer Objects (like textures for Framebuffer attachments. Might cause issues for shadows because they are unreadable)
* Light uniform location cacheing without strings
* Sort out ActiveFrambuffer and ActiveShaderProgram lifetimes, including elision when possible in other impls.
* Skeletal Animation / Skinning

## Skeletal Animation Notes

each vertex has some number of associated bones. 
each bone has a get_pose() to transform that vertex to the correct position.

BUT animations bugger that up.

To make a single skeleton.get_pose(animation, bone, time), I must adjust the animation.

Present: Bones have a bind_pose, and an animation pose. I need to get for each keyframe of each animation the displacement from the bind to the animation, so that can be passed

for each bone
    let bind_pose = X;
    let inverse_bind_pose = X.inverse();
    for each animation
        for each keyframe
            let keyframe_pose = Y;
            let final_pose = keyframe_pose.apply_after(inverse_bind_pose);

            keyframe_pose = final_pose
            // such that the original keyframe_pose is forgotten
    // after bone is finished, bind_pose (and X) can be forgotten

How to store animations.

Skeleton
    Vec of animations

    fn get_pose(&self, bone, animation, time) -> Pose {
        self.animation.get(bone).get_pose(self, animation, time)
    }

Bone
    Vec of Animations

    fn get_pose(&self, animation, time) -> Pose {
        self.animation.get(animation).get_pose(time)
    }
