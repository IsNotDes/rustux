# SuperTux Sprite Import Guide for RustUX

This guide explains how to import and use SuperTux sprites in the RustUX project.

## Overview

RustUX includes a comprehensive sprite import system that downloads sprites directly from the official SuperTux repository and organizes them for use in the game. The system includes:

- Automatic sprite downloading from GitHub
- Sprite definition files for animation management
- Texture loading and management
- Easy-to-use sprite creation utilities

## Quick Start

### 1. Download SuperTux Sprites

First, download the essential SuperTux sprites:

```bash
cargo run --bin download_assets
```

This will:
- Create the `assets/` directory structure
- Download Tux sprites (small and big variants)
- Download common enemy sprites
- Create sprite definition files

### 2. Test Sprite Loading

Verify that sprites are loaded correctly:

```bash
cargo run --bin test_sprites
```

## Directory Structure

After running the asset downloader, you'll have:

```
assets/
├── sprites/
│   ├── creatures/
│   │   ├── tux/
│   │   │   ├── small/          # Small Tux sprites
│   │   │   │   ├── idle-0.png
│   │   │   │   ├── walk-0.png to walk-7.png
│   │   │   │   ├── jump-0.png, jump-1.png
│   │   │   │   ├── climb-0.png to climb-7.png
│   │   │   │   └── ...
│   │   │   └── big/            # Big Tux sprites
│   │   └── enemies/            # Enemy sprites
│   │       ├── bouncing_snowball/
│   │       ├── snowball/
│   │       └── ...
│   └── definitions/            # Sprite definition files
│       └── tux_small.json
```

## Using Sprites in Code

### Basic Sprite Creation

```rust
use rustux::sprite::SuperTuxSpriteFactory;
use rustux::math::Vector2;

// Create a Tux sprite with walking animation
let tux_sprite = SuperTuxSpriteFactory::create_tux_sprite(
    Vector2::new(100.0, 100.0),
    "walk"
)?;

// Create a sprite from a custom definition
let definition = SuperTuxSpriteFactory::load_sprite_definition("assets/sprites/definitions/tux_small.json")?;
let sprite = SuperTuxSpriteFactory::create_sprite_from_definition(
    &definition,
    Vector2::new(200.0, 200.0),
    "jump"
)?;
```

### Loading Textures

```rust
use rustux::sprite::TextureManager;
use rustux::assets::SpriteDefinition;

// Assuming you have an SDL2 texture creator
let mut texture_manager = TextureManager::new(&texture_creator);

// Load textures from a sprite definition
let definition = SpriteDefinition::load_from_file("assets/sprites/definitions/tux_small.json")?;
texture_manager.load_from_sprite_definition(&definition)?;
```

### Rendering Sprites

```rust
use rustux::sprite::SpriteRenderer;

// Render a single sprite
SpriteRenderer::render_sprite(&mut canvas, &texture_manager, &sprite)?;

// Render multiple sprites
let sprites = vec![&sprite1, &sprite2, &sprite3];
SpriteRenderer::render_sprites(&mut canvas, &texture_manager, &sprites)?;
```

## Sprite Definition Format

Sprite definitions are JSON files that describe animations:

```json
{
  "name": "tux_small",
  "texture_path": "sprites/creatures/tux/small",
  "animations": {
    "walk": {
      "frames": [
        {"file_name": "walk-0.png", "duration": null},
        {"file_name": "walk-1.png", "duration": null},
        ...
      ],
      "loops": true,
      "frame_duration": 0.1
    },
    "jump": {
      "frames": [
        {"file_name": "jump-0.png", "duration": null},
        {"file_name": "jump-1.png", "duration": null}
      ],
      "loops": false,
      "frame_duration": 0.2
    }
  }
}
```

## Available Animations

### Tux (Small)
- `idle`: Single frame idle animation
- `walk`: 8-frame walking cycle
- `jump`: 2-frame jumping animation
- `climb`: 8-frame climbing animation
- `grow`: 8-frame growth transformation

### Tux (Big)
- `idle`, `walk`, `jump`: Basic animations
- `fire-idle`, `fire-walk`, `fire-jump`: Fire Tux variants
- `duck`, `kick`, `skid`: Additional actions

## Adding New Sprites

### 1. Download Additional Sprites

Modify `src/assets/mod.rs` to add new sprite downloads:

```rust
impl AssetDownloader {
    pub async fn download_new_enemy(&self) -> Result<()> {
        let sprites = vec!["enemy-0.png", "enemy-1.png"];
        for sprite in sprites {
            let remote_path = format!("creatures/new_enemy/{}", sprite);
            let local_path = format!("sprites/creatures/enemies/new_enemy/{}", sprite);
            self.download_file(&remote_path, &local_path).await?;
        }
        Ok(())
    }
}
```

### 2. Create Sprite Definition

Create a new JSON definition file in `assets/sprites/definitions/`:

```json
{
  "name": "new_enemy",
  "texture_path": "sprites/creatures/enemies/new_enemy",
  "animations": {
    "walk": {
      "frames": [
        {"file_name": "enemy-0.png", "duration": null},
        {"file_name": "enemy-1.png", "duration": null}
      ],
      "loops": true,
      "frame_duration": 0.2
    }
  }
}
```

### 3. Use in Code

```rust
let definition = SuperTuxSpriteFactory::load_sprite_definition("assets/sprites/definitions/new_enemy.json")?;
let enemy_sprite = SuperTuxSpriteFactory::create_sprite_from_definition(
    &definition,
    Vector2::new(300.0, 300.0),
    "walk"
)?;
```

## Performance Tips

1. **Batch Texture Loading**: Load all textures at startup rather than on-demand
2. **Sprite Pooling**: Reuse sprite objects to reduce allocations
3. **Animation Caching**: Cache frequently used animations
4. **Texture Atlasing**: Consider combining small textures into atlases for better performance

## Troubleshooting

### Common Issues

1. **Missing Assets**: Run `cargo run --bin download_assets` to ensure all sprites are downloaded
2. **File Not Found**: Check that the sprite definition file exists and paths are correct
3. **Animation Not Found**: Verify the animation name exists in the sprite definition
4. **Texture Loading Fails**: Ensure the image files are valid PNG format

### Debug Commands

```bash
# Test sprite loading
cargo run --bin test_sprites

# Re-download assets
cargo run --bin download_assets

# Check asset directory structure
ls -la assets/sprites/creatures/tux/small/
```

## License

The SuperTux sprites are licensed under GPL-3.0, same as the original SuperTux project. Make sure to comply with the license terms when using these assets.

## Contributing

To add support for more SuperTux sprites:

1. Identify the sprites in the [SuperTux repository](https://github.com/SuperTux/supertux/tree/master/data/images)
2. Add download logic to `src/assets/mod.rs`
3. Create appropriate sprite definition files
4. Add convenience functions to `SuperTuxSpriteFactory`
5. Update this documentation

For questions or issues, please refer to the main RustUX documentation or open an issue in the project repository.