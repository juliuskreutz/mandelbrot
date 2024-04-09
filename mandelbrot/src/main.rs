use std::{iter, mem};

use anyhow::{Context, Result};
use shared::{bytemuck, Params};
use wgpu::{util::DeviceExt, StoreOp};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, WindowBuilder},
};

const VERTICES: &[[f32; 2]] = &[[-1.0, 1.0], [-1.0, -1.0], [1.0, -1.0], [1.0, 1.0]];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

fn main() -> Result<()> {
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = WindowBuilder::new()
        .with_title("Mandelbrot")
        .with_resizable(false)
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)?;
    let PhysicalSize { width, height } = window.inner_size();

    let instance_descriptor = wgpu::InstanceDescriptor::default();
    let instance = wgpu::Instance::new(instance_descriptor);
    let surface = instance.create_surface(&window)?;

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .context("No adapter found")?;

    let required_features = wgpu::Features::PUSH_CONSTANTS | wgpu::Features::SHADER_F64;
    let required_limits = wgpu::Limits {
        max_push_constant_size: 128,
        ..Default::default()
    };
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features,
                required_limits,
                ..Default::default()
            },
            None,
        )
        .await?;

    let mut config = surface.get_default_config(&adapter, width, height).unwrap();
    surface.configure(&device, &config);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let shader = device.create_shader_module(wgpu::include_spirv!(env!("shader.spv")));

    let push_constant_range = wgpu::PushConstantRange {
        stages: wgpu::ShaderStages::FRAGMENT,
        range: 0..std::mem::size_of::<Params>() as u32,
    };

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[push_constant_range],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "main_vs",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "main_fs",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let mut params = Params::new(width, height, 100);
    let (mut mouse_x, mut mouse_y) = (0., 0.);

    event_loop.run(|event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    params.width = size.width;
                    params.height = size.height;
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&device, &config);
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => control_flow.exit(),
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                mouse_x = *x;
                mouse_y = *y;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::ArrowUp),
                        ..
                    },
                ..
            } => params.iterations += 100,
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::ArrowDown),
                        ..
                    },
                ..
            } => params.iterations -= 100,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                params.middle_x +=
                    ((mouse_x - params.width as f64 / 2.) / params.width as f64 * 3.) * params.zoom;

                params.middle_y += ((mouse_y - params.height as f64 / 2.) / params.height as f64
                    * 2.)
                    * params.zoom;

                params.zoom /= 2.0;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                params.zoom *= 2.0;
            }
            WindowEvent::RedrawRequested => {
                let output = surface.get_current_texture().unwrap();
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    });

                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_push_constants(
                        wgpu::ShaderStages::FRAGMENT,
                        0,
                        bytemuck::bytes_of(&params),
                    );
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
                }

                queue.submit(iter::once(encoder.finish()));
                output.present();
            }
            _ => {}
        },
        Event::AboutToWait => {
            window.request_redraw();
        }
        _ => {}
    })?;

    Ok(())
}
