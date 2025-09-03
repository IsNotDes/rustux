# RustUX - SuperTux Remake in Rust

A complete reimplementation of the classic SuperTux platformer game, built from the ground up in Rust with modern game development practices.

## Overview

RustUX is inspired by the original SuperTux game (https://github.com/SuperTux/supertux) but reimplemented entirely in Rust. The goal is to create a performant, safe, and maintainable version of the beloved 2D platformer while preserving the original gameplay experience.

## Features (Planned)

- **Modern Rust Architecture**: Built with safety, performance, and maintainability in mind
- **Cross-platform**: Runs on Windows, macOS, and Linux
- **Modular Design**: Clean separation of concerns with dedicated systems for:
  - Graphics and rendering (SDL2)
  - Audio system
  - Physics and collision detection
  - Input handling
  - Game object management
  - Level loading and scripting
  - GUI system

## Current Status

🚧 **Early Development**🚧

The project currently has:
- ✅ Basic project structure and dependencies
- ✅ Core game engine architecture
- ✅ Math utilities and vector operations
- ✅ SDL2 graphics system setup
- ✅ Game state management system
- ✅ Resource management system
- ✅ Basic module structure for all game systems
- 🔄 Basic sprite system (stub)
- 🔄 Audio system (stub)
- 🔄 Physics and collision detection (stub)
- 🔄 Player character implementation (stub)
- ❌ Level loading system
- ❌ Enemy/badguy system
- ❌ GUI implementation
- ❌ Game content (levels, sprites, sounds)

## Building

### Prerequisites

- Rust (latest stable version)
- SDL2 development libraries

### Ubuntu/Debian
```bash
sudo apt-get install libsdl2-dev
```

### macOS
```bash
brew install sdl2
```

### Windows
Download SDL2 development libraries from https://www.libsdl.org/

### Building the Project

```bash
git clone <repository-url>
cd rustux
cargo build
```

### Running

```bash
cargo run
```

## Architecture

The project follows a modular architecture inspired by the original SuperTux:

```
src/
├── lib.rs              # Main library with module declarations
├── main.rs             # Application entry point
├── engine/             # Core game engine
│   ├── mod.rs          # Engine management and game loop
│   ├── game_state.rs   # State management system
│   └── resource_manager.rs # Asset loading and management
├── math/               # Mathematical utilities
├── util/               # Common utilities and error handling
├── audio/              # Audio system
├── video/              # Rendering system
├── sprite/             # Sprite management
├── control/            # Input handling
├── physics/            # Physics simulation
├── collision/          # Collision detection
├── object/             # Game object system
├── badguy/             # Enemy system
├── gui/                # User interface
├── trigger/            # Event triggers
└── supertux/           # Main game logic and player
```

## Contributing

This project is in early development. Contributions are welcome! Please feel free to:

- Report bugs and issues
- Suggest features
- Submit pull requests
- Improve documentation

## License

This project is licensed under the GPL-3.0 license, same as the original SuperTux project.

## Acknowledgments

- Original SuperTux team for creating the amazing game that inspired this project
- SDL2 developers for the excellent multimedia library
- Rust community for the fantastic ecosystem and tools

## Development Roadmap

See the project issues and milestones for detailed development plans and progress tracking.