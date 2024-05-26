mod idle_game;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy_voxel_world::prelude::*;
use std::sync::Arc;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use crate::idle_game::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// Declare materials as consts for convenience
const SNOWY_BRICK: u8 = 0;
const FULL_BRICK: u8 = 1;
const GRASS: u8 = 2;

#[derive(Resource, Clone, Default)]
struct MyMainWorld;

impl VoxelWorldConfig for MyMainWorld {
    fn texture_index_mapper(&self) -> Arc<dyn Fn(u8) -> [u32; 3] + Send + Sync> {
        Arc::new(|vox_mat: u8| match vox_mat {
            SNOWY_BRICK => [0, 1, 2],
            FULL_BRICK => [2, 2, 2],
            GRASS | _ => [3, 3, 3],
        })
    }

    fn voxel_texture(&self) -> Option<(String, u32)> {
        Some(("example_voxel_texture.png".into(), 4))
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // We can specify a custom texture when initializing the plugin.
        // This should just be a path to an image in your assets folder.
        .add_plugins(VoxelWorldPlugin::with_config(MyMainWorld))
        .add_systems(Startup, (setup, create_voxel_scene))
        .add_systems(Update, (update_cursor_cube, mouse_button_input))

        .add_systems(Startup, setup_fps_counter)
        .add_systems(Update, (
            fps_text_update_system,
            fps_counter_showhide,
            spawn_particles_from_mouse_system,
            turn_into_voxel_system.before(velocity_system).before(gravity_system),
            velocity_system,
            gravity_system
        ))
        .run();
}

#[derive(Component)]
struct CursorCube {
    voxel_pos: IVec3,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Cursor cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid {
                half_size: Vec3::splat(0.5),
            })),
            material: materials.add(Color::rgba_u8(124, 144, 255, 128)),
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            ..default()
        },
        CursorCube {
            voxel_pos: IVec3::new(0, -10, 0),
        },
    ));

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // This tells bevy_voxel_world to use this cameras transform to calculate spawning area
        VoxelWorldCamera::<MyMainWorld>::default(),
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn create_voxel_scene(mut voxel_world: VoxelWorld<MyMainWorld>) {
    // Then we can use the `u8` consts to specify the type of voxel

    // 20 by 20 floor
    for x in -10..10 {
        for z in -10..10 {
            voxel_world.set_voxel(IVec3::new(x, -1, z), WorldVoxel::Solid(GRASS));
            // Grassy floor
        }
    }

    // Some bricks
    voxel_world.set_voxel(IVec3::new(0, 0, 0), WorldVoxel::Solid(SNOWY_BRICK));
    voxel_world.set_voxel(IVec3::new(1, 0, 0), WorldVoxel::Solid(SNOWY_BRICK));
    voxel_world.set_voxel(IVec3::new(0, 0, 1), WorldVoxel::Solid(SNOWY_BRICK));
    voxel_world.set_voxel(IVec3::new(0, 0, -1), WorldVoxel::Solid(SNOWY_BRICK));
    voxel_world.set_voxel(IVec3::new(-1, 0, 0), WorldVoxel::Solid(FULL_BRICK));
    voxel_world.set_voxel(IVec3::new(-2, 0, 0), WorldVoxel::Solid(FULL_BRICK));
    voxel_world.set_voxel(IVec3::new(-1, 1, 0), WorldVoxel::Solid(SNOWY_BRICK));
    voxel_world.set_voxel(IVec3::new(-2, 1, 0), WorldVoxel::Solid(SNOWY_BRICK));
    voxel_world.set_voxel(IVec3::new(0, 1, 0), WorldVoxel::Solid(SNOWY_BRICK));
}

fn update_cursor_cube(
    voxel_world_raycast: VoxelWorld<MyMainWorld>,
    camera_info: Query<(&Camera, &GlobalTransform), With<VoxelWorldCamera<MyMainWorld>>>,
    mut cursor_evr: EventReader<CursorMoved>,
    mut cursor_cube: Query<(&mut Transform, &mut CursorCube)>,
) {
    for ev in cursor_evr.read() {
        // Get a ray from the cursor position into the world
        let (camera, cam_gtf) = camera_info.single();
        let Some(ray) = camera.viewport_to_world(cam_gtf, ev.position) else {
            return;
        };

        if let Some(result) = voxel_world_raycast.raycast(ray, &|(_pos, _vox)| true) {
            let (mut transform, mut cursor_cube) = cursor_cube.single_mut();
            // Move the cursor cube to the position of the voxel we hit
            let voxel_pos = result.position + result.normal;
            transform.translation = voxel_pos + Vec3::new(0.5, 0.5, 0.5);
            cursor_cube.voxel_pos = voxel_pos.as_ivec3();
        }
    }
}

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut voxel_world: VoxelWorld<MyMainWorld>,
    cursor_cube: Query<&CursorCube>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let vox_pos = cursor_cube.single().voxel_pos;
        voxel_world.set_voxel(vox_pos, WorldVoxel::Solid(FULL_BRICK));
    }
}


// also get time
fn turn_into_voxel_system(
    particle_query: Query<(Entity, &Particle, &Transform, &SimpleVelocity)>,
    mut commands: Commands,
    mut voxel_world: VoxelWorld<MyMainWorld>,
    time: Res<Time>,
){
    let delta_time = time.delta_seconds();
    for (entity, particle, transform, simple_velocity) in particle_query.iter(){
        let pos = transform.translation;
        let next_pos = pos + simple_velocity.velocity * delta_time;
        let ray = Ray3d::new(pos, next_pos - pos);
        let mut hit_result = voxel_world.raycast(ray, &|(_pos, vox)| vox.is_solid());
        
        if let Some(vox_raycast_result) = hit_result{
            let hit_pos = vox_raycast_result.position;
            let distance = (hit_pos - pos).length();
            if distance > ((next_pos - pos)*3.0).length(){
                // cancel
                continue;
            }
            let hit_pos = hit_pos.as_ivec3() + IVec3{y:1, ..IVec3::ZERO};
            voxel_world.set_voxel(hit_pos, WorldVoxel::Solid(particle.tile_type as u8));
            // log the hit position
            println!("Hit voxel at {:?}", hit_pos);
            commands.entity(entity).despawn();
        }
    }
}

// get camera pos, mouse direction, and spawn particles, dont check for mouse input, just use the cursor position
fn spawn_particles_from_mouse_system(
    cursor_cube: Query<(&CursorCube, &Transform)>,
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform), With<VoxelWorldCamera<MyMainWorld>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut frame_counter: Local<u32>,
){
    // every 10 frame
    *frame_counter += 1;
    if *frame_counter % 10 != 0{
        return;
    }
    
    let cursor_cube_pos = cursor_cube.single().1.translation;
    for (_camera, camera_transform) in camera_query.iter(){
        let cursor_pos = cursor_cube_pos;
        let camera_pos = camera_transform.translation();
        let direction = cursor_pos - camera_pos;
        let direction = direction.normalize();
        let particle = Particle{
            tile_type: TileType::Stone,
        };
        let simple_velocity = SimpleVelocity{
            //velocity: direction * 10.0,
            velocity: direction * 15.0,
        };
        let pbundle = PbrBundle{
            mesh: meshes.add(Mesh::from(shape::Cube{size: 1.0})),
            material: materials.add(Color::rgb(0.0, 0.0, 1.0)),
            // up 2 units
            transform: Transform::from_translation(camera_pos + Vec3::new(0.0, 2.0, 0.0)),
            ..Default::default()
        };
        commands.spawn((particle, simple_velocity, pbundle));
    }
}
    


//-----------------------------------------------------
    



/// Marker to find the container entity so we can show/hide the FPS counter
#[derive(Component)]
struct FpsRoot;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct FpsText;

fn setup_fps_counter(
    mut commands: Commands,
) {
    // create our UI root node
    // this is the wrapper/container for the text
    let root = commands.spawn((
        FpsRoot,
        NodeBundle {
            // give it a dark background for readability
            background_color: BackgroundColor(Color::BLACK.with_a(0.5)),
            // make it "always on top" by setting the Z index to maximum
            // we want it to be displayed over all other UI
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                position_type: PositionType::Absolute,
                // position it at the top-right corner
                // 1% away from the top window edge
                right: Val::Percent(1.),
                top: Val::Percent(1.),
                // set bottom/left to Auto, so it can be
                // automatically sized depending on the text
                bottom: Val::Auto,
                left: Val::Auto,
                // give it some padding for readability
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    // create our text
    let text_fps = commands.spawn((
        FpsText,
        TextBundle {
            // use two sections, so it is easy to update just the number
            text: Text::from_sections([
                TextSection {
                    value: "FPS: ".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        // if you want to use your game's font asset,
                        // uncomment this and provide the handle:
                        // font: my_font_handle
                        ..default()
                    }
                },
                TextSection {
                    value: " N/A".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        // if you want to use your game's font asset,
                        // uncomment this and provide the handle:
                        // font: my_font_handle
                        ..default()
                    }
                },
            ]),
            ..Default::default()
        },
    )).id();
    commands.entity(root).push_children(&[text_fps]);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        // try to get a "smoothed" FPS value from Bevy
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            // Format the number as to leave space for 4 digits, just in case,
            // right-aligned and rounded. This helps readability when the
            // number changes rapidly.
            text.sections[1].value = format!("{value:>4.0}");

            // Let's make it extra fancy by changing the color of the
            // text according to the FPS value:
            text.sections[1].style.color = if value >= 120.0 {
                // Above 120 FPS, use green color
                Color::rgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                // Between 60-120 FPS, gradually transition from yellow to green
                Color::rgb(
                    (1.0 - (value - 60.0) / (120.0 - 60.0)) as f32,
                    1.0,
                    0.0,
                )
            } else if value >= 30.0 {
                // Between 30-60 FPS, gradually transition from red to yellow
                Color::rgb(
                    1.0,
                    ((value - 30.0) / (60.0 - 30.0)) as f32,
                    0.0,
                )
            } else {
                // Below 30 FPS, use red color
                Color::rgb(1.0, 0.0, 0.0)
            }
        } else {
            // display "N/A" if we can't get a FPS measurement
            // add an extra space to preserve alignment
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

/// Toggle the FPS counter when pressing F12
fn fps_counter_showhide(
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}