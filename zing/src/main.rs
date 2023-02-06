/* -- Zing --
If a cell is Living (White EFE9F4):
	If there are 1 or 2 sick cells, becomes Sick
	If there are 3 or 4 sick cells, becomes Metal
	If there are 5+ sick cells, becomes Dead
	If there are 3 or 4 fluid cells, cannot become Sick, Burning, or Dead
	If there are 5+ fluid cells, becomes Dead
	If there are 3 or 4 burning cells, becomes Burning
	If there are 3+ burning cells, becomes Dead
	If there are 2 or more metal cells, cannt become Sick
	If there are 5+ dead cells, becomes Sick
	If there are 4+ living cells, becomes Dead

If a cell is Sick (Green 57A773):
	If there is 1 or more fluid cells, becomes Living
	Otherwise, becomes Dead

If a cell is Fluid (Cyan 08b2e3):
	If there are 3 or 4 living cells, becomes Living
	If there are 4+ burning cells, becomes Dead

If a cell is Metal (Dark Blue 484D6D):
	If  there is 1 or more fluid, becomes Living
	If there are 2 or 3 burning cells, becomes Fluid
	If there are 3 or 4 fluid cells, cannot become Fluid
	If there are 1 or 2 sick cells, becomes Living

If a cell is Burning (Orange EE6352):
	If there are 1-3 fluid cells, becomes Living
	If there are 2 or more fluid cells, becomes Fluid
	Otherwise, becomes Dead

If a cell is Dead (Gray 444444):
	If there are ONLY two living cells, becomes Living
	If there are 4 burning and 4 fluid cells, becomes Living
	If there is 1 burning, becomes Burning
*/

use crow::{
    glutin::{
        dpi::LogicalSize,
        event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    target::Scaled,
    Context, DrawConfig, Texture,
};
use rand::prelude::*;
use rand::Rng;
use std::{thread, time};

const DEFAULT_SPEED: u64 = 20; // ms

const WINDOW_WIDTH: u32 = 1080;
const WINDOW_HEIGHT: u32 = 720;
const CELL_SIZE: u32 = 10;

const X_SIZE: usize = (WINDOW_WIDTH / CELL_SIZE) as usize;
const Y_SIZE: usize = (WINDOW_HEIGHT / CELL_SIZE) as usize;

fn mat((r, g, b): (u8, u8, u8)) -> [[f32; 4]; 4] {
	let r = r as f32 / 0xFF as f32;
	let g = g as f32 / 0xFF as f32;
	let b = b as f32 / 0xFF as f32;
    [
        [r, 0.0, 0.0, 0.0],
        [0.0, g, 0.0, 0.0],
        [0.0, 0.0, b, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
	// Living
	Living,
	Metal,
	Fluid,

	// Dying/Dead
	Dead,
	Burning,
	Sick,
}

impl Cell {
	pub fn random() -> Cell {
		let mut rng = rand::thread_rng();
		match rng.gen_range(0..6) {
			0 => Cell::Living,
			1 => Cell::Sick,
			2 => Cell::Fluid,
			3 => Cell::Metal,
			4 => Cell::Burning,
			_ => Cell::Dead,
		}
	}
	pub fn get_color(self) -> (u8, u8, u8) {
		match self {
			Cell::Living => (0xEF, 0xE9, 0xF4),
			Cell::Sick => (0x57, 0xA7, 0x73),
			Cell::Fluid => (0x08, 0xb2, 0xe3),
			Cell::Metal => (0x48, 0x4D, 0x6D),
			Cell::Burning => (0xEE, 0x63, 0x52),
			Cell::Dead => (0x44, 0x44, 0x44),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct World {
	pub contents: [[Cell; Y_SIZE]; X_SIZE],
	pub deaths_fire: usize,
	pub deaths_sick: usize,
	pub deaths_drown: usize,
}

impl World {
	pub fn new() -> World {
		World {
			contents: [[Cell::Dead; Y_SIZE]; X_SIZE],
			deaths_fire: 0,
			deaths_sick: 0,
			deaths_drown:0,
		}
	}
	pub fn random() -> World {
		let mut w = World {
			contents: [[Cell::Dead; Y_SIZE]; X_SIZE],
			deaths_fire: 0,
			deaths_sick: 0,
			deaths_drown:0,
		};
		for i in w.unwrap() {
			for j in i {
				*j = Cell::random();
			}
		}
		w
	}

	pub fn unwrap(&mut self) -> &mut [[Cell; Y_SIZE]; X_SIZE] {
		return &mut self.contents;
	}
}

#[derive(Debug)]
pub struct Game {
	map: World,
	speed: u64, // steps per second
}

impl Game {
	pub fn new(speed: u64) -> Game {
		Game {
			map: World::new(),
			speed,
		}
	}

	pub fn default() -> Game {
		Game::new(DEFAULT_SPEED)
	}

	pub fn random() -> Game {
		Game {
			map: World::random(),
			..Game::default()
		}
	}

	pub fn tick(&mut self) {
		let true_map = &mut self.map;
		let mut world_map = true_map.unwrap().clone();
		let wmcl = world_map.clone();
		let world_map_truncated = &wmcl[1..wmcl.len()-1];


		let mut i = 1;
		let mut j = 1;
		for row in world_map_truncated.iter() {
			let row = &row[1..row.len()-1];
			for mut cell in row.iter() {
				let neighbourhood = vec![
						true_map.contents[j-1][i-1], true_map.contents[j-1][i], true_map.contents[j-1][i+1],
						true_map.contents[j][i-1],		/*me*/			true_map.contents[j][i+1],
						true_map.contents[j+1][i-1], true_map.contents[j+1][i], true_map.contents[j+1][i+1],
				];
				let mut living: u8 = 0;
				let mut sick: u8 = 0;
				let mut fluid: u8 = 0;
				let mut metal: u8 = 0;
				let mut burning: u8 = 0;
				let mut dead: u8 = 0;
				for neighbour in neighbourhood {
					match neighbour {
						Cell::Living => {
							living += 1;
						},
						Cell::Sick => {
							sick += 1;
						},
						Cell::Fluid => {
							fluid += 1;
						},
						Cell::Metal => {
							metal += 1;
						},
						Cell::Burning => {
							burning += 1;
						},
						Cell::Dead => {
							dead += 1;
						}
					};
				}
				match cell {
					Cell::Living => {
						if fluid != 3 && fluid != 4 {
							if burning >= 1 {
								world_map[j][i] = Cell::Burning;
							} else if burning >= 3 {
								world_map[j][i] = Cell::Dead;
								true_map.deaths_fire += 1;
							} else if sick == 1 || sick == 2 {
								if metal < 2 {
									world_map[j][i] = Cell::Sick;
								}
							} else if sick == 3 || sick == 4 {
								world_map[j][i] = Cell::Metal;
							} else if sick >= 5 {
								world_map[j][i] = Cell::Dead;
								true_map.deaths_sick +=1 ;
							} else if fluid >= 5 {
								world_map[j][i] = Cell::Dead;
								true_map.deaths_drown += 1;
							} else if dead >= 5 {
								if metal < 2 {
									world_map[j][i] = Cell::Sick;
								}
							} else if living >= 4 {
								world_map[j][i] = Cell::Dead;
							}
						}
					},
					Cell::Sick => {
						if fluid >= 1 {
							world_map[j][i] = Cell::Living;
						} else {
							world_map[j][i] = Cell::Dead;
							true_map.deaths_sick += 1;
						}
					},
					Cell::Fluid => {
						if living == 3 || living == 4 {
							world_map[j][i] = Cell::Living;
						} else if burning >= 4 {
							world_map[j][i] = Cell::Dead;
						}
					},
					Cell::Metal => {
						if (fluid != 3 && fluid != 4) && (burning == 2 || burning == 3) {
							world_map[j][i] = Cell::Fluid;
						} else if fluid >= 1 {
							world_map[j][i] = Cell::Living;
						} else if sick == 1 || sick == 2 {
							world_map[j][i] = Cell::Living;
						}
					},
					Cell::Burning => {
						if fluid >= 1 && fluid <= 3 {
							world_map[j][i] = Cell::Living;
						} else if fluid >= 2 {
							world_map[j][i] = Cell::Fluid;
						} else {
							world_map[j][i] = Cell::Dead;
						}
					},
					Cell::Dead => {
						if fluid == 4 && burning == 4 {
							world_map[j][i] = Cell::Living;
						} else if living == 2 && sick == 0 && fluid == 0 && metal == 0 && burning == 0 {
							world_map[j][i] = Cell::Living;
						}
					}
				};
				i += 1;
			}
			i = 1;
			j += 1;
		}
		true_map.contents = world_map;
	}
		
}

fn main() -> Result<(), crow::Error> {
	let mut game = Game::random();

	let event_loop = EventLoop::new();
    let mut ctx = Context::new(
        WindowBuilder::new().with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
        &event_loop,
    )?;

    let mut texture = Texture::new(&mut ctx, (1, 1))?;
    ctx.clear_color(&mut texture, (1.0, 1.0, 1.0, 1.0));

    let mut mouse_position = (0, 0);

    event_loop.run(
        move |event: Event<()>, _window_target: _, control_flow: &mut ControlFlow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => mouse_position = position.into(),
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed
                        && input.virtual_keycode == Some(VirtualKeyCode::Space)
                    {
                        game.tick();
                        println!("Deaths by Fire: {:?}\nDeaths by Sickness: {:?}\nDeaths by Drowning: {:?}", 
					    	game.map.deaths_fire,
					    	game.map.deaths_sick,
					    	game.map.deaths_drown
					    );
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    let (x, y) = (
                        mouse_position.0 / CELL_SIZE as i32,
                        mouse_position.1 / CELL_SIZE as i32,
                    );
                    if let Some(cell) = game.map.unwrap()
                        .get_mut(x as usize)
                        .and_then(|row| row.get_mut(y as usize))
                    {
                        match cell {
                        	Cell::Living => {*cell = Cell::Sick;},
                        	Cell::Sick => {*cell = Cell::Fluid;},
                        	Cell::Fluid => {*cell = Cell::Metal;},
                        	Cell::Metal => {*cell = Cell::Burning;},
                        	Cell::Burning => {*cell = Cell::Dead;},
                        	Cell::Dead => {*cell = Cell::Living;},
                        };
                    }
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Right,
                    ..
                } => {
                    let (x, y) = (
                        mouse_position.0 / CELL_SIZE as i32,
                        mouse_position.1 / CELL_SIZE as i32,
                    );
                    for i in -1..2 {
                    	for j in -1..2 {
                    		if let Some(cell) = &mut game.map.unwrap()
			                	.get_mut((x+i) as usize)
			                    .and_then(|row| row.get_mut((y+j) as usize))
			                {
			                    **cell = Cell::random();
			                }
                    	}
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => ctx.window().request_redraw(),
            Event::RedrawRequested(_) => {

            	game.tick();
            	thread::sleep(time::Duration::from_millis(game.speed));

                let mut surface = Scaled::new(ctx.surface(), (CELL_SIZE, CELL_SIZE));

                ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0));

                for (x, row) in game.map.unwrap().iter().enumerate() {
                    for (y, &cell) in row.iter().enumerate() {
                        let color_modulation = mat(cell.get_color());

	                    ctx.draw(
	                        &mut surface,
	                        &texture,
	                        (x as i32, (row.len() - 1 - y) as i32),
	                        &DrawConfig {
	                            color_modulation,
	                            ..Default::default()
	                        },
	                    );
                    }
                }
                ctx.present(surface.into_inner()).unwrap();
            },
            _ => (),
        },
    )
}