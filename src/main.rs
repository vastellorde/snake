extern crate sdl2;

use std::ops::Add;
use std::time::Duration;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;

const GRID_X_SIZE: u32 = 40;
const GRID_Y_SIZE: u32 = 30;
const DOT_SIZE_IN_PXS: u32 = 20;

pub enum GameState { Playing, Paused }
pub enum PlayerDirection { Up, Down, Right, Left }
#[derive(Copy, Clone)]
pub struct Point(pub i32, pub i32);

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl PartialEq<Point> for Point {
    fn eq(&self, other: &Point) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

pub struct GameContext {
    pub player_position: Vec<Point>,
    pub player_direction: PlayerDirection,
    pub food: Point,
    pub state: GameState,
}

impl GameContext {
    fn new() -> Self {
        GameContext {
            player_position: vec![Point(3, 1), Point(2, 1), Point(1, 1)],
            player_direction: PlayerDirection::Right,
            state: GameState::Paused,
            food: Point(rand::thread_rng().gen_range(0..GRID_X_SIZE) as i32, rand::thread_rng().gen_range(0..GRID_Y_SIZE) as i32),
        }
    }

    pub fn next_tick(&mut self) {
        if let GameState::Paused = self.state {
            return;
        }
        let head_position = self.player_position.first().unwrap();
        let body_position = self.player_position.get(1).unwrap();
        let next_head_position = match self.player_direction {
            PlayerDirection::Up => {
                let mut new_head = *head_position + Point(0, -1);
                if new_head.1 == -1 {
                    new_head.1 = GRID_Y_SIZE as i32;
                }
                new_head
            },
            PlayerDirection::Down => {
                let mut new_head = *head_position + Point(0, 1);
                if new_head.1 == (GRID_Y_SIZE + 1) as i32 {
                    new_head.1 = 0;
                }
                new_head
            },
            PlayerDirection::Right => {
                let mut new_head = *head_position + Point(1, 0);
                if new_head.0 == GRID_X_SIZE as i32 {
                    new_head.0 = 0;
                }
                new_head
            },
            PlayerDirection::Left => {
                let mut new_head = *head_position + Point(-1, 0);
                if new_head.0 == -1 {
                    new_head.0 = GRID_X_SIZE as i32;
                }
                new_head
            },
        };
        if next_head_position == *body_position {
            return;
        }

        for pos in self.player_position.clone().into_iter() {
            if pos == next_head_position {
                self.player_position = vec![Point(3, 1), Point(2, 1), Point(1, 1)];
                return;
            }
        }

        self.player_position.pop();
        self.player_position.reverse();
        self.player_position.push(next_head_position);
        self.player_position.reverse();

        if self.food == next_head_position {
            let fairy_position = self.player_position.last().unwrap();
            self.food = Point(rand::thread_rng().gen_range(0..GRID_X_SIZE) as i32, rand::thread_rng().gen_range(0..GRID_Y_SIZE) as i32);

            let next_fairy_position = match self.player_direction {
                PlayerDirection::Up => *fairy_position + Point(0, 1),
                PlayerDirection::Down => *fairy_position + Point(0, -1),
                PlayerDirection::Right => *fairy_position + Point(-1, 0),
                PlayerDirection::Left => *fairy_position + Point(1, 0),
            };
            self.player_position.push(next_fairy_position);
        }
    }

    pub fn move_up(&mut self) {
        self.player_direction = PlayerDirection::Up;
    }

    pub fn move_down(&mut self) {
        self.player_direction = PlayerDirection::Down;
    }

    pub fn move_right(&mut self) {
        self.player_direction = PlayerDirection::Right;
    }

    pub fn move_left(&mut self) {
        self.player_direction = PlayerDirection::Left;
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing
        }
    }
}

pub struct Renderer { canvas: WindowCanvas }

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer { canvas })
    }

    fn draw_dot(&mut self, point: &Point) -> Result<(), String> {
        let Point(x, y) = point;
        self.canvas.fill_rect(Rect::new(
            x * DOT_SIZE_IN_PXS as i32,
            y * DOT_SIZE_IN_PXS as i32,
            DOT_SIZE_IN_PXS,
            DOT_SIZE_IN_PXS,
        ))?;

        Ok(())
    }

    pub fn draw(&mut self, context: &GameContext) -> Result<(), String> {
        self.draw_background(context);
        self.draw_player(context)?;
        self.draw_food(context)?;
        self.canvas.present();

        Ok(())
    }

    fn draw_background(&mut self, context: &GameContext) {
        let color = match context.state {
            GameState::Playing => Color::RGB(0, 0, 0),
            GameState::Paused => Color::RGB(30, 30, 30),
        };
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }

    fn draw_player(&mut self, context: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::GREEN);
        for point in &context.player_position {
            self.draw_dot(point)?;
        }

        Ok(())
    }

    fn draw_food(&mut self, context: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RED);
        self.draw_dot(&context.food)?;
        Ok(())
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context
        .video()
        .unwrap();

    let window = video_system
        .window(
            "Snake game",
            GRID_X_SIZE * DOT_SIZE_IN_PXS,
            GRID_Y_SIZE * DOT_SIZE_IN_PXS)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut context = GameContext::new();

    let mut renderer = Renderer::new(window).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut frame_counter = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Up => context.move_up(),
                        Keycode::Left => context.move_left(),
                        Keycode::Down => context.move_down(),
                        Keycode::Right => context.move_right(),
                        Keycode::Escape => context.toggle_pause(),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        frame_counter += 1;
        if frame_counter % 10 == 0 {
            frame_counter = 0;
            context.next_tick();
        }
        renderer.draw(&context).unwrap();
    }
}
