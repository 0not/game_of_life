use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::mesh::Mesh,
    sprite::MaterialMesh2dBundle,
    window::PresentMode,
};

pub mod life;
use life::{CellSet, CellSetType};

#[derive(Resource)]
struct CellStyle {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<ColorMaterial>>,
}

#[derive(Component)]
struct Cells(CellSet);

#[derive(Component)]
struct VisibleCell;

impl std::ops::Deref for Cells {
    type Target = CellSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<CellSet> for Cells {
    fn from(set: CellSet) -> Self {
        Cells(set)
    }
}

impl From<CellSetType> for Cells {
    fn from(set: CellSetType) -> Self {
        Cells(CellSet(set))
    }
}

fn update_world(mut q_cells: Query<&mut Cells>) {
    // TODO: This can panic!
    let mut cells = q_cells.single_mut();

    // Replace existing cells with next generation
    *cells = cells.update_cells().into();
}

/// Remove all VisibleCells
fn clear_cells(mut commands: Commands, query: Query<Entity, With<VisibleCell>>) {
    for vcell in &query {
        commands.entity(vcell).despawn();
    }
}

fn render_cells(mut commands: Commands, cell_style: Res<CellStyle>, q_cells: Query<&Cells>) {
    let scale = 4.;

    // TODO: This can panic!
    let cells = q_cells.single();

    for k in cells.iter() {
        let x = k.0 as f32;
        let y = k.1 as f32;

        commands.spawn((
            VisibleCell,
            MaterialMesh2dBundle {
                mesh: cell_style.mesh.as_ref().unwrap().clone().into(),
                material: cell_style.material.as_ref().unwrap().clone(),
                transform: Transform::from_translation(scale * Vec3::new(x, y, 0.)),
                ..default()
            },
        ));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cell_style: ResMut<CellStyle>,
) {
    let size = 1.5;

    // Initialize mesh
    cell_style.mesh = Some(meshes.add(shape::Circle::new(size).into()));

    // Initialize material
    cell_style.material = Some(materials.add(ColorMaterial::from(Color::WHITE)));

    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
        },
        ..Default::default()
    });
}

fn init_cells(mut commands: Commands) {
    // commands.spawn(Cells::glider());
    // commands.spawn(Cells::random(5000, (-120, -67, 240, 135)));
    commands.spawn(Cells(CellSet::solid_rect((-50, -50, 100, 100))));
    // commands.spawn(Cells::hollow_rect(2, (-50, -50, 100, 100)));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: bevy::window::WindowMode::BorderlessFullscreen,
                    present_mode: PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .insert_resource(CellStyle {
            mesh: None,
            material: None,
        })
        .add_systems(Startup, (setup, init_cells))
        .add_systems(
            Update,
            (
                update_world,
                clear_cells,
                render_cells,
                bevy::window::close_on_esc,
            ),
        )
        .run();
}
