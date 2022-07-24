extern crate sdl2;

use bevy_ecs::prelude::*;
use bevy_ecs::world::World;
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use simulation::Simulation;
use simulation::{Position, PreviousPosition};
use std::time::Duration;

use sdl2::render::Canvas;
use sdl2::video::Window;

mod simulation;

const SCREEN_WIDTH: u32 = 1440;
const SCREEN_HEIGHT: u32 = 900;
const SIMULATION_STEP: i16 = 1;

pub fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGBA8888, 400, 300)
        .map_err(|e| e.to_string())?;

    let mut simulation = Simulation::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut event_pump = sdl.event_pump()?;

    let mut lastx = 0;
    let mut lasty = 0;

    'mainloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        break 'mainloop;
                    } else if keycode == Keycode::Space {
                        println!("space down");
                        for i in 0..400 {
                            canvas.pixel(i as i16, i as i16, 0xFF000FFu32)?;
                        }
                        canvas.present();
                    }
                }

                Event::MouseButtonDown { x, y, .. } => {
                    let color = pixels::Color::RGB(x as u8, y as u8, 255);
                    let _ = canvas.line(lastx, lasty, x as i16, y as i16, color);
                    lastx = x as i16;
                    lasty = y as i16;
                    println!("mouse btn down at ({},{})", x, y);
                    canvas.present();
                }

                _ => {}
            }
        }

        canvas
            .with_texture_canvas(&mut texture, |texture_canvas| {
                texture_canvas.clear();
                texture_canvas.set_draw_color(Color::RGBA(0, 255, 0, 255));
                texture_canvas
                    .fill_rect(Rect::new(0, 0, 400, 300))
                    .expect("could not fill rect");
            })
            .map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

        canvas.clear();

        render_nodes(&canvas, &mut simulation.world).map_err(|e| e.to_string())?;

        //let dst = Some(Rect::new(0, 0, 400, 300));
        //canvas.copy_ex(&texture, None, dst, 0.0, None, false, false)?;

        //let dst = Some(Rect::new(200 + angle, 200, 400, 300));
        //canvas.copy_ex(&texture, None, dst, 0.0, None, false, false)?;
        canvas.present();

        simulation.step(SIMULATION_STEP);

        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        std::thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}

fn render_nodes(canvas: &Canvas<Window>, world: &mut World) -> Result<(), String> {
    //println!("render");
    //canvas
    //.filled_circle(50, 50, 8, Color::WHITE)
    //.map_err(|e| e.to_string())?;

    let mut query = world.query::<&Position>();
    for pos in query.iter(&world) {
        canvas
            .filled_circle(pos.x as i16, pos.y as i16, 8, Color::WHITE)
            .expect("Could not fill circle");
    }

    //println!("after render");
    Ok(())
}
