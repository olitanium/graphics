use std::sync::LazyLock;

use graphics::linear_algebra::Vector;
use graphics::shader_program::{CullFace, ShaderProgram};
use graphics::texture::{CubeMap, FlatTexture};
use graphics::vertex::NoVertex;

use crate::modelling::cubic::SkeletonCubic;
use crate::modelling::cubic::lighting::shadow::ShadowListLights;
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::{
    Line,
    Quad,
    QuadVertex,
    SimpleCubic,
    SimpleVertex,
    SkeletalVertex,
    SkyBox,
    SkyBoxVertex,
    TestLineVertex,
    Volume,
};

static ERROR_MESSAGE: &str = "Engine defined shader should exist and not have errors";

macro_rules! make_included {
    ($(: ($first:ident $(, $others:ident )*) ,)? $typ:ty, $fn_name:ident, $vertex:literal, $fragment:literal $(, $geometry:literal)? $(, cull_face: $cull_face:path)? $(,)?) => {
        pub fn $fn_name() -> &'static $typ {
            static PROGRAM: LazyLock<$typ> = LazyLock::new(|| {
                let out: $crate::ApiResult<$typ, !> = try {
                    ShaderProgram::builder()
                    .vertex_shader($vertex)?
                    .fragment_shader($fragment)?
                    $(.geometry_shader($geometry)?)?
                    $(.force_cull_face($cull_face))?
                    .build()?
                };

                out.expect(ERROR_MESSAGE)
            });

            &PROGRAM
        }
    };
}

make_included! {
    ShaderProgram<SimpleVertex, (SimpleCubic, ListLights<2>), 2, FlatTexture>,
    hdr,
    "shaders/hdr_tangent/hdr_tangent.vert",
    "shaders/hdr_tangent/hdr_tangent.frag",
}

make_included! {
    ShaderProgram<SimpleVertex, (SimpleCubic, ListLights<2>), 1, FlatTexture>,
    hdr_without_bright,
    "shaders/hdr_tangent/hdr_tangent.vert",
    "shaders/hdr_tangent/hdr_tangent.frag",
}

make_included! {
    ShaderProgram<SkyBoxVertex, SkyBox, 2, FlatTexture>,
    skybox_hdr,
    "shaders/skycube/skycube.vert",
    "shaders/skycube/skycube.frag",
}

make_included! {
    ShaderProgram<SkyBoxVertex, SkyBox, 1, FlatTexture>,
    skybox_hdr_without_bright,
    "shaders/skycube/skycube.vert",
    "shaders/skycube/skycube.frag",
}

make_included! {
    ShaderProgram<QuadVertex, Quad<1>, 1, FlatTexture>,
    exposure,
    "shaders/quad/quad.vert",
    "shaders/quad/exposure.frag",
}

make_included! {
    ShaderProgram<SimpleVertex, (SimpleCubic, ShadowListLights<2>), 2, FlatTexture>,
    shadow,
    "shaders/hdr_tangent_shadow/hdr_tangent_shadow.vert",
    "shaders/hdr_tangent_shadow/hdr_tangent_shadow.frag",
}

make_included! {
    ShaderProgram<SimpleVertex, (SimpleCubic, ()), 0, FlatTexture>,
    far_light_depth,
    "shaders/depth_testing/farspot_light_depth/farspot_light_depth.vert",
    "shaders/depth_testing/farspot_light_depth/farspot_light_depth.frag",
    cull_face: CullFace::FrontFace,
}

make_included! {
    ShaderProgram<QuadVertex, Quad<1>, 1, FlatTexture>,
    quad,
    "shaders/quad/quad.vert",
    "shaders/quad/quad.frag",
}

make_included! {
    ShaderProgram<QuadVertex, Quad<2>, 2, FlatTexture>,
    bloom_x,
    "shaders/bloom/blur_x/blur_x.vert",
    "shaders/bloom/blur_x/blur_x.frag",
}

make_included! {
    ShaderProgram<QuadVertex, Quad<2>, 1, FlatTexture>,
    bloom_y,
    "shaders/bloom/blur_y_merge/blur_y_merge.vert",
    "shaders/bloom/blur_y_merge/blur_y_merge.frag",
}

make_included! {
    ShaderProgram<SimpleVertex, (SimpleCubic, ()), 0, CubeMap>,
    point_depth,
    "shaders/depth_testing/point_light_depth/point_light_depth.vert",
    "shaders/depth_testing/point_light_depth/point_light_depth.frag",
    "shaders/depth_testing/point_light_depth/point_light_depth.geom",
    cull_face: CullFace::FrontFace,
}

make_included! {
    ShaderProgram<NoVertex, Line<TestLineVertex>, 1, FlatTexture>,
    line_test,
    "shaders/line/line.vert",
    "shaders/line/line.frag",
}

pub const LINE_SHADER_MAX_VERTICES: usize = 100;

make_included! {
    ShaderProgram<SkeletalVertex, (SkeletonCubic, ListLights<2>), 2, FlatTexture>,
    skeletal,
    "shaders/simple_skeleton_blending/simple_skeleton_blending.vert",
    "shaders/hdr_tangent/hdr_tangent.frag",
}

make_included! {
    ShaderProgram<SkeletalVertex, (SkeletonCubic, ListLights<2>), 2, FlatTexture>,
    skeletal_quat,
    "shaders/simple_skeleton_blending_quat/simple_skeleton_blending_quat.vert",
    "shaders/hdr_tangent/hdr_tangent.frag",
}

pub trait BoundingBoxShaderProgram<const DIMS: usize>: Volume<DIMS> {
    fn shader_program() -> &'static ShaderProgram<Self, (), 1, FlatTexture>;
}

impl BoundingBoxShaderProgram<3> for Vector<3> {
    fn shader_program() -> &'static ShaderProgram<Self, (), 1, FlatTexture> {
        static PROGRAM: LazyLock<ShaderProgram<Vector<3>, (), 1, FlatTexture>> =
            LazyLock::new(|| {
                let out: crate::ApiResult<_, !> = try {
                    ShaderProgram::builder()
                        .vertex_shader("shaders/bounding_box/bounding_box.vert")?
                        .fragment_shader("shaders/bounding_box/bounding_box.frag")?
                        .build()?
                };

                out.expect(ERROR_MESSAGE)
            });

        &PROGRAM
    }
}

impl BoundingBoxShaderProgram<2> for Vector<2> {
    fn shader_program() -> &'static ShaderProgram<Self, (), 1, FlatTexture> {
        // TODO
        todo!();
    }
}
