#[macro_use]
extern crate glium;

use arcball::ArcballCamera;
use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::index::PrimitiveType;
use glium::{glutin, Surface};
use ultraviolet::{
    projection::perspective_gl,
    vec::{Vec2, Vec3},
};

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}
implement_vertex!(Vertex, pos, color);

// NOTE: This is still provided as an example of how to hook up mouse events
// to the camera, however it should eventually be replaced by one using
// device events (I think) since window events don't seem to fire often

fn main() {
    let window = glutin::window::WindowBuilder::new().with_title("Arcball Camera Cube Example");
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let mut event_loop = glutin::event_loop::EventLoop::new();

    let display =
        glium::Display::new(window, context, &event_loop).expect("failed to create display");

    // Hard-coded cube triangle strip
    let vertex_buffer = glium::VertexBuffer::new(
        &display,
        &[
            Vertex {
                pos: [1.0, 1.0, -1.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, -1.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, 1.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, 1.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                pos: [-1.0, -1.0, 1.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, -1.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                pos: [-1.0, -1.0, -1.0],
                color: [0.0, 0.0, 1.0],
            },
            Vertex {
                pos: [1.0, 1.0, -1.0],
                color: [0.0, 0.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0, -1.0],
                color: [0.0, 0.0, 1.0],
            },
            Vertex {
                pos: [1.0, 1.0, 1.0],
                color: [1.0, 1.0, 0.0],
            },
            Vertex {
                pos: [1.0, -1.0, 1.0],
                color: [1.0, 1.0, 0.0],
            },
            Vertex {
                pos: [-1.0, -1.0, 1.0],
                color: [1.0, 1.0, 0.0],
            },
            Vertex {
                pos: [1.0, -1.0, -1.0],
                color: [1.0, 0.0, 1.0],
            },
            Vertex {
                pos: [-1.0, -1.0, -1.0],
                color: [1.0, 0.0, 1.0],
            },
        ],
    )
    .unwrap();
    let index_buffer = glium::index::NoIndices(PrimitiveType::TriangleStrip);

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 proj_view;

                in vec3 pos;
                in vec3 color;

                out vec3 vcolor;

                void main(void) {
                    gl_Position = proj_view * vec4(pos, 1.0);
                    vcolor = color;
                }
            ",
            fragment: "
                #version 140

                in vec3 vcolor;
                out vec4 color;

                void main(void) {
                    color = vec4(vcolor, 1.0);
                }
            "
        },
    )
    .unwrap();

    let display_dims = display.get_framebuffer_dimensions();
    let persp_proj = perspective_gl(
        f32::to_radians(65.0),
        display_dims.0 as f32 / display_dims.1 as f32,
        1.0,
        200.0,
    );
    let mut arcball_camera = ArcballCamera::new(
        Vec3::new(0.0, 0.0, 0.0),
        1.0,
        [display_dims.0 as f32, display_dims.1 as f32],
    );

    let draw_params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };
    // Track if left/right mouse is down
    let mut mouse_pressed = [false, false, false];
    let mut prev_mouse: Option<PhysicalPosition<f64>> = None;

    let mut should_quit = false;
    while !should_quit {
        event_loop.run_return(|e, _, control_flow| {
            control_flow.set_wait();
            // println!("running");
            match e {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => should_quit = true,
                    glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                        match input.virtual_keycode {
                            Some(glutin::event::VirtualKeyCode::Escape) => should_quit = true,
                            _ => {}
                        }
                    }

                    glutin::event::WindowEvent::MouseInput { state, button, .. } => {
                        if button == glutin::event::MouseButton::Left {
                            mouse_pressed[0] = state == glutin::event::ElementState::Pressed;
                        } else if button == glutin::event::MouseButton::Right
                            || button == glutin::event::MouseButton::Middle
                        {
                            mouse_pressed[1] = state == glutin::event::ElementState::Pressed;
                        }
                    }
                    glutin::event::WindowEvent::MouseWheel { delta, .. } => {
                        let y = match delta {
                            glutin::event::MouseScrollDelta::LineDelta(_, y) => y,
                            glutin::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
                        };
                        arcball_camera.zoom(y, 0.16);
                    }

                    glutin::event::WindowEvent::CursorMoved { position, .. } => {
                        if let Some(prev) = prev_mouse {
                            if mouse_pressed[0] {
                                arcball_camera.rotate(
                                    Vec2::new(prev.x as f32, prev.y as f32),
                                    Vec2::new(position.x as f32, position.y as f32),
                                );
                            } else if mouse_pressed[1] {
                                let mouse_delta = Vec2::new(
                                    (position.x - prev.x) as f32,
                                    (position.y - prev.y) as f32,
                                );
                                arcball_camera.pan(mouse_delta);
                            }
                        }

                        prev_mouse = Some(position);
                        println!("pressed = {:?}, prev = {:?}", mouse_pressed, prev_mouse);
                    }
                    _ => {}
                },
                glutin::event::Event::MainEventsCleared => {
                    control_flow.set_exit();
                }
                _ => {}
            }
        });
        let proj_view: [[f32; 4]; 4] = (persp_proj * arcball_camera.get_mat4()).into();
        let uniforms = uniform! {
            proj_view: proj_view,
        };

        let mut target = display.draw();
        target.clear_color(0.1, 0.1, 0.1, 0.0);
        target.clear_depth(1.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &draw_params,
            )
            .unwrap();
        target.finish().unwrap();
    }
}
