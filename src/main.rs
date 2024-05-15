use bevy::{
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa, tonemapping::Tonemapping},
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::RenderAssetUsages,
        render_resource::*,
        texture::{ImageSampler, ImageSamplerDescriptor},
    },
    window::{PrimaryWindow, WindowResized, WindowScaleFactorChanged},
};
// use bevy_atmosphere::prelude::*;
use character::CharacterEntity;
use render_pipeline::{VoxelVolume, VoxelVolumeBundle};

mod character;
mod render_pipeline;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (1920.0, 1080.0).into(),
                    ..default()
                }),
                ..default()
            }),
            // AtmospherePlugin,
            render_pipeline::VoxelPlugin,
            character::CharacterPlugin,
            ui::UiPlugin,
        ))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_streaming_pos, update_render_texture))
        .run();
}

#[allow(dead_code)]
#[derive(Resource)]
struct CameraData {
    render_texture: Handle<Image>,
    sprite: Entity,
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // we use a render texture to downscale the main pass
    let mut render_texture = Image::new_fill(
        Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0u8; 8],
        TextureFormat::Rgba16Float,
        RenderAssetUsages::MAIN_WORLD, // Must be MAIN_WORLD because we access it later
    );
    render_texture.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    render_texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
    let render_texture = images.add(render_texture);

    // add voxel volume
    commands.spawn(VoxelVolumeBundle::default());

    // add camera with character controller
    let character_transform =
        Transform::from_xyz(21.035963, 19.771912, -31.12883).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Camera3dBundle {
            transform: character_transform,
            camera: Camera {
                hdr: true,
                target: RenderTarget::Image(render_texture.clone()),
                order: -10,
                ..default()
            },
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 1.57,
                near: 0.001,
                far: 100.0,
                ..default()
            }),
            // tonemapping: Tonemapping::None,
            ..default()
        },
        CharacterEntity {
            look_at: -character_transform.local_z(),
            ..default()
        },
        BloomSettings::default(),
        Fxaa::default(),
        // AtmosphereCamera::default(),
    ));

    // add sprite and camera to render the render texture
    let sprite = commands
        .spawn(SpriteBundle {
            texture: render_texture.clone(),
            ..default()
        })
        .id();
    commands.spawn((Camera2dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        tonemapping: Tonemapping::None,
        ..default()
    },));
    commands.insert_resource(CameraData {
        render_texture,
        sprite,
    });
}

fn update_streaming_pos(
    mut voxel_volumes: Query<&mut VoxelVolume>,
    character: Query<&Transform, With<CharacterEntity>>,
) {
    let character = character.single();
    let mut voxel_volume = voxel_volumes.single_mut();

    voxel_volume.streaming_pos = character.translation;
}

fn update_render_texture(
    mut resize_reader: EventReader<WindowResized>,
    mut scale_factor_reader: EventReader<WindowScaleFactorChanged>,
    mut images: ResMut<Assets<Image>>,
    mut _sprites: Query<&mut Sprite>,
    render_image: Res<CameraData>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();

    let mut update = |width: f32, height: f32| {
        let new_size = Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        };

        info!("Resizing render texture to {:?}", new_size);

        let image = images.get_mut(&render_image.render_texture).unwrap();
        image.resize(new_size);
    };

    for _ in resize_reader.read() {
        update(window.width(), window.height());
    }

    for _ in scale_factor_reader.read() {
        update(window.width(), window.height());
    }
}
