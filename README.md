# Project Pulse

Project Pulse is a digital audio workstation (DAW) developed in Tauri (Rust + React). Similar software would be FL Studio, Ableton Live, Logic Pro and others. There are many things I hope to improve compared to these programs, including fewer crashes (thanks to Rust) and a snappy interface (thanks to React). I also plan to combine the best features from existing DAWs with some of my own ideas.

There are several goals for this project:
- Hone my React and JS/TS skills
- Learn and better understand the Rust language, especially concepts like ownership and the borrow checker
- Combine my two big hobbies: music production and programming
- Build an actual product that people would enjoy using

At the moment, there are just two working parts; a file browser with real-time audio previewing and a timeline for mixing. You can drag&drop files to the timeline and rearrange them as you like. Even though a lot of backend stuff is done, the core part - actual mixing and sending the final mixed samples to the speakers for playback, is still in the works. I'm still figuring out the best way to structure the code for efficiency. Audio programs like this need to be extremely optimized to run without hiccups, and also be real-time safe and thread-safe. I will write about it in more detail once I have something ready.

Just to give a glimpse of how much work goes into this, here's how just basic audio previewing works under the hood; audio data is read from a file and then decoded to raw PCM data, which is basically an uncompressed digital representation of analog data. If the audio's sample rate (how many samples per second there is) differs from the output's, it is resampled in real-time. Lastly, the final samples are sent to the audio output (slightly) in advance using a ring buffer. A sample is, for example, just a number between -1.0 and 1.0 for floating-point formats. Since I've chosen f32 format (it's the most common format; some DAWs also use f64, but the performance impact overshadows any sort of advantage in precision so I didn't bother with supporting it just yet) for the underlying audio engine, any other format is converted to f32 before being used by the engine. Also, the underlying audio engine uses exclusively stereo mode (2 channels - left and right; a standard in music production) so if an audio file is not in stereo (almost always will be, though), it's downmixed or upmixed using simple well-known math formulas.

# To-Do for basic DAW features
- A timeline where users can arrange multiple audio files in time to create a song
- Settings panel
- Save/load a project
- Basic audio manipulation (volume, panning, etc.) for mixing purposes
- Real-time audio mixing (summing all samples for each audio callback using SIMD and multithreading for optimization)
- Mixer (used to put effects on tracks and mix them together)
- Piano roll (placing notes in time and space, best analogy would be sheet music)
- Automations (change audio manipulation values over time)
- Support for hosting third-party effect plugins and synths (VST3-based)

# Running the project
First, make sure you have all the prerequisites: https://v2.tauri.app/start/prerequisites/. Then run the following commands:
1. `cd tauri-app`
2. `npm install`
3. `npm run tauri dev`
