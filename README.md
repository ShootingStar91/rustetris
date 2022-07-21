# Rustetris

Simple tetris with Rust for learning. Using pixels and winit.

# Instructions to run

### Windows

Download and run [the exe file of release 0.1](https://github.com/ShootingStar91/rustetris/releases/download/0.1/rustetris.exe)

### Linux

Download the [linux executable file](https://github.com/ShootingStar91/rustetris/releases/download/0.1/rustetris), should work on at least Debian based systems... no guarantees.

To run, in the download folder do `sudo chmod 777 ./rustetris` and then `./rustetris`

Alternatively, clone and run the project's source directly with Rust's package manager cargo.

```
git clone https://github.com/ShootingStar91/rustetris
cd rustetris
cargo run
```

![Rustetris image](picture.png "")


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
