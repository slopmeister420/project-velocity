//! draws core's debug menus (`velocity_game::game_menu_manager`) the way the original does: OSD text items in the
//! debug font (FontHelper id 12) on an OSD manager standing in for the game OSD `dword_6FD1FC`, rasterised into a
//! 640x480 overlay above the race / front end overlays.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use velocity_frontend::font_helper::font_helper::{CFontHelper, FONT_DEBUG};
use velocity_frontend::osd::osd_manager::COSDManager;
use velocity_frontend::render2d::{DrawList, TextureStore};
use velocity_frontend::string_manager::string_manager::CStringManager;
use velocity_frontend::{DISPLAY_H, DISPLAY_W};
use velocity_game::game_menu_manager::game_menu_manager::{CDebugMenuManager, DebugMenuText};

/// above the FE / HUD overlays (10, 11)
const DEBUG_MENU_ORDER: isize = 40;

struct DebugOsd {
    fonts: CFontHelper,
    textures: TextureStore,
    strings: CStringManager,
}

#[derive(Resource, Default)]
struct DebugMenuView {
    osd: Option<DebugOsd>,
    image: Option<Handle<Image>>,
    root: Option<(Entity, Entity)>,
    last: Vec<DebugMenuText>,
    time: f32,
}

pub fn build(app: &mut App) {
    app.init_resource::<DebugMenuView>().add_systems(PostUpdate, draw_debug_menu);
}

fn draw_debug_menu(
    mut commands: Commands,
    mut view: ResMut<DebugMenuView>,
    dm: Res<CDebugMenuManager>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
) {
    let list = dm.draw();
    if list == view.last {
        return;
    }
    view.last = list.clone();
    view.time += time.delta_secs();
    if view.image.is_none() {
        let img = Image::new_fill(
            Extent3d { width: DISPLAY_W, height: DISPLAY_H, depth_or_array_layers: 1 },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        view.image = Some(images.add(img));
    }
    let image = view.image.clone().unwrap();
    if list.is_empty() {
        if let Some((cam, root)) = view.root.take() {
            commands.entity(root).despawn();
            commands.entity(cam).despawn();
        }
        return;
    }
    if view.root.is_none() {
        let cam = commands
            .spawn((Camera2d, Camera { order: DEBUG_MENU_ORDER, clear_color: ClearColorConfig::None, ..default() }))
            .id();
        let root = commands
            .spawn((
                Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                ImageNode::new(image.clone()),
                UiTargetCamera(cam),
            ))
            .id();
        view.root = Some((cam, root));
    }
    let o = view.osd.get_or_insert_with(|| {
        let mut textures = TextureStore::default();
        let mut fonts = CFontHelper::default();
        fonts.load_game_fonts(&mut textures);
        DebugOsd { fonts, textures, strings: CStringManager::new() }
    });
    let mut osd = COSDManager::default();
    for t in &list {
        osd.create_text_item(&o.fonts, &o.textures, &t.text, FONT_DEBUG, t.x, t.y, t.layer, -1.0, t.flags, t.mask);
    }
    let now = view.time;
    let o = view.osd.as_ref().unwrap();
    osd.update(now, &o.fonts, &o.strings, &o.textures);
    let mut dl = DrawList::default();
    osd.render(&o.fonts, &o.strings, &o.textures, &mut dl);
    let frame = dl.rasterise_overlay(&o.textures, DISPLAY_W, DISPLAY_H);
    if let Some(mut img) = images.get_mut(&image) {
        img.data = Some(frame.data);
    }
}
