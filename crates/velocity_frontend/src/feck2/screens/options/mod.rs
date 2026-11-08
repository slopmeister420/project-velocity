//! the options screens (`FECK2CreateOptionsScreen` and its sub-screens). They have no `__FILE__` of their own: the
//! code sits between the profile and main menu screens in the idb's "FEOnlineSignInScreen.cpp" range
//! (0x00230AC0..0x0023A4D0). Each uses the overlay menu for its items and reads the selection back.
//!
//! not ported: `OPTION_TWEAK` (the in-game tweak menu); selecting it goes nowhere. TUTORIALS opens
//! `common::fe_tips_screen`.

pub mod fe_audio_options_screen;
pub mod fe_config_options_screen;
pub mod fe_language_options_screen;
pub mod fe_options_screen;
pub mod fe_pc_options_screen;
pub mod fe_plate_customization_screen;
pub mod fe_profile_options_screen;
pub mod fe_video_options_screen;

use crate::feck2::feck2_manager::Feck2;
use crate::feck2::screens::common::fe_2d_overlay::{self as overlay, CFE2DOverlay};

/// run `f` on the overlay taken out of the frontend.
pub(crate) fn ov<R: Default>(fe: &mut Feck2, f: impl FnOnce(&mut CFE2DOverlay, &mut Feck2) -> R) -> R {
    let Some(mut o) = fe.overlay.take() else { return R::default() };
    let r = f(&mut o, fe);
    fe.overlay = Some(o);
    r
}

/// the shared start of every options screen's Activate: reset the overlay but keep the 2D background as it was,
/// hide the options / online prompts.
pub(crate) fn reset_keep_background(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    let bg = fe.el(o.background_2d).is_visible();
    overlay::reset(o, fe);
    fe.el(o.background_2d).set_visible(bg);
    overlay::set_hide_options(o, fe, true);
}

/// the shared Enter: attach the overlay, park the camera where it is going.
pub(crate) fn enter(fe: &mut Feck2, screen: crate::feck2::feck2_object::ScreenId) {
    ov(fe, |o, fe| overlay::attach_to_screen(o, fe, Some(screen)));
    // FE3DCamera::SetScreen(0) (0x0026FB60) then 0x00271E80
    fe.foyer.snap_to_target();
}
