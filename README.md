# Rustetris

Simple tetris with Rust for learning. Using pixels and winit.

### Known issues

Tiles sometimes are left hanging in air if the rest of the piece is blown up. Probably should not happen.

### Task list

1. Show scores afterwards
2. Menu to restart game, or to choose bigger/smaller or faster game

### Done tasks
1. Destroy a full line and move all other dead pieces downwards
2. Create other types of pieces randomly
3. Recognize when game is over, meaning when a spawned piece cannot move even once (or is already overlapping)
4. Drop with space or downkey
5. Show where piece is gonna fall to
6. Show what piece comes next
7. Speed up
