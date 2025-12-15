# Project Pulse

Project Pulse is a digital audio workstation (DAW) developed in Tauri (Rust + React). Similar software would be FL Studio, Ableton Live, Logic Pro and others. There are many things I hope to improve compared to these programs, including fewer crashes (thanks to Rust) and a snappy interface (thanks to React). I also plan to combine the best features from existing DAWs with some of my own ideas.

There are several goals for this project:
- Hone my React and JS/TS skills
- Learn and better understand the Rust language, especially its borrow checker
- Combine my two big hobbies, music production and programming
- Build an actual product that people would want to use and enjoy

At the moment, users can only select a folder and play audio files from it (there is a file browser panel on the left side of the screen). However, it is very low-level. A file is read and then decoded to raw PCM data. If the file's sample rate differs from the output's, it is resampled in realtime. Lastly, the resulting samples are sent to the audio output in advance using a ring buffer.

# To-Do for basic DAW features
- A timeline where users can arrange multiple audio files in time to create a song
- Settings panel
- Save/load a project
- Basic audio manipulation (volume, panning, etc.) for mixing purposes
- Realtime audio mixing (summing all samples for each audio callback using SIMD and multithreading for optimization)
- Mixer (used to put effects on tracks and mix them together)
- Piano roll (manipulate the pitch of audio samples by placing notes in time; later to be used with synths as well)
- Automations (change audio manipulation values over time)
- Support for hosting third-party effect plugins and synths (VST3-based)

# Running the project
First, make sure you have all the prerequisites: https://v2.tauri.app/start/prerequisites/. Then run the following commands:
1. `cd tauri-app`
2. `npm install`
3. `npm run tauri dev`
