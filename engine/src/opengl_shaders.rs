use std::sync::LazyLock;

use graphics::shader_program::{CullFace, ShaderProgram};
use graphics::texture::{CubeMap, FlatTexture};

use crate::modelling::cubic::SkinningCubic;
use crate::modelling::cubic::lighting::shadow::ShadowListLights;
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::{Line, Quad, SimpleCubic, SkyBox, TestLineVertex};

static ERROR_MESSAGE: &str = "Engine defined shader should exist and not have errors";

macro_rules! make_included {
    ($typ:ty, $fn_name:ident, $vertex:literal, $fragment:literal $(, $geometry:literal)? $(, cull_face: $cull_face:path)? $(,)?) => {
        pub fn $fn_name() -> &'static $typ {
            static PROGRAM: LazyLock<$typ> = LazyLock::new(|| {
                    ShaderProgram::builder()
                    .vertex_shader($vertex).expect(ERROR_MESSAGE)
                    .fragment_shader($fragment).expect(ERROR_MESSAGE)
                    $(.geometry_shader($geometry).expect(ERROR_MESSAGE))?
                    $(.force_cull_face($cull_face))?
                    .build().expect(ERROR_MESSAGE)
            });

            &PROGRAM
        }
    };
}

make_included! {
    ShaderProgram<SimpleCubic, ListLights<2>, 2, FlatTexture>,
    hdr,
    "shaders/hdr_tangent/hdr_tangent.vert",
    "shaders/hdr_tangent/hdr_tangent.frag",
}

make_included! {
    ShaderProgram<SimpleCubic, ListLights<2>, 1, FlatTexture>,
    hdr_without_bright,
    "shaders/hdr_tangent/hdr_tangent.vert",
    "shaders/hdr_tangent/hdr_tangent.frag",
}

make_included! {
    ShaderProgram<SkyBox, (), 2, FlatTexture>,
    skybox_hdr,
    "shaders/skycube/skycube.vert",
    "shaders/skycube/skycube.frag",
}

make_included! {
    ShaderProgram<SkyBox, (), 1, FlatTexture>,
    skybox_hdr_without_bright,
    "shaders/skycube/skycube.vert",
    "shaders/skycube/skycube.frag",
}

make_included! {
    ShaderProgram<Quad<1>, (), 1, FlatTexture>,
    exposure,
    "shaders/quad/quad.vert",
    "shaders/quad/exposure.frag",
}

make_included! {
    ShaderProgram<SimpleCubic, ShadowListLights<2>, 2, FlatTexture>,
    shadow,
    "shaders/hdr_tangent_shadow/hdr_tangent_shadow.vert",
    "shaders/hdr_tangent_shadow/hdr_tangent_shadow.frag",
}

make_included! {
    ShaderProgram<SimpleCubic, (), 0, FlatTexture>,
    far_light_depth,
    "shaders/depth_testing/farspot_light_depth/farspot_light_depth.vert",
    "shaders/depth_testing/farspot_light_depth/farspot_light_depth.frag",
    cull_face: CullFace::FrontFace,
}

make_included! {
    ShaderProgram<Quad<1>, (), 1, FlatTexture>,
    quad,
    "shaders/quad/quad.vert",
    "shaders/quad/quad.frag",
}

make_included! {
    ShaderProgram<Quad<2>, (), 2, FlatTexture>,
    bloom_x,
    "shaders/bloom/blur_x/blur_x.vert",
    "shaders/bloom/blur_x/blur_x.frag",
}

make_included! {
    ShaderProgram<Quad<2>, (), 1, FlatTexture>,
    bloom_y,
    "shaders/bloom/blur_y_merge/blur_y_merge.vert",
    "shaders/bloom/blur_y_merge/blur_y_merge.frag",
}

make_included! {
    ShaderProgram<SimpleCubic, (), 0, CubeMap>,
    point_depth,
    "shaders/depth_testing/point_light_depth/point_light_depth.vert",
    "shaders/depth_testing/point_light_depth/point_light_depth.frag",
    "shaders/depth_testing/point_light_depth/point_light_depth.geom",
    cull_face: CullFace::FrontFace,
}

make_included! {
    ShaderProgram<Line<TestLineVertex>, (), 1, FlatTexture>,
    line_test,
    "shaders/line/line.vert",
    "shaders/line/line.frag",
}

pub const LINE_SHADER_MAX_VERTICES: usize = 100;

make_included! {
    ShaderProgram<SkinningCubic, ListLights<2>, 2, FlatTexture>,
    skeletal,
    "shaders/simple_skeleton_blending/simple_skeleton_blending.vert",
    "shaders/hdr_tangent/hdr_tangent.frag",
}

make_included! {
    ShaderProgram<SkinningCubic, ListLights<2>, 2, FlatTexture>,
    skeletal_quat,
    "shaders/simple_skeleton_blending_quat/simple_skeleton_blending_quat.vert",
    "shaders/hdr_tangent/hdr_tangent.frag",
}
