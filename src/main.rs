use std::f32::consts::PI;
use std::time::Duration;

use bevy::{
    animation::{animate_targets, RepeatAnimation},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};

const FOX_GLB: &str = "assets/models/animated/Fox.glb"; //manually set this to the fixed location of Fox.glbt

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, keyboard_animation_control)
        .add_systems(Update, follow_fox)
        .add_systems(Update, move_fox)
        .run();
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

#[derive(Component)]
struct Fox; // Marker component for the fox

#[derive(Component)]
struct MoveSpeed(f32); // Component to store the speed of the fox

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips(
            [
                GltfAssetLabel::Animation(2).from_asset(FOX_GLB),
                GltfAssetLabel::Animation(1).from_asset(FOX_GLB),
                GltfAssetLabel::Animation(0).from_asset(FOX_GLB),
            ]
                .into_iter()
                .map(|path| asset_server.load(path)),
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        animations,
        graph: graph.clone(),
    });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 200.0, -300.0) // Start behind the fox
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FollowFox, // Add marker component to identify the following camera
    ));

    // Plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
            .into(),
        ..default()
    });

    // Fox
    commands.spawn((
        SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(FOX_GLB)),
            ..default()
        },
        Fox,      // Add the Fox component to identify this entity
        MoveSpeed(5.0), // Add the MoveSpeed component to control movement speed
    ));

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - digit 1 / 3 / 5: play the animation <digit> times");
    println!("  - L: loop the animation forever");
    println!("  - return: change animation");
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>, // Correctly import Input from Bevy
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for (mut player, mut transitions) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };

        if keyboard_input.just_pressed(KeyCode::Space) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            if playing_animation.is_paused() {
                playing_animation.resume();
            } else {
                playing_animation.pause();
            }
        }

        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 1.2);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 0.8);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed - 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed + 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Enter) {
            *current_animation = (*current_animation + 1) % animations.animations.len();

            transitions
                .play(
                    &mut player,
                    animations.animations[*current_animation],
                    Duration::from_millis(250),
                )
                .repeat();
        }

        if keyboard_input.just_pressed(KeyCode::Digit1) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(1))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::Digit3) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(3))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::Digit5) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(5))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::KeyL) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation.set_repeat(RepeatAnimation::Forever);
        }
    }
}

#[derive(Component)]
struct FollowFox; // Marker component for camera

fn follow_fox(
    fox_query: Query<&Transform, With<Fox>>,
    mut camera_query: Query<&mut Transform, (With<FollowFox>, Without<Fox>)>,
) {
    if let Ok(fox_transform) = fox_query.get_single() {
        for mut camera_transform in &mut camera_query {
            // Calculate the position behind the fox
            let fox_forward = fox_transform.rotation.mul_vec3(Vec3::Z);
            let camera_offset = Vec3::new(0.0, 200.0, -300.0);
            let camera_position = fox_transform.translation + fox_forward * camera_offset.z;

            // Update camera position and look at the fox
            camera_transform.translation = Vec3::new(
                camera_position.x,
                fox_transform.translation.y + camera_offset.y,
                camera_position.z,
            );

            camera_transform.look_at(fox_transform.translation, Vec3::Y);
        }
    }
}

fn move_fox(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>, // Correctly import Input from Bevy
    mut fox_query: Query<(&mut Transform, &MoveSpeed), With<Fox>>,
) {
    for (mut transform, move_speed) in &mut fox_query {
        let mut direction = Vec3::ZERO;

        // Determine the direction based on key input
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // Normalize the direction vector to avoid faster diagonal movement
        if direction != Vec3::ZERO {
            direction = direction.normalize();
        }

        // Move the fox in the direction with the given speed
        transform.translation += direction * move_speed.0 * time.delta_seconds();

        // Update fox orientation to face the direction of movement
        if direction != Vec3::ZERO {
            transform.rotation = Quat::from_rotation_y(-direction.x.atan2(direction.z));
        }
    }
}
