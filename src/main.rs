use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::mesh::Mesh,
    sprite::MaterialMesh2dBundle,
    window::PresentMode,
};

use std::collections::HashSet;

use rand::Rng;
use rayon::prelude::*;

type Pos = (isize, isize);
type CellType = HashSet<Pos>;

#[derive(Resource)]
struct CellStyle {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<ColorMaterial>>,
}

#[derive(Component)]
struct Cells(CellType);

#[derive(Component)]
struct VisibleCell;

// TODO: Add 'builder' pattern support and support for translating structures.
impl Cells {
    #[allow(dead_code)]
    fn new() -> Self {
        Cells(CellType::new())
    }

    #[allow(dead_code)]
    fn random(number: usize, extents: (isize, isize, isize, isize)) -> Self {
        let (x, y, width, height) = extents;
        // let x_range = x..(x + width);
        // let y_range = y..(y + height);
        let mut rng = rand::thread_rng();

        let mut cells = CellType::new();

        for _n in 0..number {
            let a = rng.gen_range(x..(x + width));
            let b = rng.gen_range(y..(y + height));
            cells.insert((a, b));
        }

        Cells(cells)
    }

    #[allow(dead_code)]
    fn solid_rect(extents: (isize, isize, isize, isize)) -> Self {
        let (x, y, width, height) = extents;

        let mut cells = CellType::new();

        for ix in x..(x + width) {
            for iy in y..(y + height) {
                cells.insert((ix, iy));
            }
        }

        Cells(cells)
    }

    /// Create a hollow rectangle. I don't know if there is a use case for this,
    /// since all internal cells will die from overpopulation anyway.
    #[allow(dead_code)]
    fn hollow_rect(wall_thick: isize, extents: (isize, isize, isize, isize)) -> Self {
        let (x, y, width, height) = extents;

        let mut cells = CellType::new();
        let mut cells_inner = CellType::new();

        for ix in x..(x + width) {
            for iy in y..(y + height) {
                cells.insert((ix, iy));
            }
        }

        for ix in (x + wall_thick)..(x + width - wall_thick) {
            for iy in (y + wall_thick)..(y + height - wall_thick) {
                cells_inner.insert((ix, iy));
            }
        }

        Cells(cells.difference(&cells_inner).map(|c| *c).collect())
    }

    #[allow(dead_code)]
    fn blinker() -> Self {
        Cells(CellType::from([(-1, 0), (0, 0), (1, 0)]))
    }

    #[allow(dead_code)]
    fn glider() -> Self {
        Cells(CellType::from([(-1, 0), (0, 0), (1, 0), (1, -1), (0, -2)]))
    }

    #[allow(dead_code)]
    fn pentadecathlon() -> Self {
        Cells(CellType::from([
            (0, 0),
            (0, -1),
            (1, -1),
            (1, -2),
            (3, 0),
            (6, 0),
            (6, -1),
            (6, -2),
            (7, -1),
        ]))
    }

    fn get_neighbors_keys(&self, key: &Pos) -> CellType {
        let (x, y) = *key;
        CellType::from([
            (x - 1, y - 1),
            (x, y - 1),
            (x + 1, y - 1),
            (x - 1, y),
            (x + 1, y),
            (x - 1, y + 1),
            (x, y + 1),
            (x + 1, y + 1),
        ])
    }

    fn get_self_and_neighbors_keys(&self, key: &Pos) -> CellType {
        let mut keys = self.get_neighbors_keys(key);
        keys.insert(*key);
        keys
    }

    fn count_neighbors(&self, key: &Pos) -> u8 {
        let neighbors = self.get_neighbors_keys(key);

        // Count neighbors
        neighbors.iter().map(|k| self.0.contains(k) as u8).sum()
    }
}

fn update_world(mut q_cells: Query<&mut Cells>) {
    // TODO: This can panic!
    let mut cells = q_cells.single_mut();

    // println!("Cells: {}", cells.0.len());

    // Declare new Cells. Will add to this in loop below.
    // let mut next_cells = Cells::new();
    // let mut checked_cells = Cells::new();

    // Create list of all cells to check
    let cells_to_check: CellType = cells
        .0
        .par_iter()
        .map(|k| cells.get_self_and_neighbors_keys(k))
        .flatten()
        .collect();

    // Loop over all live cells and their neighbors
    let next_live_cells: CellType = cells_to_check
        .par_iter()
        .map(|key| {
            // if checked_cells.0.contains(&key) {
            //     continue;
            // }

            // // Add current cell to checked_cells
            // checked_cells.0.insert(*key);

            let count = cells.count_neighbors(&key);
            let alive = cells.0.contains(&key);

            // println!("{:?} = {}", key, count);

            return if alive && (count == 2 || count == 3) {
                // Stay alive
                // next_cells.0.insert(*key);
                Some(*key)
            } else if !alive && count == 3 {
                // Come alive
                // next_cells.0.insert(*key);
                Some(*key)
            } else {
                None
            };
        })
        .into_par_iter()
        .flatten() // Remove Nones and "unwrap" Somes
        .collect();

    let next_cells = Cells(next_live_cells);

    // println!("{:?} = {} ;; neighbors = {}", k, v, count);

    // Replace existing cells with next generation
    cells.0 = next_cells.0;
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

    for k in cells.0.iter() {
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
    // commands.spawn(Cells::pentadecathlon());
    // commands.spawn(Cells::random(5000, (-120, -67, 240, 135)));
    commands.spawn(Cells::solid_rect((-100, -50, 200, 100)));
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
