//! Audio system for RustUX

use crate::util::{Result, Error};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Audio clip for sound effects (simplified to avoid thread safety issues)
#[derive(Clone)]
pub struct AudioClip {
    /// Raw audio data
    data: Vec<u8>,
    /// Sample rate
    sample_rate: i32,
    /// Number of channels
    channels: u8,
}

impl AudioClip {
    /// Load an audio clip from a WAV file
    pub fn from_wav<P: AsRef<Path>>(path: P) -> Result<Self> {
        let wav = sdl2::audio::AudioSpecWAV::load_wav(path)
            .map_err(|e| Error::Audio(format!("Failed to load WAV file: {}", e)))?;
        
        Ok(Self {
            data: wav.buffer().to_vec(),
            sample_rate: wav.freq,
            channels: wav.channels,
        })
    }

    /// Create an empty audio clip
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            sample_rate: 44100,
            channels: 2,
        }
    }

    /// Get the audio data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> i32 {
        self.sample_rate
    }

    /// Get the number of channels
    pub fn channels(&self) -> u8 {
        self.channels
    }
}

/// Audio channel for playing sounds
#[derive(Clone)]
pub struct AudioChannel {
    /// Whether the channel is currently playing
    playing: bool,
    /// Current position in the audio data
    position: usize,
    /// Audio clip being played
    clip: Option<AudioClip>,
    /// Volume (0.0 to 1.0)
    volume: f32,
    /// Whether the audio should loop
    looping: bool,
}

impl AudioChannel {
    /// Create a new audio channel
    pub fn new() -> Self {
        Self {
            playing: false,
            position: 0,
            clip: None,
            volume: 1.0,
            looping: false,
        }
    }

    /// Play an audio clip
    pub fn play(&mut self, clip: AudioClip, looping: bool) {
        self.clip = Some(clip);
        self.position = 0;
        self.playing = true;
        self.looping = looping;
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.playing = false;
        self.position = 0;
        self.clip = None;
    }

    /// Set the volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Check if the channel is playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Mix audio data into the output buffer
    pub fn mix_audio(&mut self, output: &mut [i16]) {
        if !self.playing || self.clip.is_none() {
            return;
        }

        let clip = self.clip.as_ref().unwrap();
        let audio_data = clip.data();
        let bytes_per_sample = 2; // 16-bit audio
        
        for (i, output_sample) in output.iter_mut().enumerate() {
            let byte_pos = self.position + (i * bytes_per_sample);
            if byte_pos + 1 < audio_data.len() {
                // Convert little-endian bytes to i16
                let sample = i16::from_le_bytes([
                    audio_data[byte_pos],
                    audio_data[byte_pos + 1],
                ]);
                
                // Apply volume and mix
                let mixed_sample = (sample as f32 * self.volume) as i16;
                *output_sample = output_sample.saturating_add(mixed_sample);
            } else if self.looping {
                // Loop back to the beginning
                self.position = 0;
            } else {
                // End of audio, stop playing
                self.playing = false;
                break;
            }
        }
        
        // Update position
        self.position += output.len() * bytes_per_sample;
    }
}

/// Audio manager for handling sound effects and music
pub struct AudioManager {
    /// SDL2 audio subsystem
    audio_subsystem: sdl2::AudioSubsystem,
    /// Audio device
    _device: sdl2::audio::AudioDevice<AudioCallback>,
    /// Loaded audio clips
    clips: HashMap<String, AudioClip>,
    /// Audio channels for sound effects
    channels: Arc<Mutex<Vec<AudioChannel>>>,
    /// Music channel
    music_channel: Arc<Mutex<AudioChannel>>,
    /// Master volume
    master_volume: Arc<Mutex<f32>>,
}

/// Audio callback for SDL2
pub struct AudioCallback {
    /// Sound effect channels
    channels: Arc<Mutex<Vec<AudioChannel>>>,
    /// Music channel
    music_channel: Arc<Mutex<AudioChannel>>,
    /// Master volume
    master_volume: Arc<Mutex<f32>>,
}

impl sdl2::audio::AudioCallback for AudioCallback {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        // Clear the output buffer
        for sample in out.iter_mut() {
            *sample = 0;
        }

        let master_vol = *self.master_volume.lock().unwrap_or_else(|poisoned| {
            // If the mutex is poisoned, extract the value from the poisoned guard
            poisoned.into_inner()
        });

        // Mix music
        if let Ok(mut music) = self.music_channel.try_lock() {
            music.mix_audio(out);
        }

        // Mix sound effects
        if let Ok(mut channels) = self.channels.try_lock() {
            for channel in channels.iter_mut() {
                channel.mix_audio(out);
            }
        }

        // Apply master volume
        for sample in out.iter_mut() {
            *sample = (*sample as f32 * master_vol) as i16;
        }
    }
}

impl AudioManager {
    /// Create a new audio manager
    pub fn new(audio_subsystem: sdl2::AudioSubsystem) -> Result<Self> {
        let channels = Arc::new(Mutex::new(vec![AudioChannel::new(); 8])); // 8 sound effect channels
        let music_channel = Arc::new(Mutex::new(AudioChannel::new()));
        let master_volume = Arc::new(Mutex::new(1.0));

        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2), // Stereo
            samples: Some(1024),
        };

        let callback = AudioCallback {
            channels: channels.clone(),
            music_channel: music_channel.clone(),
            master_volume: master_volume.clone(),
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |_spec| callback)
            .map_err(|e| Error::Audio(format!("Failed to open audio device: {}", e)))?;

        device.resume();

        Ok(Self {
            audio_subsystem,
            _device: device,
            clips: HashMap::new(),
            channels,
            music_channel,
            master_volume,
        })
    }

    /// Load an audio clip
    pub fn load_clip<P: AsRef<Path>>(&mut self, name: &str, path: P) -> Result<()> {
        let clip = AudioClip::from_wav(path)?;
        self.clips.insert(name.to_string(), clip);
        log::debug!("Loaded audio clip: {}", name);
        Ok(())
    }

    /// Play a sound effect
    pub fn play_sound(&self, sound_name: &str) -> Result<()> {
        let clip = self.clips.get(sound_name)
            .ok_or_else(|| Error::Audio(format!("Sound not found: {}", sound_name)))?;

        if let Ok(mut channels) = self.channels.lock() {
            // Find an available channel
            for channel in channels.iter_mut() {
                if !channel.is_playing() {
                    channel.play(clip.clone(), false);
                    return Ok(());
                }
            }
            log::warn!("No available audio channels for sound: {}", sound_name);
        }

        Ok(())
    }

    /// Play a sound effect with looping
    pub fn play_sound_looped(&self, sound_name: &str) -> Result<()> {
        let clip = self.clips.get(sound_name)
            .ok_or_else(|| Error::Audio(format!("Sound not found: {}", sound_name)))?;

        if let Ok(mut channels) = self.channels.lock() {
            // Find an available channel
            for channel in channels.iter_mut() {
                if !channel.is_playing() {
                    channel.play(clip.clone(), true);
                    return Ok(());
                }
            }
            log::warn!("No available audio channels for looped sound: {}", sound_name);
        }

        Ok(())
    }

    /// Play background music
    pub fn play_music(&self, music_name: &str) -> Result<()> {
        let clip = self.clips.get(music_name)
            .ok_or_else(|| Error::Audio(format!("Music not found: {}", music_name)))?;

        if let Ok(mut music_channel) = self.music_channel.lock() {
            music_channel.play(clip.clone(), true); // Music always loops
        }

        Ok(())
    }

    /// Stop background music
    pub fn stop_music(&self) {
        if let Ok(mut music_channel) = self.music_channel.lock() {
            music_channel.stop();
        }
    }

    /// Stop all sound effects
    pub fn stop_sounds(&self) {
        if let Ok(mut channels) = self.channels.lock() {
            for channel in channels.iter_mut() {
                channel.stop();
            }
        }
    }

    /// Stop all audio
    pub fn stop_all(&self) {
        self.stop_music();
        self.stop_sounds();
    }

    /// Set master volume (0.0 to 1.0)
    pub fn set_master_volume(&self, volume: f32) {
        if let Ok(mut master_vol) = self.master_volume.lock() {
            *master_vol = volume.clamp(0.0, 1.0);
        }
    }

    /// Get master volume
    pub fn get_master_volume(&self) -> f32 {
        *self.master_volume.lock().unwrap_or_else(|poisoned| {
            poisoned.into_inner()
        })
    }

    /// Set music volume
    pub fn set_music_volume(&self, volume: f32) {
        if let Ok(mut music_channel) = self.music_channel.lock() {
            music_channel.set_volume(volume);
        }
    }

    /// Set sound effects volume
    pub fn set_sound_volume(&self, volume: f32) {
        if let Ok(mut channels) = self.channels.lock() {
            for channel in channels.iter_mut() {
                channel.set_volume(volume);
            }
        }
    }

    /// Check if music is playing
    pub fn is_music_playing(&self) -> bool {
        self.music_channel.lock()
            .map(|channel| channel.is_playing())
            .unwrap_or(false)
    }

    /// Get the number of loaded clips
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// Check if a clip is loaded
    pub fn has_clip(&self, name: &str) -> bool {
        self.clips.contains_key(name)
    }

    /// Remove a clip from memory
    pub fn unload_clip(&mut self, name: &str) -> bool {
        self.clips.remove(name).is_some()
    }

    /// Clear all loaded clips
    pub fn clear_clips(&mut self) {
        self.clips.clear();
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        // This will panic if SDL2 is not initialized, but that's expected
        let sdl_context = sdl2::init().expect("SDL2 not initialized");
        let audio_subsystem = sdl_context.audio().expect("Failed to get audio subsystem");
        Self::new(audio_subsystem).expect("Failed to create AudioManager")
    }
}

/// Audio configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Music volume (0.0 to 1.0)
    pub music_volume: f32,
    /// Sound effects volume (0.0 to 1.0)
    pub sound_volume: f32,
    /// Whether audio is enabled
    pub enabled: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sound_volume: 0.8,
            enabled: true,
        }
    }
}

/// Audio utilities
pub mod utils {
    /// Convert volume from percentage (0-100) to float (0.0-1.0)
    pub fn volume_from_percentage(percentage: u8) -> f32 {
        (percentage.min(100) as f32) / 100.0
    }

    /// Convert volume from float (0.0-1.0) to percentage (0-100)
    pub fn volume_to_percentage(volume: f32) -> u8 {
        (volume.clamp(0.0, 1.0) * 100.0) as u8
    }

    /// Apply fade effect to volume
    pub fn apply_fade(current_volume: f32, target_volume: f32, fade_speed: f32, delta_time: f32) -> f32 {
        if (current_volume - target_volume).abs() < 0.01 {
            target_volume
        } else if current_volume < target_volume {
            (current_volume + fade_speed * delta_time).min(target_volume)
        } else {
            (current_volume - fade_speed * delta_time).max(target_volume)
        }
    }
}