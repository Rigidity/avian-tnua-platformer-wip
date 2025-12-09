use avian2d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bevy_ecs_tiled::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian2d::prelude::*;

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PhysicsPlugins::default().with_length_unit(100.0),
            PhysicsDebugPlugin,
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian2dPlugin::new(FixedUpdate),
            TiledPlugin::default(),
            TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default(),
        ))
        .insert_resource(Gravity(Vec2::NEG_Y * 9.81 * 15.0))
        .insert_resource(ClearColor(css::BLACK.into()))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, apply_controls.in_set(TnuaUserControlsSystems))
        .add_systems(FixedUpdate, move_camera.after(PhysicsSystems::Last))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn a 2D camera
    commands.spawn((
        PlayerCamera,
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.25,
            ..OrthographicProjection::default_2d()
        }),
    ));

    // Spawn player
    commands
        .spawn((
            Visibility::default(),
            Player,
            RigidBody::Dynamic,
            Collider::rectangle(8.0, 8.0),
            TnuaController::default(),
            TnuaAvian2dSensorShape(Collider::rectangle(7.95, 7.95)),
            LockedAxes::ROTATION_LOCKED,
            LinearDamping(0.7),
            SweptCcd::default(),
            Friction::new(0.0),
            Transform::from_xyz(0.0, 100.0, 0.0),
        ))
        .with_child((
            Mesh2d(meshes.add(Rectangle::new(8.0, 8.0))),
            MeshMaterial2d(materials.add(Color::from(css::WHITE))),
        ));

    // Spawn a square
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(10.0, 10.0))),
        MeshMaterial2d(materials.add(Color::from(css::RED))),
        Transform::from_xyz(20.0, 75.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(10.0, 10.0),
        Friction::new(0.0),
    ));

    // Load a map asset and retrieve its handle
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map.tmx");

    // Spawn a new entity with the TiledMap component
    commands.spawn(TiledMap(map_handle)).observe(
        |collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
            commands
                .entity(collider_created.event().origin)
                .insert((RigidBody::Static, Friction::new(0.0)));
        },
    );
}

fn move_camera(
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation = camera_transform
        .translation
        .lerp(player_transform.translation, 0.1);
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut TnuaController>,
) {
    let Ok(mut controller) = controller_query.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyA) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * 50.0,
        acceleration: 100.0,
        air_acceleration: 75.0,
        float_height: 0.1,
        ..Default::default()
    });

    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            height: 30.0,
            ..Default::default()
        });
    }
}
