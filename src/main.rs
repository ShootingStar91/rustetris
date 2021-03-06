use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const TILE_SIZE: u16 = 32;
const GRID_WIDTH: u16 = 12; // 12 normal, 16 big
const GRID_HEIGHT: u16 = 20; // 20 normal, 26 big
const SCREEN_WIDTH: u32 = TILE_SIZE as u32 * GRID_WIDTH as u32;
const SCREEN_HEIGHT: u32 = TILE_SIZE as u32 * GRID_HEIGHT as u32;

const TICK_LENGTH: i128 = 400; // Game speed at start 
const MIN_TICK_LENGTH: i128 = 220; // Smallest tick length
const TICK_SPEEDUP: u128 = 15; // How much ticks will speed up
const SPEEDUP_TIMER: usize = 25; // After how many seconds speedup is applied

const PIECE_SPAWN_Y: usize = 2;
const PIECE_SPAWN_X: usize = GRID_WIDTH as usize / 2;
const NEXT_PIECE_OFFSET_X: i16 = 1;
const NEXT_PIECE_OFFSET_Y: i16 = 2;

const COLORS: usize = 4;

// Set debug prints on or off
const DEBUG_PRINT: bool = false;

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let mut time = Instant::now();
    let mut speedup_timer = Instant::now();
    let speedup: i128 = TICK_SPEEDUP as i128;
    let mut rng = rand::thread_rng();
    let mut score = 0;
    let mut pause = false;
    let mut time_limit: i128 = TICK_LENGTH;

    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Rustetris")
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

    let mut grid = vec![vec![0i16; GRID_HEIGHT as usize]; GRID_WIDTH as usize];
    let mut shadegrid = vec![vec![0i16; GRID_HEIGHT as usize]; GRID_WIDTH as usize];

    let mut piece = create_piece(&mut rng);
    let mut next_piece = create_piece(&mut rng);
    let mut at_bottom = false;

    // *** Main loop ***
    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            for t in &next_piece.tiles {
                grid[(t.0 + NEXT_PIECE_OFFSET_X) as usize][(t.1 + NEXT_PIECE_OFFSET_Y) as usize] =
                    -2;
            }
            draw_grid(&mut grid, pixels.get_frame(), &shadegrid);
            for t in &next_piece.tiles {
                grid[(t.0 + NEXT_PIECE_OFFSET_X) as usize][(t.1 + NEXT_PIECE_OFFSET_Y) as usize] =
                    0;
            }
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
            piece = next_piece.copy();
            next_piece = create_piece(&mut rng);
            at_bottom = false;

            // Check for game over
            if piece.overlaps(&grid) {
                *control_flow = ControlFlow::Exit;
                println!("Game over! Score: {}", score);
            }
        }

        refresh_tiles(&mut piece, &mut grid, &mut shadegrid);

        if input.update(&event) {
            let mut piece_moved = false;

            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::P) || input.quit() {
                pause = !pause; 
            }            
            if !pause {
                refresh_tiles(&mut piece, &mut grid, &mut shadegrid);
                if input.key_pressed(VirtualKeyCode::Up) {
                    piece_moved = piece.rotate(true, &grid);
                }
                if input.key_pressed(VirtualKeyCode::Down) {
                    while piece.try_relocate(0, 1, &grid) {
                        refresh_tiles(&mut piece, &mut grid, &mut shadegrid);
                    }
                    at_bottom = true;
                }
                if input.key_pressed(VirtualKeyCode::Left) {
                    piece_moved = piece.try_relocate(-1, 0, &grid);
                }
                if input.key_pressed(VirtualKeyCode::Right) {
                    piece_moved = piece.try_relocate(1, 0, &grid);
                }
                let now = Instant::now();
                let dt = now.duration_since(time);
                if piece_moved {
                    refresh_tiles(&mut piece, &mut grid, &mut shadegrid);
                }

                if dt.as_millis() > time_limit as u128 {
                    at_bottom = !piece.try_relocate(0, 1, &grid);

                    time = Instant::now();
                    if DEBUG_PRINT {
                        for row in 0..GRID_HEIGHT {
                            for col in 0..GRID_WIDTH {
                                let sign = grid[col as usize][row as usize];
                                if sign == 0 {
                                    print!(" ");
                                } else {
                                    print!("#");
                                }
                            }
                            println!("");
                        }
                        println!("");
                    }
                }
            }
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
                // Add full lines to scores
                score += full_lines.len();
                // Demolish each full line and move tiles above it downwards
                for row in full_lines {
                    // First, empty row
                    for col in 0..GRID_WIDTH {
                        grid[col as usize][row as usize] = 0;
                    }
                    // Then, move all rows above it down by 1
                    for row_to_move in (1..=row).rev() {
                        for col in (0..GRID_WIDTH).rev() {
                            let tile_to_move = grid[col as usize][(row_to_move - 1) as usize];
                            grid[col as usize][row_to_move as usize] = tile_to_move;
                            grid[col as usize][(row_to_move - 1) as usize] = 0;
                        }
                    }
                }
            }
            if !pause {
                let speedup_timer_now = Instant::now();
                let speedup_dt = speedup_timer_now.duration_since(speedup_timer);
                if speedup_dt.as_secs() > SPEEDUP_TIMER as u64  {
                    if time_limit > MIN_TICK_LENGTH - speedup {
                        time_limit -= TICK_SPEEDUP as i128;
                    }
                    println!("time limit {}", time_limit);
                    speedup_timer = Instant::now();
                }
            }
            window.request_redraw();
        }
    });
}

/// Draws the game grid
pub fn draw_grid(grid: &Vec<Vec<i16>>, frame: &mut [u8], shadegrid: &Vec<Vec<i16>>) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let tile = get_tile(i);
        let mut color: i16 = 2;
        let x = tile.0 as usize;
        let y = tile.1 as usize;
        if tile.0 <= GRID_WIDTH as usize && tile.1 <= GRID_HEIGHT as usize {
            color = grid[x][y];
        }

        if shadegrid[x][y] == 1 && grid[x][y] == 0 {
            color = -1;
        }
        let rgba = match color {
            -2 => [0xde, 0xde, 0xde, 0xff], // lighter grey
            -1 => [0xde, 0xde, 0xde, 0xff], // grey
            0 => [0xfd, 0xfd, 0xfd, 0xff],  // 253, 237, 248
            1 => [0x32, 0xb9, 0x13, 0xff],  // 50 185 19
            2 => [0x13, 0x32, 0xb9, 0xff],  // 19, 50, 185
            3 => [0xb9, 0x48, 0x13, 0xff],  // 185, 94, 19
            4 => [0xb9, 0x13, 0x82, 0xff],  // 185, 19, 130
            _ => panic!("Invalid color value in draw grid"),
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
    fn copy(&self) -> Piece {
        Piece {
            tiles: self.tiles.clone(),
            orientation: self.orientation,
            color: self.color,
            x: self.x,
            y: self.y,
            old_tiles: self.old_tiles.clone(),
        }
    }
    /// Move the piece
    fn relocate(&mut self, dx: i16, dy: i16) {
        self.x += dx;
        self.y += dy;
    }

    /// Returns false if could not relocate due to going out
    /// of boundaries
    fn try_relocate(&mut self, dx: i16, dy: i16, grid: &Vec<Vec<i16>>) -> bool {
        self.relocate(dx, dy);
        let old_tiles_backup = self.old_tiles.clone();

        if !self.is_in_boundaries() || self.overlaps(grid) {
            self.relocate(-dx, -dy);
            return false;
        }

        self.old_tiles = old_tiles_backup.clone();

        true
    }

    /// Rotate piece to right or left
    /// Checks first if rotation is possible,
    /// if not, returns false
    fn rotate(&mut self, to_right: bool, grid: &Vec<Vec<i16>>) -> bool {
        self.rotate_tiles(to_right);
        let old_tiles_backup = self.old_tiles.clone();
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
                (-tile.1, tile.0)
            };
            *tile = new_tile;
        }
    }

    fn is_in_boundaries(&mut self) -> bool {
        for tile in &self.tiles {
            let x = self.x + tile.0;
            let y = self.y + tile.1;
            if x < 0 || x >= GRID_WIDTH as i16 || y < 0 || y >= GRID_HEIGHT as i16 {
                return false;
            }
        }
        true
    }

    fn overlaps(&self, grid: &Vec<Vec<i16>>) -> bool {
        for tile in &self.tiles {
            let mut skip = false;

            for old_tile in &self.old_tiles {
                if old_tile.0 == (tile.0 + self.x) as i16 && old_tile.1 == (tile.1 + self.y) as i16
                {
                    skip = true;
                    break;
                }
            }
            if skip {
                continue;
            }
            if grid[(tile.0 + self.x) as usize][(tile.1 + self.y) as usize] > 0 {
                return true;
            }
        }

        false
    }
}

pub fn create_piece(rng: &mut rand::rngs::ThreadRng) -> Piece {
    let piece_type = rng.gen_range(0..=470);
    let tiles = match piece_type {
        0..=100 => vec![(0, 2), (0, -1), (0, 1), (0, 0)],     // Stick piece
        101..=170 => vec![(0, 0), (0, -1), (0, 1), (1, 1)],   // L piece
        171..=240 => vec![(0, 0), (0, -1), (0, 1), (-1, 1)],  // L invert
        241..=280 => vec![(-1, 0), (0, 0), (0, 1), (1, 1)],   // zigzag
        281..=320 => vec![(-1, 1), (0, 0), (0, 1), (1, 0)],   // zigzag invert
        321..=380 =>  vec![(0, -1), (0, 0), (0, 1), (1, 0)],  // "fork"
        381..=470 => vec![(0, 0), (0, 1), (1, 1), (1, 0)],    // block
        _ => panic!("Create piece panicked"),
    };

    Piece {
        tiles,
        orientation: 0,
        color: rng.gen_range(1..=COLORS),
        x: PIECE_SPAWN_X as i16,
        y: PIECE_SPAWN_Y as i16,
        old_tiles: vec![],
    }
}

pub fn refresh_tiles(piece: &mut Piece, grid: &mut Vec<Vec<i16>>, shadegrid: &mut Vec<Vec<i16>>) {
    piece
        .old_tiles
        .iter()
        .map(|tile| grid[tile.0 as usize][tile.1 as usize] = 0)
        .count();
    piece.old_tiles = vec![];
    for tile in &piece.tiles {
        piece.old_tiles.push((tile.0 + piece.x, tile.1 + piece.y));
    }

    for x in 0..GRID_WIDTH as usize {
        for y in 0..GRID_HEIGHT as usize {
            shadegrid[x][y] = 0;
        }
    }

    // Load piece into grid
    for tile in &piece.tiles {
        grid[(tile.0 + piece.x) as usize][(tile.1 + piece.y) as usize] = piece.color as i16;
    }

    // Add shade of where piece would currently fall
    let mut shade: Vec<(i16, i16)> = piece.tiles.clone();
    for tile in &mut shade {
        let new_y = tile.1 + piece.y;
        *tile = (tile.0 + piece.x, new_y);
    }

    let mut final_dy = 0; // save last dy without overlap here
    'drop_piece: for dy in 1..GRID_HEIGHT as i16 {
        for tile in &shade {
            let x = tile.0 as usize;
            let y = (tile.1 + dy) as usize;
            if y >= GRID_HEIGHT as usize || (grid[x][y] > 0) {
                let mut piece_at = false;
                for t in &piece.tiles {
                    if t.0 + piece.x == x as i16 && t.1 + piece.y == y as i16 {
                        piece_at = true;
                        break;
                    }
                }
                if !piece_at {
                    final_dy = dy - 1;
                    break 'drop_piece;
                }
            }
        }
    }
    if final_dy == 0 {
        return;
    }
    for tile in shade {
        let x = tile.0 as usize;
        let y = (tile.1 + final_dy) as usize;
        if y >= GRID_HEIGHT as usize {
            break;
        }
        shadegrid[x][y] = 1;
    }
}
