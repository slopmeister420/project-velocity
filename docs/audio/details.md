# Audio

_owner: audio agent_ — crate `velocity_audio` (`cargo test -p velocity_audio`, 36 tests; all-files positive controls: xva, xvg (`data_tests`), rbd, aee, MusLists.dfm, velocity.vol; `--features wma` adds the WMA path;
dev-dependency `velocity_physics` for the vehicle database rev band names).

**Use from other crates.** `AudioPlugin` adds resource `VisosAudio { system: AudioSystem, game: VsAudio, mixer }`,
resource `GameAudioInput` (overwrite each frame: camera, player car rpm/gear/speed/wheels/surfaces, ai, traffic,
police, wet, tft) and message `AudioCommand` (Init{frontend,track_dir,race_mode}, LoadFrontEndGroups, Gui(GuiSound),
Nos/Slipstream/Zone/Tb/Mode3 start/stop, Countdown, RaceStarted, Impact{event,pos,strength}, Settings, PlayTrack,
Pause/Resume, Shutdown, LoadLevelSounds{track_dir,wet} (GameDataLoading 0xA1190, load step LevelSounds),
LoadRevBands{name} (PhysicsManager 0x19F149, vehicle 0, `VehicleAudio::model`), LoadAiEngine{path} (0x531D0 from
PhysicsManager 0x19F2B0, per ai race vehicle, `VehicleAudio::audio`), GlassImpact{pos} (CarDamageFX
0x78320), SetLists(SavedPlaylists) (save load 0x104F30; gather = `VisosAudio.game.get_lists()`, 0x105E10),
timeline TimelineSound{object,event} / TimelineSoundStop{object,event?} (TimelineSound.cpp 0x17EA00 / 0x17EBD0 /
0x17EB80: one 2D emitter per object crc, flags 0x40000001, AUDIO 0x304F30 = `stop_emitter_event`),
FadeToVolumePreset{preset,time}, CityAmbience(bool), StopMusicEvent...); resource `MovieAudioOut` (Bink movie sound,
below).
Race order: Shutdown(FE) -> Init -> LoadLevelSounds -> LoadRevBands -> Countdown... (core wiring requested, core.md
"Requests from audio"). Diagnostics: `VELOCITY_AUDIO_TRACE=<file>` (module `trace`) writes a per-second summary
(dt range, player rpm, active events + pitch ratio, PlayEvent calls with `!FAIL`, music state, `cmds:` the
AudioCommands applied); with `VELOCITY_AUDIO_TRACE_ENGINE=1` also one `E` line per frame (player rpm in, gear,
throttle, x1.25 rpm, rev band rpm/norm/smoothed, each playing band `i:cents/vol`, ai `rpm:cents/vol`). **Null output**: `VELOCITY_AUDIO_OUT=null` keeps mixing (a thread pulls the mixer at real
time, so voices end as with a device) but registers no Bevy audio source: nothing reaches the speakers.
`tools/hidden_run.py` sets it by default (`--sound` opts out; audio's one edit outside its crate). With
`SMOKE_RECORD` Bevy's AudioPlugin is off and the recorder pulls the mix instead (wav capture unchanged). Order: `AudioSystems::Commands` then `::Update` in `Update`. Output is an engine-side
48 kHz stereo mixer exposed as bevy audio source `VisosMixOutput` (only registered if bevy's AudioPlugin exists;
headless works). Everything decodes in memory at runtime from raw game files via `visos::file`.

**Module map.** `audio/` = visos audio layer (AUDIO_* in the binary): `xva` banks, `adpcm`, `xvg` groups, `load`
(LoadGroup 0x2BD6C0, LoadBank 0x316450), `event` (PlayEvent 0x2BE440, selection, chains, update 0x314DF0), `sample`,
`emitter` (3D 0x315180), `system` (Update 0x2BCFE0, category volumes/presets), `xbox/voice` (dsound voices; DS3D
emulated: inverse distance, pan, doppler), `xbox/bink_sound` (RAD Bink's dsound driver, movie sound), `mixer`. Game side: `vs_audio` (VsAudio.cpp, `g_AudioSystem` 0x6D4558,
Init 0x47A00, Update 0x48BC0), `vs_audio_vehics` (VsAudioVehics.cpp from 0x4FD50), `vs_audio_events` (player engine,
gear/turbo, old-id table 0x684F10), `vs_audio_rev_bands` (VsAudioRevBands, 0x4C6F0..0x4D470), `vs_audio_track_list`
(MusLists.dfm), `vs_soundtrack` (soundtrack player 0x54FD0..0x56840), `wma` (feature), `input`, `plugin`, `trace`
(port diagnostics). Decompile dumps: `research/audio/`.

**Formats.**
- `.XVA` 'ASVX' v3: 16-B header (magic, ver, sampleCount, bankFlags) + 12-B {size, format, fileOffset}. format bits
  0-3 codec (0 PCM, 1 Xbox ADPCM), 4-6 channels, 7-11 bits, 12-15 rate idx (1 8000,2 11025,3 12000,4 16000,
  5 22050,6 24000,7 32000,8 44100,9 48000,A 64000,B 28000,else 30000). bankFlags&0x70000 residency (0x20000 =
  streamed, 2 KB-aligned). Xbox ADPCM = IMA, 36-B blocks/channel (s16 pred, u8 index, u8 0), stereo interleaves
  4-B words, 64 frames/block (header + 63 nibbles; the 64th nibble is filler). 8-bit PCM is signed.
- `.xvg` 'GSVS' v3: events 20 B {hash, ?, flags, ?, soundCount}, sounds 64 B (weight, vol/pitch(cents)/delay ranges,
  selection flags), samples 16 B {weight, vol%, bank, sample}, chains 12 B, then debug names. event hash =
  HANDY_CRC32(uppercase name, 0); EnvFX groups use small numeric ids. Selection = flags&0xF00 (0 random, 0x100 seq,
  0x200 control param, 0x300/400/500 min/mid/max), 0x1000 weighted, 0x2000 no-repeat; event 0x20000 loop.
- `.rbd` rev bands: group[128], count, cents/rpm ranges, smoothing, volume; 40-B bands {event, lo, hi, fadeIn,
  fadeOut, vol, centsRef, cents, flags 1 on/2 on-load/4 off-load}.
- `envaudio.aee` 'AED1': count, 76-B records {name[32], pos (z negated on load), dir, min, vol, max, ?, event};
  names "GLOBAL_" / "ONESHOT_a:b". `velocity.vol` = 32x32 category volume presets. `MusLists.dfm` (MDV2 text):
  3 lists + master table name→`.wma` path; track names are also music-bank event names.
- Unused by the game: `ProjectVelocity.mix`, `vsfx.bin`, `asp/*` (no xbe refs); `voice.bin` = DSP image.
  Music XVA banks (`Audio\Music\fe\music_fe`, `ingame\music`) are not loaded by the original but hold the same songs.

**Decisions / quirks.**
- Music default = XVA music banks (exact); WMA (WMAv2 44.1k, 68 files) via `oxideav-wma` 0.0.4 behind feature
  `wma` + env `VELOCITY_MUSIC=wma`: decodes all files with 0 frame failures but only ~20 dB SNR vs ffmpeg.
- Data bugs kept: vvipesu0.xvg references samples 3..5 of a 3-sample bank; aee "volume" values (e.g. 40) fail
  SetEmitterVolume's range check exactly like the original; LACoast AudioEmitter_5 has max distance 0.
- Dead in this build: software mixdown path, road-noise/wheelspin events, numeric events 210/979/1314/1801.
- Original sets only the DS listener position (no orientation/velocity); doppler factor 0.3, speed 343.3.
  Verified in IDA 2026-11-09 (`_AUDIO_UpdateListenerM` 0x341BD0 averages front/top then calls only SetPosition
  via 0x313B50): the original pans every positional sound by WORLD X. Deliberate deviation (user bug "player
  car sound swings L/R"): `VoiceSystem::apply_orientation` (default true) applies the camera orientation
  (un-flipping the x/z flip of VsAudio::update_listener) so pan is camera-relative; false = original.
  Player engine itself (rev bands) is FLAT/2D; the panned player sounds are the positional emitters (tyres,
  skids, drift, brakes, left/right pairs). Test `chase_camera_pan_is_heading_independent`.
- **Frame rate.** VsAudio::Update runs once per rendered frame (GameManager::InGameUpdate 0xB7EE7) with the frame
  dt; the engine (AUDIO_Update) clamps dt to 0.05 (0 -> 1/30). The game-side steps were per rendered frame at the
  60 Hz reference (core.md "Original render rate"); the port scales them by `VsAudio::frame_scale` = clamped dt·60
  (exact original arithmetic at 60 Hz): impact history age (15 frames), suspension airtime counters (5), NOS /
  slipstream param steps, TB fade, skid/brake leaky accumulators (`leaky`: same steady state and time constant),
  rev band smoothing/acc factors (`per_frame` = 1-(1-a)^s). Physics-fed deltas (compression jumps) are per 24 Hz
  tick and need no scaling as long as `GameAudioInput` is not interpolated. Test `race_session_engine_at_any_rate`.
- PlayPhysicsImpact (0x52E90) original bug kept: the object branch clears the "allowed" flag (var_25 at 0x53042),
  so the close-range car-layer replay only runs for `car_layer_only` calls (the history list is still filled).
- Glass impact event 0x3B5B1A82 is in no shipped group: only its IMPACT_CAR_PHYSICS follow-up sounds (rand%700+100
  strength clamps to 1), at the car's position (CarDamage 0x761B0), not the hit point.
- LoadLevelSounds: EnvFX groups = `\<track>Audio\en<name cut to 7>m/s`; then (0x000A21A6) "\Tracks\LON" and byte
  14 != '2' -> `<track>Audio\asPetrol/asPetros/ashop` (London petrol station), **else** "\Tracks\LAD" ->
  `<track>Audio\aschina/aschinam` (not shipped, logged) `/asbilbrd/ascarpk/ascarpkm/ascarpks/ashydrnt`,
  `\Audio\surface\tarmac\sftarmw`, `\Audio\fx\mdtrail` (TB_ENTER, tb_rumble) and `mdtrails` (not shipped). Fixed
  2026-11-05: the port had all of it in the London block, so LADown's trailblazer (tb_enter), carpark timeline sounds
  (ASP_CARPARK_*), billboard and hydrant (ASP_ENV_FIREHYDRANT, physics AddEnvSound) were silent.
- **Trailblazer** (StartTb 0x0004BEA0 / UpdateTb 0x0004BF40 / SetTbFadeIn 0x0004C050 / StopTb 0x0004C080 re-read):
  SetTbFadeIn writes **0x3D08882F (1/30)**; the port had the .data default of 0x684F08 (0x3C088723, 1/120) — fixed.
  TB_ATMOS (0x231D8755) is in **no shipped group**, so every UpdateTb while tb is on calls StopSoundtrack, fails the
  play and re-applies preset 15 (original; the soundtrack restarts and is stopped in the same Update each frame:
  music silent during a trailblazer). Seen (LADown Race02, `review2/tb.txt`): StartTb -> tb_enter, StopTb -> music.
- Playlists: `SavedPlaylists` = options +0x28 (3×16 B lists {name, count, type, shuffle}, 150×12 B tracks {hash,
  soundtrack, master}, 100×8 B soundtrack-list {type, shuffle}); SetLists 0x4E2A0 re-fills names from the master
  table, sets soundtrack types/shuffles (list i -> soundtrack i-2) and restarts a playing soundtrack; InitMusic
  re-applies the kept copy when `lists[0].name != 0`. PickList/PlayTrack now read the real soundtrack types/shuffles.
- Countdown: MonitorSessions 0x000BD263 passes a constant 3.0 (not dt) to UpdateCountdown 0x52B40, so the flag
  girl's "ready" plays on the first countdown call; RaceStarted 0x52C20 plays go + airhorn and starts the music.
- IsTrackWet 0x4B0A0 = race def (+0x2C78 -> 0x17CB0) +22 & 1; the port latches `LoadLevelSounds { wet }` in
  `VsAudio::track_wet` (`is_track_wet` = that || `GameAudioInput::track_wet`): rain loops + wet skids/passbys.
- Rev bands: the vehicle database names only 3 models: `tier2\civic\vcivirb0`, `tier4\wrx\vimprrb0` (both load,
  tested over the rpm sweep) and `tier1\mr2\vmr2rb0`, which is **not shipped** (that car's engine is silent, as in
  the original; "Unable to load revband" logged).
- **AI racer engines** (the earlier "no ai engine loop" note was wrong: the idb's `VsAIPlayerTFTManager__17310` is
  `m_pAIRaceVehicles[i]`). The only caller of LoadAiEngine 0x531D0 is PhysicsManager 0x19E080 (0x19F2B0): per
  non-human race vehicle, after its physics setup, `AUDIO_LoadGroup(entry +0x40 +4)` ("Load AI Engine Audio"
  loading timer; failure logs "Failed to load audio group 'AIEngine'" + assert line 1437); then ai +356 =
  VsStringAnsi::Hash (0x2B46B0 = HANDY_CRC32) of the base name, upper-cased in place. UpdateAiCars 0x53290
  re-reads +12 players / +16 ai / +20 sum / +24 traffic count every update (port: from `GameAudioInput`, one player
  car), loops i in 1..+20: i < players = human (emitters only), else ai i-players -> loop at its +356 event, pitch
  -1000..50 cents over 1000..`ai_max_rpm`. All 25 vehicle-db ai groups load; 23 have their base-name event; data
  bugs: `v240sai0.xvg` holds `v200sai0`, `vc2vtai0.xvg` holds `vsaxoai0` (240SX / C2 opponents silent, original).
  Test `every_ai_engine_loads_and_plays`. Core sends LoadAiEngine now: the LA trace `research/audio/rpm_trace.txt`
  plays `v300zai0/vceliai0/vsilvai0` once each (the older `ai_engines_trace.txt` with `!FAIL` predates that).
- **Engine rpm (checked against F3_D, 2026-11-05)**: core feeds `engine_rpm` = `PhysicsWorld::engine_rpm`
  (0x001B6F10, 1000..6000 with `set_audio_max_rpm(6000)`). UpdateRpm 0x000523F0 (`(float)(int)rpm * 1.25`, cap
  10000, params 17/26), UpdatePlayerEngine 0x0004C400 (gear 0x001B74A0, load = history +36, `gear > 100` -> load 0,
  Update before SetRpm, GEAR_DISENGAGE on a higher gear / GEAR_ENGAGE on a lower one), RevBands Update 0x0004CD90 /
  SetRpm 0x0004CBF0 / SetLoad 0x0004D2C0 / band SetPitch 0x0004D1E0 / SetVolume 0x0004D230 and UpdateAiCars' pitch
  (`(float)(int)rpm + rand%100`, -1000..50 cents over 1000..dword_6D4D98, vol (0.35t+0.2)·0.45/0.25) all re-read:
  the port matches, no audio change needed. LA smoke (`research/audio/rpm_trace.txt`, `VELOCITY_AUDIO_TRACE_ENGINE=1`):
  player rpm 1240..5294 through gears 0,1,102,2,103,3 (x1.25 -> rbd rpm 1550..6618 of the loaded rbd's range
  1000..~7800), band cents follow the smoothed norm, shift states (>100) put every band on the off-load volume,
  gear sounds 4 disengage / 3 engage as the sequence predicts; ai loops 1000..~6600 rpm (startline revs above 6000)
  -> -1000..+174 cents. Clip `research/video/audio_rpm.mp4` (38 s, with sound).
  Core fixed the throttle read (heap history block), the rpm-before-shift order and dropped the traffic
  `engine_rpm` call (core.md "Requests from audio", done). The traffic engine pitch block is dead in the original
  (slot +88 is never filled, `research/audio/traffic88.txt`) and not ported. Earlier traces with `thr 0.00` predate this.
- Tokyo / Tokyo2 have no Audio dir: no city ambience, no env emitters (data). LA/London: city_ambience + aee
  emitters (London river/boat; `xa_env_london_pub_01` fails = data).
- **Voice id bug fixed** (`xbox/voice.rs`): the 3d/2d/stereo pools each started ids at the same index/generation,
  so one id named voices in two pools; frees/stops/buffer writes hit the wrong voice (seen: in-race music cut ~1.5 s
  after the start when another voice was set up). Now one id space (`ids`), per-pool limits via `free_slots`.
- Review smokes (all `VELOCITY_AUDIO_TRACE`, traces in `research/audio/review/*.txt`): LA, London (+HIT/ZONE),
  Tokyo, LA wet, attract (wet LA: rain, wet skids, glass -> impact_car_physics) all as above; FE: cursor/accept/back
  + FEMusicTrack01 (all 11 PlayGui* callers are ported by frontend). Zone heartbeat not starting is original
  (graphics: 0x00162D10 reads the level after 0x00154BD0 raised it to 0.25, so StartZone only fires on a paused frame
  at level 0). Clip: `research/video/audio_review.mp4`.
- Review 2 (2026-11-05, exe `target/au`, runner `research/audio/review2/run.sh hit|tb|street|career`, traces +
  logs beside it; the trace's `cmds:` now prints `Impact[event dDIST sSTRENGTH c]`):
  - object impact (London, `SMOKE_DIRECT=1 SMOKE_AREA=2 SMOKE_HIT=1 SMOKE_DRIVE=22`; 12 s stops 5 m short):
    imp_metal_sign + impact_car_physics. LADown Race02: far impacts (objects knocked by ai, 300..730 m) are dropped
    by the 5 m check; near car-layer calls play impact_car_physics once per history window; glass objects play
    imp_glass_vlrg. PlayPhysicsImpact 0x00052E90 re-read: unchanged.
  - timeline sounds (Race02 street route, `street`): TimelineSound x2 -> asp_carpark_explosion_inside_start /
    _inside (ascarpks), with TunnelReverb, PauseMusic, CityAmbience, FadeToVolumePreset. TimelineSound 0x0017EA00 /
    stops 0x0017EBD0 / 0x0017EB80 / emitter 0x0017E7F0 re-read: unchanged. Only LADown Race02 and London Race05
    timelines hold sound objects.
  - career (`SMOKE_CAREER=1`; SMOKE_DIRECT turns it into a quick race): FE gui sounds, StopMusicEvent, two LADown
    sprint races, SetLists on return. race_mode = PickList's `raceDef +12 -> +136` (0x0004F1A0) = track area, which
    core sends for every race (all go through the jump selection). Second race: city_ambience and passbys fail in the
    frames between race Init and LoadLevelSounds — VsAudio::Update is only called from CFrontendManager::Update
    0x002016DD / InGameUpdate 0x000B7EE7 / Pause / Resume, never during the load. Resource `AudioUpdateGate(bool)`
    (default true) skips the update; core asked to clear it during the race load (core.md "Requests from audio").
  - remaining `!FAIL`s are data/original: TB_ATMOS, glass 0x3B5B1A82, numeric 979, LADown aee `BeverlyHills3` event
    0xF3089C4F (in no xvg).

**Debug-menu tweaks** (`audio_tweaks`, 2026-11-06): the 7 AddTweaks (0x48410 ImpactStuff, 0x48530 Debug, 0x48600
ENVFX, 0x4B380 Slowdown, 0x4CB10 Engine, 0x4D7D0 Music, 0x4FD50 VolTweaks/SirenTweaks/Suspension/OtherCarImpacts/AIRevs/
PlayerPositionVols; 12 systems, 36 tweaks) as call lists + value slots (graphics' scheme; velocity_game depends on
this crate, so core replays them: core.md "Requests from audio (audio AddTweaks ...)", open). Only caller:
VsAudio::AddTweaks 0x484D0 (call order `ADD_TWEAKS_ORDER`) from VsAudio::Init 0x48362, skipped for the FE Init (+104);
Init's one caller is GameManager::CommonInit 0xB5169. Live: impact_frames, debug_text (+484, bss 0; only reader
Update 0x48F4C, not drawn), envfx_on, doppler, speed_of_sound, settings.engine (0x684F94) / settings.music (0x684FB4,
both rewritten from the options by Update 0x48CC1 / 0x48CA1 as in the original), tft_music_mult, `vehicle_tweaks.*`;
Slowdown Factor/Target (0x684F00/04) have no reader in the binary (slot only). `sync_audio_tweaks` (First) pushes
edits / pulls live values. Tests `audio_tweaks::tests` vs `research/audio/tweak_regs.tsv` (`tools/ida/audio_tweak_regs.py`;
xrefs `tweak_xrefs.txt`, callers `tweak_callers.txt`).

**Requests from core / graphics**: all done (see "AI racer engines", "Engine rpm"). Core sends `ai` = opponents in
order, `traffic` = all 14 manager slots. Open request to core: `AudioUpdateGate` during the race load (Review 2).

**Bink movie sound (done; was frontend's request)** — `audio/xbox/bink_sound.rs` = RAD Bink's xbox DirectSound
driver (BINK 0x00543EF0..0x00544320, dump `research/audio/bink_snd.c`, script `tools/ida/audio_bink_snd.py`).
- API: resource `velocity_audio::MovieAudioOut { rate, channels, pcm, volume, paused, open, generation }` (frontend
  re-exports it as `movie::MovieAudioOut`): `open(rate, ch)` at BinkOpen, push s16 into `pcm`, `close()` at BinkClose.
  System `movie_sound` (AudioSystems::Update, after `update`) opens / closes the driver per `generation`, applies
  volume / pause and moves `pcm` into the mixer's `StreamVoice` (`Mixer::stream`: one extra buffer outside the voice
  pool and the category volumes, linear resample to 48 kHz, silence on underrun). `AudioCommand::StopMusicEvent`.
- Wiring (movie.rs, minimal): `MovieCommand::Play` = GameManager__BE9F0: `StopMusicEvent` (a stop, not a duck: the
  original does not restore the music after the movie) and `volume = 1.0 * 32768.0` (GameManager__BFFF0 0x000BFFF0
  writes VsMovie +124 = v * 32768; TFT movies use `flt_6F9174`), `open` on each opened movie, `close` in finish.
- Driver: BinkOpen installs 0x00544280 when no sound system is set; open 0x00543FF0 (freq, bits, chans): buffer =
  (bytes/s * 600 / 1000 + 0x3FF) & ~0x3FF, chunk 50 ms rounded to 256, volume / pan 0x7FFF, 6 mixbins at 0 mB,
  silence fill. Table at track +0: Ready 0x5438F0, Lock 0x543AA0, Unlock 0x543BE0, SetVolume 0x543C70 (clamp
  0x7FFF), SetPan 0x543CA0 (clamp 0x10000; stored, never applied), Pause 0x543E60, OnOff 0x543DC0, Close 0x543F70,
  0x543CD0 / 0x543D10 mixbin setters. Volume apply 0x5435E0: 0 -> -10000 mB, else
  fistp(log10(v * 3.0518509e-5 [0x38000100]) * 3333.3333 [0x45505555]) clamped >= -10000 (gain = (v/32767)^(5/3));
  VsMovie passes 32768 -> 0 mB. BinkSetVolume 0x00540D30 = track +12; BinkPause 0x005401E0 = track +20, called only
  by BinkClose 0x00541C60. 0x0053E690 is BinkSetSoundTrack(count, ids): VsMovie calls (0, 0) for mode 1 = no sound.
- **Timing**: BinkWait 0x0053FFB0 / DoFrame skip are on the millisecond clock 0x005428A0 (QueryPerformanceCounter),
  frame due = start + (frame - start_frame) * 1000 * den / num * (+108 = 0x10000) >> 16; with sound the clock is not
  resynced on lateness (+808 / +812 catch-up) and the first frame waits for the sound start (+648, BinkOpen). The
  sound buffer just plays the decoded ring in real time. So video is **not** slaved to the audio position; the
  frontend's real-clock `frame / fps` is the original behaviour (frontend.md's "audio-synced" stand-in note is wrong).
- Tests: `bink_sound::tests` (volume curve, 24 kHz stream resampled + stopped on close). Clip
  `research/video/fe_movie_sound.mp4` (`--smoke-test`, `SMOKE_RECORD`, `VELOCITY_MOVIE=In001,In002,In003
  VELOCITY_MOVIE_EXIT=35`, exe `target/au`): movie sound throughout, the clip's audio correlates with the decoded
  `bink_in003.wav` at In003's start (5.0 s). The dev boot path sends StopMusicEvent before the FE starts its music,
  so FEMusicTrack01 plays under the movies there (trace `research/audio/fe_movie_trace.txt`); the attract path
  (0x000B3620 -> BE9F0) stops a playing track as the original.

**DONE 2026-11-06**: `AudioCommand::{StartLoopEvent210, StopLoopEvent210, FadeMusic{to,env,time}}` (plugin.rs, each logs
`audio: ...` at info); frontend's lib.rs drains `Feck2.music` into them; `fecapture` now adds `AudioPlugin` + a frontend
`Init` (log `research/audio/hangout_cmds.log`: Start + 3 FadeMusic seen in `fecapture scene`). Originals re-read:
0x4A8D0 plays event 210 on the player emitter (+8) into slot 0x6D4538 if empty; FadeMusic 0x4DA80 stores `to` in
dword_684FB8 (write-only), fades the music sample, env == 0.1 -> environment 6 else 3 (port matches). Whether the FE
`Init` creates a player emitter (so loop 210 is audible) is unchecked; event 210 is listed under "dead" numeric events.

**Requests from frontend (hangout "THE SCENE" music, 2026-11-06; done, see above)**: FEHangoutScreen Start 0x00219300 / Leave 0x00219920
/ ChangeCharacter 0x0021B0D0 call `VsAudio::StartLoopEvent210` (0x0004A8D0), `StopLoopEvent210` (0x0004A920) and
`VsAudio::FadeMusic(to, env, time)` (0x0004DA80; to/env = the character's Chars.txt volume / reverb, time 1.0; Leave to
another screen: (1.0, 0.1, 1.0)). Please add `AudioCommand::{StartLoopEvent210, StopLoopEvent210, FadeMusic { to: f32,
env: f32, time: f32 }}` mapped onto `VsAudio::start_loop_event_210` / `stop_loop_event_210` / `fade_music`. The
frontend already queues them as `velocity_frontend::feck2::feck2_manager::FeMusicCall` (same fields) in `Feck2.music`
and drops them in `lib.rs` (search "no AudioCommand yet"); tell frontend when the variants exist. Note the FE has
no player emitter for loop 210 unless `VsAudio::Init` (front end) makes one — check what 0x0004A8D0 plays in the FE.

**Device-path check (2026-11-08, user report "sound barely works / not at all" on dist/velocity.exe)** — not
reproduced on this PC. Default endpoint = Focusrite USB "Speakers", shared mix 2 ch 44.1 kHz f32; rodio 0.22 opens it
with a fixed 2048-frame buffer and resamples the 48 kHz mix. Measured (release and debug, `--smoke-test` and a plain
boot, vsync and `--fps 0`, 28..85 fps): device pulls 47.6-48.6 k mix frames/s every second, longest gap between pulls
~20.5 ms (one wasapi period), mixer lock wait 0.00 ms, mix rms FE -27..-37 dB (music 0.2 = GamePlayer 0xC1930
default 0x3E4CCCCD, original), race -12..-24 dB; the endpoint loopback (`examples/loopback.rs`) receives 88200
samples/s at the same rms one second later. Mix level is independent of the update rate (test
`front_end_mix_at_any_rate`, 30..1000 Hz identical to 0.1 dB); rodio Player -> device mixer at 44.1/48 kHz keeps the
level (test `device_chain_keeps_level`). The device is pulled by cpal's callback thread, never by the frame or
GameTick. So no regression in the candidate commits. Untested: anything specific to the user's session (another
default device / device switched or invalidated mid-run: bevy_audio opens the default device once at startup and
never reopens; on open failure rodio silently falls back to another endpoint, e.g. HDMI or the USB headset).
Tools: `VELOCITY_AUDIO_DIAG=<file>` (MixerStream, per-second frames / max gap / lock wait / rms) with
`VELOCITY_AUDIO_DIAG_MUTE=1` (silence to the device) or `VELOCITY_AUDIO_DIAG_GAIN=<g>`;
`cargo run -p velocity_audio --example loopback -- <secs>` = default endpoint loopback rms per second (wasapi sends
no packets while nothing plays).

