use std::any::Any;

use crate::framework::backend::BackendTexture;
use crate::framework::error::GameResult;

pub trait EffectParameter {
    fn set_float(&mut self, value: f32) -> GameResult;
    fn set_float2(&mut self, value: [f32; 2]) -> GameResult;
    fn set_float3(&mut self, value: [f32; 3]) -> GameResult;
    fn set_float4(&mut self, value: [f32; 4]) -> GameResult;
    fn set_matrix(&mut self, value: &[[f32; 4]; 4]) -> GameResult;
    fn set_int(&mut self, value: i32) -> GameResult;
    fn set_texture(&mut self, texture: &dyn BackendTexture) -> GameResult;
}

pub trait EffectTechnique {
    fn name(&self) -> &str;
    fn pass_count(&self) -> usize;
}

pub trait BackendEffect: Any {
    fn technique_count(&self) -> usize;
    fn current_technique(&self) -> usize;
    fn set_current_technique(&mut self, index: usize) -> GameResult;
    fn technique_name(&self, index: usize) -> Option<&str>;

    fn pass_count(&self) -> usize;
    fn begin_pass(&mut self, pass_index: usize) -> GameResult;
    fn end_pass(&mut self) -> GameResult;

    fn parameter(&self, name: &str) -> Option<&dyn EffectParameter>;
    fn parameter_mut(&mut self, name: &str) -> Option<&mut dyn EffectParameter>;

    fn as_any(&self) -> &dyn Any;
}
