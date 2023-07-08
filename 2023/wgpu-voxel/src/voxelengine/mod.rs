use std::{f32::consts::PI, time::Instant};

use cgmath::prelude::*;
use cgmath::{Quaternion, Rad, Rotation, Vector3};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use winit::window::Window;

mod buffers;
mod world;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,

    render_input: buffers::Data<buffers::RenderInputData>,
    chunk: buffers::Data<world::Chunk>,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        let render_input = buffers::create_render_input_buffer(&device);
        let chunk = buffers::create_chunk_buffer(&device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_input.layout, &chunk.layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[],           // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,

            render_input,
            chunk,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_bind_group(0, &self.render_input.bind_group, &[]);
            render_pass.set_bind_group(1, &self.chunk.bind_group, &[]);
            render_pass.draw(0..6, 0..2); // 3.
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Default, Debug)]
struct Controlls {
    forward: bool,
    backward: bool,
    right: bool,
    left: bool,
    up: bool,
    down: bool,
    mousex: f32,
    mousey: f32,
}

fn directional_speed(delta: f32, f: f32, t: bool, t2: bool) -> f32 {
    if t && !t2 {
        return f * delta;
    }
    if t2 {
        return -f * delta;
    }
    return 0.;
}

const MOUSE_SENSITIVITY: f32 = 0.001;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state: State = State::new(window).await;
    let mut controller = Controlls::default();
    let mut time = Instant::now();
    let speed = 0.5;
    let mut moving = false;
    let mut prev_mouse_pos: Option<winit::dpi::PhysicalPosition<f64>> = None;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            (state.render_input.data.frame, _) = state.render_input.data.frame.overflowing_add(1);

            let delta = time.elapsed().as_secs_f32();
            time = Instant::now();
            let dx = directional_speed(delta, speed, controller.forward, controller.backward);
            let dy = directional_speed(delta, speed, controller.right, controller.left);
            let dz = directional_speed(delta, speed, controller.up, controller.down);
            let [x, y, z] = state.render_input.data.camera;

            let wasd_vec = Vector3::new(dx, dy, 0.);
            let rot = Quaternion::from_angle_z(Rad(controller.mousex));
            let rot_vel = rot.rotate_vector(wasd_vec);

            state.render_input.data.camera = [x + rot_vel.x, y + rot_vel.y, z + dz];
            state.render_input.data.dir = [controller.mousex, controller.mousey];

            // println!("{:?}", state.render_input.camera);
            state.queue.write_buffer(
                &state.render_input.buffer,
                0,
                bytemuck::cast_slice(&[state.render_input.data]),
            );

            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: pressed,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => {
                        let active = &ElementState::Pressed == pressed;
                        match keycode {
                            VirtualKeyCode::Escape => state
                                .window
                                .set_cursor_grab(winit::window::CursorGrabMode::None)
                                .unwrap(),
                            VirtualKeyCode::W => controller.forward = active,
                            VirtualKeyCode::S => controller.backward = active,
                            VirtualKeyCode::A => controller.left = active,
                            VirtualKeyCode::D => controller.right = active,
                            VirtualKeyCode::Space => controller.up = active,
                            VirtualKeyCode::LShift => controller.down = active,
                            _ => {}
                        };
                    }
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state,
                        ..
                    } => {
                        moving = state == &ElementState::Pressed;
                        // state
                        //     .window
                        //     .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                        //     .unwrap();
                        // state
                        //     .window
                        //     .set_cursor_position(LogicalPosition::new(200, 200))
                        //     .unwrap();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if moving {
                            if let Some(pos) = prev_mouse_pos {
                                controller.mousex +=
                                    (pos.x - position.x) as f32 * MOUSE_SENSITIVITY;
                                if controller.mousex > 2. * PI {
                                    controller.mousex -= 2. * PI;
                                }
                                controller.mousey +=
                                    (pos.y - position.y) as f32 * MOUSE_SENSITIVITY;
                                controller.mousey = controller.mousey.clamp(-PI / 2., PI / 2.);
                            }
                        }
                        prev_mouse_pos = Some(*position);
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                        state.render_input.data.dim =
                            [physical_size.width as f32, physical_size.height as f32];
                        state.queue.write_buffer(
                            &state.render_input.buffer,
                            0,
                            bytemuck::cast_slice(&[state.render_input.data]),
                        );
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
