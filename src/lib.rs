#![allow(unused)]

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use serde::Serialize;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

struct Buffer {
    allocation: wgpu::Buffer,
    size: usize,
}

pub struct RenderPass {
    render_pipeline: wgpu::RenderPipeline,
    pipeline_layout: wgpu::PipelineLayout,
    common_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Buffer,
    index_buffer: Vec<Buffer>,
    vertex_buffer: Vec<Buffer>,
    sampler: wgpu::Sampler,
}

impl RenderPass {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let common_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("egui common bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
            });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("egui texture bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                    },
                    count: None,
                }],
            });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("egui sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let uniform_buffer = Buffer {
            allocation: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("egui uniform buffer"),
                size: std::mem::size_of::<egui::Vec2>() as u64,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: false,
            }),
            size: std::mem::size_of::<egui::Vec2>(),
        };
        let common_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("egui common bind group"),
            layout: &common_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.allocation.slice(..)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],


        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("egui pipeline layout"),
            bind_group_layouts: &[&common_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!(env!("vert.spv")));
        let fs_module = device.create_shader_module(wgpu::include_spirv!(env!("frag.spv")));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("egui render pipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[format.into()],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![ 0 => Float2, 1 => Float2, 2 =>  Uint ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            uniform_buffer,
            render_pipeline,
            pipeline_layout,
            common_bind_group,
            texture_bind_group: None,
            index_buffer: vec![],
            vertex_buffer: vec![],
            sampler,
            texture_bind_group_layout,
        }
    }

    pub fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: match clear_color {
                        Some(color) => wgpu::LoadOp::Clear(color),
                        None => wgpu::LoadOp::Load,
                    },
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        for i in 0..self.index_buffer.len() {
            render_pass.set_index_buffer(self.index_buffer[i].allocation.slice(..));
            render_pass.set_vertex_buffer(0, self.vertex_buffer[i].allocation.slice(..));
            render_pass.set_bind_group(0, &self.common_bind_group, &[]);
            render_pass.set_bind_group(1, self.texture_bind_group.as_ref().unwrap(), &[]);
            let index_count = self.index_buffer[i].size / std::mem::size_of::<u32>();
            log::debug!("draw {} vertices", index_count);
            render_pass.draw_indexed(0..index_count as u32, 0, 0..1);
        }
    }

    pub fn upload_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        egui_texture: Arc<egui::Texture>,
    ) {
        let texture_size = wgpu::Extent3d {
            width: egui_texture.width as u32,
            height: egui_texture.height as u32,
            depth: 1,
        };
        let texture_format = wgpu::TextureFormat::R8Unorm;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("egui texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            egui_texture.pixels.as_slice(),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: texture_size.width,
                rows_per_image: texture_size.height,
            },
            texture_size,
        );

        self.texture_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("egui texture bind group"),
            layout: &self.texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        }));
    }

    pub fn upload_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: egui::Vec2,
        paint_jobs: &egui::PaintJobs,
    ) {
        self.index_buffer.clear();
        self.vertex_buffer.clear();

        for (_, triangles) in paint_jobs {
            let index_data = bytemuck::cast_slice(&triangles.indices);
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("egui index buffer"),
                contents: &index_data,
                usage: wgpu::BufferUsage::INDEX,
            });

            let vertices: Vec<Vertex> = triangles
                .vertices
                .iter()
                .map(|v| Vertex {
                    position: [v.pos.x, v.pos.y],
                    uv: [v.uv.x, v.uv.y],
                    color: [
                        v.color.r() as f32 / 255.0,
                        v.color.g() as f32 / 255.0,
                        v.color.b() as f32 / 255.0,
                        v.color.a() as f32 / 255.0,
                    ],
                })
                .collect();

            let vertex_data = bytemuck::cast_slice(&vertices);
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("egui vertex buffer"),
                contents: &vertex_data,
                usage: wgpu::BufferUsage::VERTEX,
            });

            self.index_buffer.push(Buffer {
                allocation: index_buffer,
                size: index_data.len(),
            });
            self.vertex_buffer.push(Buffer {
                allocation: vertex_buffer,
                size: vertex_data.len(),
            });
        }

        queue.write_buffer(
            &self.uniform_buffer.allocation,
            0,
            &bincode::serialize(&screen_size).unwrap(),
        );
    }
}
