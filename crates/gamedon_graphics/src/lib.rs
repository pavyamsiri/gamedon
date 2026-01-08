/// A wrapper over wgpu textures.
mod texture;
/// Vertex data.
mod vertex;

use crate::{
    texture::TextureHandle,
    vertex::{INDICES, VERTICES, Vertex},
};
use color_eyre::eyre;
use core::iter;
use std::sync::Arc;
use wgpu::util::DeviceExt as _;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, keyboard::KeyCode, window};
use zerocopy::IntoBytes;

/// A wrapper over wgpu surface errors.
#[derive(Debug, Clone, Copy)]
pub enum SurfaceError {
    /// The surface is either outdated or the swapchain has been lost.
    /// Can be recovered from by recreating the surface.
    SurfaceNeedsRecreation,
    /// Other errors that can occur but are not expected to be recoverable.
    Unrecoverable,
}

impl From<wgpu::SurfaceError> for SurfaceError {
    fn from(value: wgpu::SurfaceError) -> Self {
        match value {
            wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost => {
                SurfaceError::SurfaceNeedsRecreation
            }
            _ => SurfaceError::Unrecoverable,
        }
    }
}

/// The graphics state.
pub struct GraphicsState {
    /// The wgpu surface.
    surface: wgpu::Surface<'static>,
    /// The wgpu device.
    device: wgpu::Device,
    /// The wgpu queue.
    queue: wgpu::Queue,
    /// The surface configuration.
    config: wgpu::SurfaceConfiguration,
    /// Whether the surface has ever been configured.
    is_surface_configured: bool,
    /// The vertex buffer.
    vertex_buffer: wgpu::Buffer,
    /// The index buffer.
    index_buffer: wgpu::Buffer,
    /// The number of indices to draw.
    num_indices: u32,
    /// The texture bind group.
    diffuse_bind_group: wgpu::BindGroup,
    /// The render pipeline.
    render_pipeline: wgpu::RenderPipeline,
    /// The render texture.
    diffuse_texture: TextureHandle,
    /// The window.
    pub window: Arc<window::Window>,
}

impl GraphicsState {
    /// Initialise a new graphics state.
    pub async fn new(window: Arc<window::Window>) -> eyre::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("gamedon_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gamedon_vertex_buffer"),
            contents: VERTICES.as_bytes(),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gamedon_index_buffer"),
            contents: INDICES.as_bytes(),
            usage: wgpu::BufferUsages::INDEX,
        });

        let diffuse_texture = TextureHandle::from_bytes(&device, &queue, &[0; 1024], 16, 16);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("gamedon_texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("gamedon_diffuse_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("gamedon_render_pipeline_layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                immediate_size: 0,
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gamedon_render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview_mask: None,
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: u32::try_from(INDICES.len())
                .expect("number of vertices will never exceed u32."),
            diffuse_bind_group,
            diffuse_texture,
        })
    }

    /// Resize the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    /// Handle key presses.
    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if let (KeyCode::Escape, true) = (code, is_pressed) {
            event_loop.exit();
        }
    }

    /// Update the frame buffer.
    pub fn update_framebuffer(&mut self, data: &[u8], width: u32, height: u32) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.diffuse_texture.handle,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );
    }

    /// Render a frame.
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gamedon_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gamedon_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl GraphicsState {
    /// Return the inner size of the window in pixels i.e. ignoring window decorations.
    pub fn get_inner_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }
}
