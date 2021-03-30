use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::time::{Duration, Instant};

const TILE_SIZE: u16 = 32;
const GRID_WIDTH: u16 = 12;
const GRID_HEIGHT: u16 = 20;
const TICK_LENGTH: u128 = 650;
const TICK_SPEEDUP: u128 = 50;
const SPEEDUP_LIMIT: usize = 20; // After how many ticks speedup is applied
const SCREEN_WIDTH: u32 = TILE_SIZE as u32 * GRID_WIDTH as u32;
const SCREEN_HEIGHT: u32 = TILE_SIZE as u32 * GRID_HEIGHT as u32;

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let mut time = Instant::now();
    let mut tick_counter = 0;
    let mut speedup = 0;
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new().with_title("Rustetris")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .with_max_inner_size(size)
        .build(&event_loop)
        .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap()
    };

    let mut grid = vec![vec![0u16; GRID_HEIGHT as usize]; GRID_WIDTH as usize];

    // for testing grid only.
    /*
        grid[5][5] = 1;
        grid[10][10] = 1;
        grid[0][0] = 1;
        grid[0][2] = 1;
        grid[0][4] = 1;
        grid[0][6] = 1;
        grid[0][8] = 1;
        grid[0][10] = 1;
        grid[0][12] = 1;
        grid[0][14] = 1;
        grid[0][16] = 1;
        grid[0][18] = 1;
    */

    let mut piece = create_piece(0);
    let mut at_bottom = false;

    // *** Main loop ***
    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            draw_grid(&grid, pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| panic!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }


        if at_bottom {
            println!("created piece");
            piece = create_piece(1);
            at_bottom = false;
        }

        piece.old_tiles.iter().map(|tile| grid[tile.0 as usize][tile.1 as usize] = 0).count();
        piece.old_tiles = vec![];
        for tile in &piece.tiles {
            piece.old_tiles.push((tile.0 + piece.x, tile.1 + piece.y));
        }

        // Load piece into grid
        for tile in &piece.tiles {
            grid[(tile.0 + piece.x) as usize][(tile.1 + piece.y) as usize] = 1;
        }

        /* 
        let mut reserved_tiles = Vec::new();
        
        // Load location of pieces into separate grid without current piece
        for piece in &pieces[1..] {
            for tile in &piece.tiles {
                reserved_tiles.push(((tile.0 + piece.x) as i16, (tile.1 + piece.y) as i16));
            }
        }*/

        if input.update(&event) {
            let mut piece_moved = false;
            let mut old_tiles = piece.tiles.clone();
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::Up) {
                piece_moved = piece.rotate(true, &grid);
            }
            if input.key_pressed(VirtualKeyCode::Down) {
                // piece_moved = piece.rotate(false, &grid);
                while piece.try_relocate(0, 1, &grid) {
                    
                }

            }
            if input.key_pressed(VirtualKeyCode::Left) {
                piece_moved =  piece.try_relocate(-1, 0, &grid);
            }
            if input.key_pressed(VirtualKeyCode::Right) {
                piece_moved = piece.try_relocate(1, 0, &grid);
            }
            let now = Instant::now();
            let dt = now.duration_since(time);

            // If tick has passed, move current piece downwards and check if it stopped
            
            let mut time_limit = TICK_LENGTH - speedup as u128;
            if time_limit < 100 {
                time_limit = 100;
            }
            if dt.as_millis() > time_limit {
                at_bottom = !piece.try_relocate(0, 1, &grid);
                if at_bottom {piece_moved = false;}
                time = Instant::now();
                tick_counter += 1;
                if tick_counter > SPEEDUP_LIMIT {
                    tick_counter = 0;
                    speedup += TICK_SPEEDUP;
                }
            }
            /*if piece_moved {
                for tile in &piece.old_tiles {
                    grid[tile.0 as usize][tile.1 as usize] = 0;
                }
                
            }*/

            if at_bottom {
                let mut full_lines: Vec<usize> = Vec::new();
                // Check if there is a full line that needs demolishing and move everything downwards
                for row in 0..GRID_HEIGHT {
                    let mut is_full = true;
                    for col in 0..GRID_WIDTH {
                        if grid[col as usize][row as usize] == 0 {
                            is_full = false;
                            break;
                        }
                    }
                    if is_full {
                        full_lines.push(row as usize);
                    }
                }

                // Demolish each full line and move tiles above it downwards
                for row in full_lines {
                    // First, empty row
                    for col in 0..GRID_WIDTH {
                        grid[col as usize][row as usize] = 0;
                    }
                    println!("moving line at {}", row);
                    // Then, move all rows above it down by 1
                    for row_to_move in (1..=row).rev() {
                        for col in (0..GRID_WIDTH).rev() {
                            println!("row {} col {}", row, col);
                            let tile_to_move = grid[col as usize][(row_to_move - 1) as usize];
                            grid[col as usize][row_to_move as usize] = tile_to_move;
                            grid[col as usize][(row_to_move - 1) as usize] = 0;
                        }
                    }
                }
            }
            window.request_redraw();
        }
    });



}

/// Draws the game grid
pub fn draw_grid(grid: &Vec<Vec<u16>>, frame: &mut [u8]) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let tile = get_tile(i);
        let mut color: usize = 2;
        if tile.0 <= GRID_WIDTH as usize && tile.1 <= GRID_HEIGHT as usize {
            if grid[tile.0][tile.1] > 0 {
                color = 1;
            } else {
                color = 0;
            }
        }
        let rgba = if color == 1 {
            [0x5e, 0x48, 0xe8, 0xff]
        } else if color == 0 {
            [0x48, 0xb2, 0xe8, 0xff]
        } else {
            [0x00, 0x00, 0x00, 0x00]
        };

        pixel.copy_from_slice(&rgba);
    }
}

/// Get values (0 - GRID_WIDTH, 0 - GRID_HEIGHT)
/// 
/// If outside, get -1
pub fn get_tile(pixel: usize) -> (usize, usize) {
    let screen_x = pixel % SCREEN_WIDTH as usize;
    let screen_y = pixel / SCREEN_WIDTH as usize;
    let grid_x = screen_x / TILE_SIZE as usize;
    let grid_y = screen_y / TILE_SIZE as usize;

    (grid_x, grid_y)
}

pub struct Piece {
    pub tiles: Vec<(i16, i16)>,
    pub orientation: usize, // how many times it is rotated (to right)
    pub color: usize,
    x: i16,
    y: i16,
    pub old_tiles: Vec<(i16, i16)>, // these include x and y. current tiles dont
}

impl Piece {

    /// Move the piece
    fn relocate(&mut self, dx: i16, dy: i16) {
        /*self.tiles
        .iter_mut()
        .map(|item| *item = (item.0 + dx, item.1 + dy))
        .count();*/
        self.x += dx;
        self.y += dy;
    }

    /// Returns false if could not relocate due to going out
    /// of boundaries
    fn try_relocate(&mut self, dx: i16, dy: i16, grid: &Vec<Vec<u16>>) -> bool {
        self.relocate(dx, dy);
        let mut old_tiles_backup = self.old_tiles.clone();

        if !self.is_in_boundaries()
        || self.overlaps(grid) {
            self.relocate(-dx, -dy);
            return false;
        }
        
        self.old_tiles = old_tiles_backup.clone();
        
        true
    }

    /// Rotate piece to right or left
    /// Checks first if rotation is possible,
    /// if not, returns false
    fn rotate(&mut self, to_right: bool, grid: &Vec<Vec<u16>>) -> bool {
        self.rotate_tiles(to_right);
        let mut old_tiles_backup  = self.old_tiles.clone();
        // Undo rotation if tiles are not inside game grid
        // and inform function caller
        if !self.is_in_boundaries() {
            self.rotate_tiles(!to_right);
            return false;
        }
        
        if self.overlaps(grid) {
            self.rotate_tiles(!to_right);
            return false;
        }


        // Set orientation variable
        if to_right {
            if self.orientation < 3 {
                self.orientation += 1;
            }
            self.orientation = 0;
        } else {
            if self.orientation > 0 {
                self.orientation -= 1;
            }
            self.orientation = 3;
        }
        self.old_tiles = old_tiles_backup.clone();
        true
    }

    fn rotate_tiles(&mut self, to_right: bool) {
        for tile in &mut self.tiles {
            let new_tile = if to_right { 
                (tile.1, -tile.0) 
            } else { 
                    (-tile.1, tile.0) };
            *tile = new_tile;
        }
    }

    fn is_in_boundaries(&mut self) -> bool {
        for tile in &self.tiles {
            let x = self.x + tile.0;
            let y = self.y + tile.1;
            if x < 0 || 
                x >= GRID_WIDTH as i16 || 
                y < 0 || y >= GRID_HEIGHT as i16 {
                return false;
            }
        }
        true
    }

    fn overlaps(&self, grid: &Vec<Vec<u16>>) -> bool {
        for tile in &self.tiles {

            let mut skip = false;

            for old_tile in &self.old_tiles {
                if old_tile.0 == (tile.0 + self.x) as i16 && old_tile.1 == (tile.1 + self.y) as i16 {
                    skip = true;
                    break;
                }
            }
            if skip { continue; }
            if grid[(tile.0 + self.x) as usize][(tile.1 + self.y) as usize] > 0 {
                return true;
            }
        }
        
        false
    }

}

pub fn create_piece(piece_type: u16) -> Piece {
    let mut tiles = match piece_type {
        0 => vec![(0, 2), (0, -1), (0, 1), (0, 0)],
        1 => vec![(0, 0), (0, -1), (0, 1), (1, 1)],
        1 => vec![(0, 0), (0, -1), (0, 1), (1, 1)],
        _ => panic!("Create piece panicked"),
    };
    

    Piece {
        tiles,
        orientation: 0,
        color: 0,
        x: 5,
        y: 3,
        old_tiles: vec![],
    }
}
