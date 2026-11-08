# Audio - INDEX

_owner: audio agent. Crate `velocity_audio` (`cargo test -p velocity_audio`). Details live in `docs/audio/details.md` (the full former audio.md): NEVER read it whole; grep it by heading (`grep -n '^\*\*' docs/audio/details.md`, then `Read` with offset/limit)._

## APIs other crates use (details: "Use from other crates")
- `AudioPlugin` -> resource `VisosAudio { system, game: VsAudio, mixer }`; resource `GameAudioInput` (overwrite each frame: camera, player car rpm/gear/speed/wheels/surfaces, ai, traffic, police, wet, tft).
- Message `AudioCommand`: Init{frontend,track_dir,race_mode}, LoadFrontEndGroups, Gui(GuiSound), Nos/Slipstream/Zone/Tb/Mode3 start/stop, Countdown, RaceStarted, Impact{event,pos,strength}, Settings, PlayTrack, Pause/Resume, Shutdown.
- `AudioCommand` loads: LoadLevelSounds{track_dir,wet}, LoadRevBands{name}, LoadAiEngine{path}, GlassImpact{pos}, SetLists(SavedPlaylists) (gather = `VisosAudio.game.get_lists()`).
- `AudioCommand` timeline/misc: TimelineSound{object,event} / TimelineSoundStop{object,event?}, FadeToVolumePreset{preset,time}, CityAmbience(bool), StopMusicEvent, StartLoopEvent210 / StopLoopEvent210 / FadeMusic{to,env,time}.
- Resource `MovieAudioOut { rate, channels, pcm, volume, paused, open, generation }` (Bink movie sound; frontend re-exports as `movie::MovieAudioOut`): `open(rate,ch)` at BinkOpen, push s16 into `pcm`, `close()` at BinkClose.
- Resource `AudioUpdateGate(bool)` (default true): false skips `VsAudio::update`.
- System order: `AudioSystems::Commands` then `::Update` in `Update`. Race order: Shutdown(FE) -> Init -> LoadLevelSounds -> LoadRevBands -> Countdown.
- Debug-menu tweaks: `audio_tweaks` call lists + value slots (12 systems, 36 tweaks), replayed by core; `sync_audio_tweaks` in `First`.
- Env: `VELOCITY_AUDIO_TRACE=<file>` (+ `VELOCITY_AUDIO_TRACE_ENGINE=1`), `VELOCITY_AUDIO_OUT=null` (hidden_run default; `--sound` opts out), `VELOCITY_MUSIC=wma` (feature `wma`), `VELOCITY_AUDIO_MUTE=1` (silent device output), `VELOCITY_AUDIO_DIAG=<file>` (+`_MUTE=1` / `_GAIN=<g>`: device-pull stats + rms; with `--sound` always set MUTE), loopback probe `--example loopback`.

## Music randomisation (2026-11-08)
- `AudioSystem.music_rng`/`music_rand()` = the music's own stream (pick_list + play_track); `VisosAudio::default` calls `randomize_music_per_launch()` (system-time seed, `music_random=true`: no immediate repeat, random start in sequential lists) unless `VELOCITY_HIDDEN`/`VELOCITY_GOLDEN` is set (then the original deterministic picks, seed 1). Effect rng `rand()` untouched. Unit test only (`music_rng_is_separate_and_seedable`).

## OPEN requests
- Requests from frontend (2026-11-09, PC settings "Music: Original Order / Random per Launch"): frontend's `pc_settings::apply_pc_settings` sets `VisosAudio.system.music_random = false` for Original and calls `randomize_music_per_launch()` for Random. Please add a public `AudioSystem::original_music_order()` (music_rng = 1, music_random = false) so Original reproduces the original pick sequence exactly; frontend will switch to it.
- To core: clear `AudioUpdateGate` during the race load (VsAudio::Update is never called between race Init and LoadLevelSounds, so city_ambience / passbys fail on the second race of a career); also replay the audio AddTweaks ("Debug-menu tweaks").
- User report 2026-11-08 "sound barely works / not at all" (dist exe): NOT reproduced; device path measured healthy (details "Device-path check"). DONE as robustness fix: `audio/output.rs` owns the device (supervisor thread, rodio sink on the cpal default device; reopens on stream error callback or default-device id change, retries every 2 s after a failed open, falls back to another output if the default will not open; the mixer is never touched). Every open/reopen/failure is logged at info/warn ("audio: output opened/reopened ..." with name, id, channels, rate, format, buffer). Tests: `reopens_on_error_and_default_change` (fake backend); `-- --ignored real_device_muted` (real device, muted). `VELOCITY_AUDIO_MUTE=1` mutes the device output (no diag file needed). Still need from the user: the log lines and their device.
- Request to core (optional): bevy's own `AudioPlugin` still opens the default device (idle) — disable it in main.rs always (it is only disabled for the recorder) so it cannot hold/fail a second stream.
- From audio: none other open. Unchecked: whether the FE `Init` creates a player emitter so loop 210 is audible.

## Next candidates / coverage
- Ported: xva/xvg/rbd/aee/MusLists readers, event engine, emitters, rev bands, soundtrack, Bink sound driver, tweaks, playlist editor API. Known data/original failures: TB_ATMOS, glass 0x3B5B1A82, numeric 979, LADown aee BeverlyHills3.
- Coverage 2026-11-07: 49.9% -> 85.6% by bytes (251/418 functions, 91334/106681 B) of the audio scope (every `*AUDIO_*` function of the visos Audio layer + src/Audio + TimelineSound/GlobalAudio; objregions.tsv mis-attributes the visos audio code, so scope = `research/audio_names.tsv` from `tools/ida/audio_cov_names.py`). Regenerate: `python tools/coverage/cov.py audio research/audio_names.tsv` then `tools/coverage/audio_tag.py`; list `research/audio_coverage.txt`. Left: platform (software mixer/resampler/pools/dsound/wma packets, replaced by `mixer.rs`) 77/10.3 KB, ctor-dtor 50/3.7 KB, getters 35/1 KB, real behaviour 0. Origin lines for audio-crate code use `/// <name>.cpp 0x...` (visos layer: the function name as file stem).
- Playlist editor (`VsAudioTrackList` build_list/add_track_by_id/by_index/remove_track/shuffle/commit_list/get_track(_id)/set_list_type/shuffle/free_lists, `VsSoundtrackPlayer` next/prev song/soundtrack, shutdown) is ported with a test but nothing calls it yet: the only original caller is the Quick Race music playlist screen (`FEQuickRaceTextScreen__22C880/22CA90/22CCD0/22D080/22C560`, frontend). Request to frontend: drive it through `VisosAudio.game.tracks` (`edit[list]` arena, node 0 = head).

## Section index of details.md (heading, line)
- L1: Audio (crate, tests)
- L6: Use from other crates (commands, resources, diagnostics, null output)
- L29: Module map
- L38: Formats (.XVA, .xvg, .rbd, .aee/.vol/.dfm, unused files)
- L56: Decisions / quirks (frame rate, impacts, LoadLevelSounds, trailblazer, playlists, countdown, wet, rev bands, AI engines, engine rpm, voice id bug, review smokes, Review 2)
- L150: Debug-menu tweaks
- L162: Requests from core / graphics (done)
- L165: Bink movie sound (done; driver, timing, tests)
- L195: DONE 2026-11-06 (StartLoopEvent210 / FadeMusic)
- L202: Requests from frontend (hangout music, done)
- L211: Device-path check (2026-11-08 user report, measurements, diag tools)
