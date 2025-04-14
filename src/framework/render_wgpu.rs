use std::{
    any::Any,
    borrow::Borrow,
    cell::{Cell, Ref, RefCell, RefMut},
    rc::Rc,
};

use imgui::{DrawData, TextureId, Ui};
use wgpu::{
    rwh::{RawDisplayHandle, RawWindowHandle},
    util::{DeviceExt, TextureDataOrder},
    AdapterInfo, Device, Extent3d, Instance, PowerPreference, Queue, RenderPass, Surface, SurfaceTargetUnsafe, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::common::{Color, Rect};

use super::{
    backend::{BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand, VertexData},
    error::{GameError, GameResult},
    graphics::{BlendMode, SwapMode},
    util::field_offset,
};

const fn convert_swap_mode(swap_mode: SwapMode) -> wgpu::PresentMode {
    match swap_mode {
        SwapMode::Immediate => wgpu::PresentMode::AutoNoVsync,
        SwapMode::VSync => wgpu::PresentMode::AutoVsync,
        SwapMode::Adaptive => wgpu::PresentMode::Mailbox,
    }
}

const fn to_wgpu_color(color: Color) -> wgpu::Color {
    wgpu::Color { r: color.r as _, g: color.g as _, b: color.b as _, a: color.a as _ }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VertUBO {
    proj_mtx: [[f32; 4]; 4],
}

impl Default for VertUBO {
    fn default() -> Self {
        VertUBO { proj_mtx: [[0.0; 4]; 4] }
    }
}

struct WGPUContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_texture: RefCell<Option<wgpu::SurfaceTexture>>,
    swap_mode: Cell<SwapMode>,
    encoder: RefCell<Option<wgpu::CommandEncoder>>,
    render_target: RefCell<Option<wgpu::TextureView>>,
    basic_shader: wgpu::ShaderModule,
    vbo: wgpu::Buffer,
    ebo: wgpu::Buffer,
    curr_mtx: RefCell<VertUBO>,
}

impl Drop for WGPUContext {
    fn drop(&mut self) {
        self.surface_texture.replace(None);
        self.render_target.replace(None);
        self.encoder.replace(None);
    }
}

fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Unorm8x4, 2 => Float32x2];

    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<VertexData>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ATTRIBS,
    }
}

impl WGPUContext {
    fn get_encoder(&self) -> RefMut<'_, wgpu::CommandEncoder> {
        {
            let mut encoder = self.encoder.borrow_mut();
            if encoder.is_none() {
                *encoder = Some(self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }));
            }
        }

        RefMut::map(self.encoder.borrow_mut(), |e| e.as_mut().unwrap())
    }

    fn finish_encoder_and_present(&self) {
        let mut encoder = self.encoder.borrow_mut();
        if let Some(encoder) = encoder.take() {
            self.queue.submit(std::iter::once(encoder.finish()));
            let st = self.surface_texture.take();
            if let Some(st) = st {
                st.present();
            }
        }
    }

    fn resize(&self, width: u32, height: u32) -> GameResult {
        if width == 0 || height == 0 {
            return Err(GameError::RenderError("Invalid window size".to_owned()));
        }

        let mut config = self.surface.get_default_config(&self.adapter, width, height);

        if let Some(mut c) = config {
            c.present_mode = convert_swap_mode(self.swap_mode.get());

            self.render_target.replace(None);
            self.surface_texture.replace(None);
            self.surface.configure(&self.device, &c);

            Ok(())
        } else {
            Err(GameError::RenderError("Failed to get default surface configuration".to_owned()))
        }
    }

    fn draw_indexed(&self, vertices: &[VertexData], indices: &[u16], texture: &wgpu::Texture) {
        let mut encoder = self.get_encoder();

        let camera_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: unsafe {
                std::slice::from_raw_parts(
                    &self.curr_mtx.borrow().proj_mtx as *const _ as *const u8,
                    std::mem::size_of::<VertUBO>(),
                )
            },
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let tex_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });
        let camera_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });
        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
            label: None,
        });
        let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }],
            label: None,
        });

        let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.basic_shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            fragment: Some(wgpu::FragmentState {
                module: &self.basic_shader,
                entry_point: Some("fs_main_textured"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.render_target.borrow().as_ref().unwrap(),
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let vbo = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VBO"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    vertices.len() * std::mem::size_of::<VertexData>(),
                )
            },
            usage: wgpu::BufferUsages::VERTEX,
        });

        let ebo = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("EBO"),
            contents: unsafe {
                std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * std::mem::size_of::<u16>())
            },
            usage: wgpu::BufferUsages::INDEX,
        });

        rpass.set_pipeline(&render_pipeline);
        rpass.set_bind_group(0, &texture_bind_group, &[]);
        rpass.set_bind_group(1, &camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, vbo.slice(..));
        rpass.set_index_buffer(ebo.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}

pub struct WGPUTexture {
    ctx: Rc<WGPUContext>,
    image: Rc<wgpu::Texture>,
    vertices: Vec<VertexData>,
    indices: Vec<u16>,
    uv: (f32, f32),
}

impl WGPUTexture {
    fn new(ctx: Rc<WGPUContext>, image: wgpu::Texture) -> Self {
        let (width, height) = (image.size().width, image.size().height);
        let uv = (1.0 / width as f32, 1.0 / height as f32);
        let image = Rc::new(image);

        Self { ctx, image, vertices: Vec::new(), indices: Vec::new(), uv }
    }
}

impl BackendTexture for WGPUTexture {
    fn dimensions(&self) -> (u16, u16) {
        let size = self.image.size();
        (size.width as _, size.height as _)
    }

    fn add(&mut self, command: SpriteBatchCommand) {
        let (tex_scale_x, tex_scale_y) = self.uv;

        match command {
            SpriteBatchCommand::DrawRect(src, dest) => {
                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                ];
                let idx = self.vertices.len() as u16;
                self.indices.extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectFlip(mut src, dest, flip_x, flip_y) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }

                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }

                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                ];
                let idx = self.vertices.len() as u16;
                self.indices.extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectTinted(src, dest, color) => {
                let color = color.to_srgba();
                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                ];
                let idx = self.vertices.len() as u16;
                self.indices.extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectFlipTinted(mut src, dest, flip_x, flip_y, color) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }

                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }

                let color = color.to_srgba();

                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                ];
                let idx = self.vertices.len() as u16;
                self.indices.extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
                self.vertices.extend_from_slice(&vertices);
            }
        }
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    fn draw(&mut self) -> GameResult<()> {
        if self.vertices.is_empty() {
            return Ok(());
        }

        self.ctx.draw_indexed(&self.vertices, &self.indices, &self.image);

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct WGPURenderer {
    imgui: Rc<RefCell<imgui::Context>>,
    ctx: Rc<WGPUContext>,
    size: (u32, u32),
    name: String,
}

impl WGPURenderer {
    pub const RENDERER_ID: &'static str = "wgpu";

    pub fn new(
        mut imgui: imgui::Context,
        raw_display_handle: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
    ) -> GameResult<Box<dyn BackendRenderer>> {
        pollster::block_on(Self::new_async(imgui, raw_display_handle, raw_window_handle))
    }

    // grrrr
    pub async fn new_async(
        mut imgui: imgui::Context,
        raw_display_handle: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
    ) -> GameResult<Box<dyn BackendRenderer>> {
        let _ = imgui.fonts().build_alpha8_texture();

        let instance = wgpu::Instance::default();

        let surface = unsafe {
            instance
                .create_surface_unsafe(SurfaceTargetUnsafe::RawHandle { raw_display_handle, raw_window_handle })
                .map_err(|e: wgpu::CreateSurfaceError| {
                    GameError::RenderError(format!("Failed to create WGPU surface: {:?}", e))
                })?
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await;

        let adapter = if let Some(adapter) = adapter {
            adapter
        } else {
            return Err(GameError::RenderError("No WGPU compatible adapter found".to_owned()));
        };

        let AdapterInfo { name, driver, device_type, backend, .. } = adapter.get_info();
        log::info!("Selected adapter: {} ({}), type: {:?}", name, driver, device_type);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .map_err(|e| GameError::RenderError(format!("Failed to create WGPU device: {:?}", e)))?;

        let config = surface
            .get_default_config(&adapter, 640, 480)
            .ok_or_else(|| GameError::RenderError("Failed to get preferred surface format".to_owned()))?;

        surface.configure(&device, &config);

        let surface_texture = surface
            .get_current_texture()
            .map_err(|e| GameError::RenderError(format!("Failed to get surface texture: {:?}", e)))?;

        let basic_module_desc: wgpu::ShaderModuleDescriptor = wgpu::include_wgsl!("shaders/wgpu/basic.wgsl");
        let basic_shader = device.create_shader_module(basic_module_desc);

        let vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VBO"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ebo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("EBO"),
            contents: &[],
            usage: wgpu::BufferUsages::INDEX,
        });

        let ctx = Rc::new(WGPUContext {
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_texture: RefCell::new(Some(surface_texture)),
            encoder: RefCell::new(None),
            swap_mode: Cell::new(SwapMode::VSync),
            render_target: RefCell::new(None),
            basic_shader,
            vbo,
            ebo,
            curr_mtx: RefCell::new(VertUBO::default()),
        });

        let name = format!("wgpu-rs ({}, {})", backend, name);
        Ok(Box::new(WGPURenderer { imgui: Rc::new(RefCell::new(imgui)), ctx, name, size: (640, 480) }))
    }
}

impl BackendRenderer for WGPURenderer {
    fn renderer_name(&self) -> String {
        self.name.clone()
    }

    fn prepare_draw(&mut self, width: f32, height: f32) -> GameResult {
        let new_size = (width as u32, height as u32);
        if new_size != self.size {
            self.size = new_size;
            self.ctx.resize(self.size.0, self.size.1)?;
        }

        self.ctx.surface_texture.replace(None);
        let surface_texture = self.ctx.surface.get_current_texture();
        let surface_texture = match surface_texture {
            Ok(surface_texture) => surface_texture,
            Err(e) => {
                self.ctx.resize(self.size.0, self.size.1)?;
                self.ctx
                    .surface
                    .get_current_texture()
                    .map_err(|e| GameError::RenderError(format!("Failed to get surface texture: {:?}", e)))?
            }
        };
        self.ctx.surface_texture.replace(Some(surface_texture));

        self.set_render_target(None)?;

        self.ctx.curr_mtx.borrow_mut().proj_mtx = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -height, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];
        Ok(())
    }

    fn clear(&mut self, color: Color) {
        let view = self.ctx.render_target.borrow();
        let view = if let Some(rt) = view.as_ref() {
            rt
        } else {
            return;
        };

        let mut encoder = self.ctx.get_encoder();

        let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    //
                    load: wgpu::LoadOp::Clear(to_wgpu_color(color)),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    fn present(&mut self) -> GameResult {
        self.ctx.finish_encoder_and_present();
        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        let texture = self.ctx.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d { width: width as _, height: height as _, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        Ok(Box::new(WGPUTexture::new(Rc::clone(&self.ctx), texture)))
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        let texture = self.ctx.device.create_texture_with_data(
            &self.ctx.queue,
            &TextureDescriptor {
                label: None,
                size: Extent3d { width: width as _, height: height as _, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            TextureDataOrder::LayerMajor,
            data,
        );

        Ok(Box::new(WGPUTexture::new(Rc::clone(&self.ctx), texture)))
    }

    fn set_blend_mode(&mut self, _blend: BlendMode) -> GameResult {
        Ok(())
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        if let Some(texture) = texture {
            let wgpu_texture = texture
                .as_any()
                .downcast_ref::<WGPUTexture>()
                .ok_or_else(|| GameError::RenderError("This texture was not created by WGPU backend.".to_string()))?;

            let view = wgpu_texture.image.create_view(&wgpu::TextureViewDescriptor::default());
            self.ctx.render_target.replace(Some(view));
        } else {
            let surf_tex = self.ctx.surface_texture.borrow();
            if let Some(surf_tex) = surf_tex.as_ref() {
                let view = surf_tex.texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.ctx.render_target.replace(Some(view));
            }
        }
        Ok(())
    }

    fn draw_rect(&mut self, _rect: Rect<isize>, _color: Color) -> GameResult {
        Ok(())
    }

    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    fn set_clip_rect(&mut self, _rect: Option<Rect>) -> GameResult {
        Ok(())
    }

    fn imgui(&self) -> GameResult<Rc<RefCell<imgui::Context>>> {
        Ok(self.imgui.clone())
    }

    fn imgui_texture_id(&self, _texture: &Box<dyn BackendTexture>) -> GameResult<TextureId> {
        Ok(TextureId::from(0))
    }

    fn prepare_imgui(&mut self, _ui: &Ui) -> GameResult {
        Ok(())
    }

    fn render_imgui(&mut self, _draw_data: &DrawData) -> GameResult {
        Ok(())
    }

    fn draw_triangle_list(
        &mut self,
        _vertices: &[VertexData],
        _texture: Option<&Box<dyn BackendTexture>>,
        _shader: BackendShader,
    ) -> GameResult<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
