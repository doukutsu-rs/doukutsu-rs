use gfx::{self, *};
use ggez::graphics::Shader;
use ggez::{Context, GameResult};

gfx_defines! {
    constant WaterShaderParams {
        resolution: [f32; 2] = "u_Resolution",
        frame_pos: [f32; 2] = "u_FramePos",
        t: f32 = "u_Tick",
    }
}

pub struct Shaders {
    pub water_shader: Shader<WaterShaderParams>,
    pub water_shader_params: WaterShaderParams,
}

impl Shaders {
    pub fn new(ctx: &mut Context) -> GameResult<Shaders> {
        let water_shader_params = WaterShaderParams {
            t: 0.0,
            resolution: [0.0, 0.0],
            frame_pos: [0.0, 0.0],
        };

        Ok(Shaders {
            water_shader: Shader::new(
                ctx,
                "/builtin/shaders/basic_150.vert.glsl",
                "/builtin/shaders/water_150.frag.glsl",
                water_shader_params,
                "WaterShaderParams",
                None,
            )?,
            water_shader_params,
        })
    }
}
