//! wgpu 29 workaround: an acquired-but-unpresented swapchain texture must not take the process down with the
//! swapchain-semaphore panic (wgpu-hal-29.0.4 `vulkan/swapchain/native.rs:359` "Trying to destroy a
//! SwapchainAcquireSemaphore that is still in use by a SurfaceTexture").
//!
//! wgpu-core `Surface::drop` (instance.rs:666) runs `unconfigure` -> `NativeSwapchain::release_resources` without
//! first releasing the surface's `acquired_texture`; while that texture's hal SurfaceTexture is alive it holds an Arc
//! of the acquire semaphore and `Arc::into_inner(..).expect(..)` panics (unchanged in 30.0.1). Two ways to get there:
//! - **device lost**: `Surface::present` / `discard` bail out on `check_is_valid` before taking `acquired_texture`;
//!   bevy's default `RenderErrorHandler` sends AppExit and the teardown drops `WindowSurfaces`.
//! - **any panic while a frame is acquired**: `wgpu::SurfaceTexture::drop` skips `discard` while the thread is
//!   panicking, so the unwind drops the surface with the texture held and the second panic aborts (exit 127),
//!   hiding the first one (`research/fe_smoke_career2.log`).
//! `Texture::destroy` on a surface texture snatches and drops the hal texture without a device check (wgpu-core
//! resource.rs:1451), which frees the semaphore Arcs, so both paths destroy the held swapchain textures first.
//! Acquire errors themselves are already survived by bevy's `prepare_windows`: Outdated -> reconfigure + retry,
//! Timeout / Occluded / Lost -> log and skip the frame.

use bevy::prelude::*;
use bevy::render::error_handler::{RenderError, RenderErrorHandler, RenderErrorPolicy};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::{prepare_windows, ExtractedWindows};
use bevy::render::{Render, RenderApp, RenderSystems};
use std::sync::Mutex;

pub struct SwapchainGuardPlugin;

impl Plugin for SwapchainGuardPlugin {
    fn build(&self, app: &mut App) {
        // VELOCITY_GFX_SWAPCHAIN_GUARD=0 keeps bevy's default handler and no drop guard (A/B)
        let guard = std::env::var("VELOCITY_GFX_SWAPCHAIN_GUARD").as_deref() != Ok("0");
        if guard {
            app.insert_resource(RenderErrorHandler(handle_render_error));
        }
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        if guard {
            // inserted at build, i.e. before RenderStartup inits `WindowSurfaces`: resources drop in insertion
            // order, so this one releases the textures before the surfaces are dropped
            render_app.insert_resource(SwapchainDropGuard::default());
            render_app.add_systems(Render, track_swapchain_textures.after(prepare_windows).in_set(RenderSystems::PrepareViews));
        }
        // debug: VELOCITY_GFX_LOSE_DEVICE=<render frame> destroys the wgpu device there (simulated device loss)
        if let Some(frame) = std::env::var("VELOCITY_GFX_LOSE_DEVICE").ok().and_then(|v| v.parse::<u32>().ok()) {
            render_app.insert_resource(LoseDeviceAt(frame));
            render_app.add_systems(Render, lose_device.in_set(RenderSystems::Cleanup));
        }
        // debug: VELOCITY_GFX_PANIC_AT=<render frame> panics in the render world with the frame acquired
        if let Some(frame) = std::env::var("VELOCITY_GFX_PANIC_AT").ok().and_then(|v| v.parse::<u32>().ok()) {
            render_app.insert_resource(PanicAt(frame));
            render_app.add_systems(Render, panic_at.after(prepare_windows).in_set(RenderSystems::PrepareViews));
        }
    }
}

/// bevy's default policy (log, AppExit::error, stop rendering), after releasing the swapchain textures.
fn handle_render_error(error: &RenderError, main_world: &mut World, render_world: &mut World) -> RenderErrorPolicy {
    let released = release_swapchain_textures(render_world);
    error!("Quitting the application due to {:?} RenderError ({released} swapchain textures released): {}", error.ty, error.description);
    main_world.write_message(AppExit::error());
    RenderErrorPolicy::StopRendering
}

/// destroys (= hal-releases) every window's current swapchain texture; returns how many were held.
pub fn release_swapchain_textures(render_world: &mut World) -> usize {
    let Some(mut windows) = render_world.get_resource_mut::<ExtractedWindows>() else { return 0 };
    let mut n = 0;
    for w in windows.windows.values_mut() {
        if let Some(st) = w.swap_chain_texture.take() {
            st.texture.destroy();
            // its drop would discard, which is a fatal wgpu error on a lost device
            std::mem::forget(st);
            n += 1;
        }
        if let Some(view) = w.swap_chain_texture_view.take() {
            // presented-but-failed (lost device) textures are only reachable through the view
            view.texture().destroy();
            n += 1;
        }
    }
    n
}

/// clones of this frame's swapchain textures; destroyed on drop (no-op for presented ones, whose hal texture is
/// already snatched by present).
#[derive(Resource, Default)]
struct SwapchainDropGuard(Mutex<Vec<bevy::render::render_resource::Texture>>);

impl Drop for SwapchainDropGuard {
    fn drop(&mut self) {
        let textures = std::mem::take(self.0.get_mut().unwrap_or_else(|e| e.into_inner()));
        if std::thread::panicking() {
            for t in &textures {
                t.destroy();
            }
            if !textures.is_empty() {
                eprintln!("swapchain guard: released {} swapchain texture(s) during unwind", textures.len());
            }
        }
    }
}

fn track_swapchain_textures(windows: Res<ExtractedWindows>, guard: Res<SwapchainDropGuard>) {
    let mut held = guard.0.lock().unwrap_or_else(|e| e.into_inner());
    held.clear();
    for w in windows.windows.values() {
        if let Some(st) = &w.swap_chain_texture {
            held.push(st.texture.clone().into());
        }
    }
}

#[derive(Resource)]
struct LoseDeviceAt(u32);

fn lose_device(at: Res<LoseDeviceAt>, device: Res<RenderDevice>, mut frame: Local<u32>) {
    *frame += 1;
    if *frame == at.0 {
        warn!("VELOCITY_GFX_LOSE_DEVICE: destroying the wgpu device at render frame {}", *frame);
        device.wgpu_device().destroy();
    }
}

#[derive(Resource)]
struct PanicAt(u32);

fn panic_at(at: Res<PanicAt>, mut frame: Local<u32>) {
    *frame += 1;
    if *frame == at.0 {
        panic!("VELOCITY_GFX_PANIC_AT: render-world panic at render frame {}", *frame);
    }
}
