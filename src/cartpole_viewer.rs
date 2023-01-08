extern crate glutin_window;
extern crate graphics;
extern crate mem_macros;
extern crate opengl_graphics;
extern crate piston;

use bus::BusReader;
use glutin_window::GlutinWindow as Window;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use std::time::Duration;

pub struct CartPoleViewer {
    gl: GlGraphics, // OpenGL drawing backend
    window: Window,
    state_reader: BusReader<[f32; 4]>,
    translation: f64,  // For cart
    rotation: f64,  // For pole
}

impl CartPoleViewer {
    pub fn new(state_reader: BusReader<[f32; 4]>) -> CartPoleViewer {
        let opengl = OpenGL::V3_2;

        // Create a Glutin window.
        let window: Window = WindowSettings::new("spinning-square", [1024, 512])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        CartPoleViewer {
            gl: GlGraphics::new(opengl),
            window,
            rotation: 0.0,
            translation: 0.0,
            state_reader,
        }
    }
    fn render(&mut self, args: &RenderArgs) {
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLUE_GRAY: [f32; 4] = [0.4, 0.6, 0.8, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        // Dimensions are halves
        const CART_H: f64 = 20.0;
        const CART_W: f64 = 50.0;
        const POLE_L: f64 = 50.0;
        const POLE_W: f64 = 4.0;

        // Create objects
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        let cart_wagon = rectangle::centered([0.0, 0.0, CART_W, CART_H]);
        let cart_pole = rectangle::centered([0.0, 0.0, POLE_W, POLE_L]);
        let line = rectangle::centered([0.0, 0.0, 1024.0, 1.0]);

        // Render objects in viewport
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);

            // Center of screen tranform
            let transform_default = c.transform.trans(x, y);

            // Cart tranform
            let transform_cart_wagon = c
                .transform
                .trans(x, y - CART_H)
                .trans(self.translation as f64, 0.0);

            // Pole tranform from cart tranform
            let transform_cart_pole = transform_cart_wagon
                .trans(0.0, -CART_H + POLE_W) // translate pole center up on cart
                .rot_rad(self.rotation) //rotate it around its center
                .trans(0.0, -POLE_L); // translate it up to its bottom
            
            // Draw
            rectangle(BLACK, line, transform_default, gl);
            rectangle(BLUE_GRAY, cart_wagon, transform_cart_wagon, gl);
            rectangle(BLACK, cart_pole, transform_cart_pole, gl);
        });
    }
    fn update_states(&mut self, args: &UpdateArgs) {
        match self
            .state_reader
            .recv_timeout(Duration::from_millis((args.dt * 1000.0) as u64))
        {
            Ok(states) => {
                let [x, _, theta, _]: [f32; 4] = states.clone().try_into().ok().unwrap();
                self.translation = (x * 100.0) as f64; // NOTE: mult is since units seem inconsistent with Piston's coordinate system
                self.rotation = theta as f64;
            }
            Err(_) => {
                return;
            }
        }
    }
    pub fn spin_renderer(&mut self) {
        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut self.window) {
            if let Some(args) = e.render_args() {
                self.render(&args);
            }

            if let Some(args) = e.update_args() {
                self.update_states(&args);
            }
        }
    }
}
