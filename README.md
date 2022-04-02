<div align="center">

# 🌳 concord-bevy-app

**An experimental bevy app that achieves to support run-time level editing and content creation**
</div>

NOTE: only tested on Windows.

For more context, check out the [bevy-kajiya repo](https://github.com/Seabass247/bevy-kajiya). 

This bevy app uses the most advanced Rust renderer open-sourced yet, as of writing.

### Future Goals
- [x] Editor gui
- [x] Transform edit tools
- [x] In-game debugging/info console
- [x] Scene persistence via saving/loading and standardized file format
- [ ] Netorking
- [ ] Scripting

![alt text](https://github.com/Seabass247/concord-bevy-app/blob/main/screenshot.png)

## Usage

Navigating the scene:
- WASD + Q/E - movement and up/down.
- Left Click drag - move sun
- Right Click drag - rotate camera

1. `git submodule update --init --recursive`
1. Run `bake.cmd` for  the first time building or if you've made changes to mesh assets
2. Then do `cargo run`

### Configuration

These `WindowDescriptor` setting are recommended for `kajiya` to run properly as `kajiya` does not support resizable windows yet,
but you can change the window width/height as necessary.
```
        .insert_resource(WindowDescriptor {
            width: 1920.,
            height: 1080.,
            vsync: false,
            resizable: false,
            ..Default::default()
        })
```
