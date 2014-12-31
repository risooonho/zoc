// See LICENSE file for copyright and license details.

extern crate glutin;
extern crate cgmath;
extern crate serialize;

use cgmath::{Vector2, Vector3};
use core_types::{Size2, ZInt};
use visualizer_types::{ZFloat, Color3, Color4, ColorId, MatId, WorldPos, ScreenPos};
use zgl::{Zgl};
use camera::Camera;
use shader::{Shader};

// TODO: remove 'gl'
use gl;

static VS_SRC: &'static str = "\
    #version 100\n\
    uniform mat4 mvp_mat;\n\
    attribute vec2 position;\n\
    void main() {\n\
        gl_Position = mvp_mat * vec4(position, 0.0, 1.0);\n\
    }\n\
";

static FS_SRC: &'static str = "\
    #version 100\n\
    precision mediump float;
    uniform vec4 col;\n\
    void main() {\n\
        gl_FragColor = col;\n\
    }\n\
";

fn get_win_size(window: &glutin::Window) -> Size2<ZInt> {
    let (w, h) = window.get_inner_size().expect("Can`t get window size");
    Size2{w: w as ZInt, h: h as ZInt}
}

// TODO: use this
// fn get_max_camera_pos(map_size: &Size2<ZInt>) -> WorldPos {
//     let pos = geom::map_pos_to_world_pos(
//         MapPos{v: Vector2{x: map_size.w, y: map_size.h}});
//     WorldPos{v: Vector3{x: -pos.v.x, y: -pos.v.y, z: 0.0}}
// }

fn get_max_camera_pos() -> WorldPos {
    WorldPos{v: Vector3{x: -2.0, y: -2.0, z: 0.0}}
}

fn print_gl_info(zgl: &Zgl) {
    println!("GL_VERSION: {}", zgl.get_info(gl::VERSION));
    println!("GL_SHADING_LANGUAGE_VERSION: {}", zgl.get_info(gl::SHADING_LANGUAGE_VERSION));
    println!("GL_VENDOR: {}", zgl.get_info(gl::VENDOR));
    println!("GL_RENDERER: {}", zgl.get_info(gl::RENDERER));
    // println!("GL_EXTENSIONS: {}", zgl.get_info(gl::EXTENSIONS));
}

pub struct Visualizer {
    zgl: Zgl,
    window: glutin::Window,
    should_close: bool,
    color_counter: i32, // TODO: remove
    test_color: Color4,
    shader: Shader,
    color_uniform_location: ColorId,
    mvp_uniform_location: MatId,
    camera: Camera,
    mouse_pos: ScreenPos,
    is_mouse_lmb_pressed: bool, // TODO: rename: is_lmb_pressed
    win_size: Size2<ZInt>,
}

impl Visualizer {
    pub fn new() -> Visualizer {
        let window_builder = glutin::WindowBuilder::new().with_gl_version((2, 0));
        let window = window_builder.build().ok().expect("Can`t create window");
        unsafe {
            window.make_current();
        };
        let win_size = get_win_size(&window);
        let mut zgl = Zgl::new(|s| window.get_proc_address(s));
        print_gl_info(&zgl);
        let shader = Shader::new(&zgl, VS_SRC, FS_SRC);
        let color_uniform_location = shader.get_uniform_color(&zgl, "col");
        let mvp_uniform_location = shader.get_uniform_mat(&zgl, "mvp_mat");
        zgl.set_clear_color(Color3{r: 0.0, g: 0.0, b: 0.4});
        let mut camera = Camera::new(&win_size);
        camera.set_max_pos(get_max_camera_pos());
        Visualizer {
            zgl: zgl,
            window: window,
            should_close: false,
            color_counter: 0,
            test_color: Color4{r: 0.0, g: 0.0, b: 0.0, a: 0.0},
            shader: shader,
            color_uniform_location: color_uniform_location,
            mvp_uniform_location: mvp_uniform_location,
            camera: camera,
            mouse_pos: ScreenPos{v: Vector2::from_value(0)},
            is_mouse_lmb_pressed: false,
            win_size: win_size,
        }
    }

    pub fn is_running(&self) -> bool {
        !self.should_close
    }

    fn handle_event(&mut self, event: &glutin::Event) {
        match *event {
            glutin::Event::Closed => {
                self.should_close = true;
            },
            glutin::Event::Resized(w, h) => {
                self.win_size = Size2{w: w as ZInt, h: h as ZInt};
                self.zgl.set_viewport(&self.win_size);
            },
            glutin::Event::MouseMoved((x, y)) => {
                let new_pos = ScreenPos{v: Vector2{x: x as ZInt, y: y as ZInt}};
                if self.is_mouse_lmb_pressed {
                    let diff = new_pos.v - self.mouse_pos.v;
                    let win_w = self.win_size.w as ZFloat;
                    let win_h = self.win_size.h as ZFloat;
                    self.camera.add_z_angle(diff.x as ZFloat * (360.0 / win_w));
                    self.camera.add_x_angle(diff.y as ZFloat * (360.0 / win_h));
                }
                self.mouse_pos = new_pos;
            },
            glutin::Event::MouseInput(
                glutin::ElementState::Pressed,
                glutin::MouseButton::LeftMouseButton,
            ) => {
                self.is_mouse_lmb_pressed = true;
            },
            glutin::Event::MouseInput(
                glutin::ElementState::Released,
                glutin::MouseButton::LeftMouseButton,
            ) => {
                self.is_mouse_lmb_pressed = false;
                let (r, g, b, a) = self.zgl.read_pixel_bytes(
                    &self.win_size, &self.mouse_pos);
                println!("r: {}, g: {}, b: {}, a: {}", r, g, b, a);
            },
            glutin::Event::KeyboardInput(_, _, Some(key)) => match key {
                glutin::VirtualKeyCode::Q | glutin::VirtualKeyCode::Escape => {
                    self.should_close = true;
                },
                glutin::VirtualKeyCode::W => self.camera.move_camera(270.0, 0.1),
                glutin::VirtualKeyCode::S => self.camera.move_camera(90.0, 0.1),
                glutin::VirtualKeyCode::D => self.camera.move_camera(0.0, 0.1),
                glutin::VirtualKeyCode::A => self.camera.move_camera(180.0, 0.1),
                glutin::VirtualKeyCode::Minus => self.camera.change_zoom(1.3),
                glutin::VirtualKeyCode::Equals => self.camera.change_zoom(0.7),
                _ => {},
            },
            _ => {},
        }
    }

    fn handle_events(&mut self) {
        let events = self.window.poll_events().collect::<Vec<_>>();
        if events.is_empty() {
            return;
        }
        println!("{}", events);
        for event in events.iter() {
            self.handle_event(event);
        }
    }

    fn update_test_color(&mut self) {
        self.test_color = match self.color_counter {
            0 => Color4{r: 1.0, g: 0.0, b: 0.0, a: 1.0},
            30 => Color4{r: 0.0, g: 1.0, b: 0.0, a: 1.0},
            60 => Color4{r: 0.0, g: 0.0, b: 1.0, a: 1.0},
            _ => self.test_color,
        };
        self.color_counter += 1;
        if self.color_counter > 90 {
            self.color_counter = -1;
        }
    }

    fn draw(&mut self) {
        self.zgl.clear_screen();
        let vertices: [ZFloat, ..3 * 3] = [
            0.0,  0.5, 0.0,
            0.5, -0.5, 0.0,
            -0.5, -0.5, 0.0,
        ];
        self.shader.activate(&self.zgl);
        self.shader.set_uniform_color(
            &self.zgl, self.color_uniform_location, &self.test_color);
        self.shader.set_uniform_mat4f(
            &self.zgl, self.mvp_uniform_location, &self.camera.mat(&self.zgl));
        self.zgl.draw_vertices(&vertices, 3);
        self.window.swap_buffers();
    }

    pub fn tick(&mut self) {
        self.handle_events();
        // self.logic();
        self.update_test_color();
        self.draw();
        // self.update_time();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
