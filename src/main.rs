use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::mesh::Mesh,
    sprite::MaterialMesh2dBundle,
};

use std::collections::HashMap;

use rand::Rng;

type Pos = (isize, isize);
type Cell = HashMap<Pos, bool>; // TODO: Think about using HashSet instead

#[derive(Component)]
struct Cells(Cell);

#[derive(Component)]
struct VisibleCell;

impl Cells {
    fn new() -> Self {
        Cells(Cell::new())
    }

    fn random(number: usize, extents: (isize, isize, isize, isize)) -> Self {
        let (x, y, width, height) = extents;
        // let x_range = x..(x + width);
        // let y_range = y..(y + height);
        let mut rng = rand::thread_rng();

        let mut cells: Vec<(Pos, bool)> = Vec::new();

        for n in 0..number {
            let a = rng.gen_range(x..(x + width));
            let b = rng.gen_range(y..(y + height));
            cells.push(((a, b), true));
        }

        // Cells(Cell::from(cells))
        cells.iter().collect()
    }

    fn blinker() -> Self {
        Cells(Cell::from([
            ((-1, 0), true),
            ((0, 0), true),
            ((1, 0), true),
        ]))
    }

    fn glider() -> Self {
        Cells(Cell::from([
            ((-1, 0), true),
            ((0, 0), true),
            ((1, 0), true),
            ((1, -1), true),
            ((0, -2), true),
        ]))
    }

    fn pentadecathlon() -> Self {
        Cells(Cell::from([
            ((0, 0), true),
            ((0, -1), true),
            ((1, -1), true),
            ((1, -2), true),
            ((3, 0), true),
            ((6, 0), true),
            ((6, -1), true),
            ((6, -2), true),
            ((7, -1), true),
        ]))
    }

    fn get_neighbors_keys(&self, key: &Pos) -> Vec<Pos> {
        let (x, y) = *key;
        vec![
            (x - 1, y - 1),
            (x, y - 1),
            (x + 1, y - 1),
            (x - 1, y),
            (x + 1, y),
            (x - 1, y + 1),
            (x, y + 1),
            (x + 1, y + 1),
        ]
    }

    fn get_self_and_neighbors_keys(&self, key: &Pos) -> Vec<Pos> {
        let mut keys = self.get_neighbors_keys(key);
        keys.push(*key);
        keys
    }

    fn count_neighbors(&self, key: &Pos) -> u8 {
        let neighbors = self.get_neighbors_keys(key);

        // Count neighbors
        neighbors.iter().map(|k| self.0.contains_key(k) as u8).sum()
    }
}

impl<'a> FromIterator<&'a (Pos, bool)> for Cells {
    fn from_iter<T: IntoIterator<Item = &'a (Pos, bool)>>(iter: T) -> Self {
        let mut map = Cells::new();
        for (k, v) in iter {
            map.0.insert(*k, *v);
        }

        map
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
        },
        ..Default::default()
    });
}

fn update_world(mut q_cells: Query<&mut Cells>) {
    // TODO: This can panic!
    let mut cells = q_cells.single_mut();

    // println!("Cells: {}", cells.0.len());

    // Declare new Cells. Will add to this in loop below.
    let mut next_cells = Cells::new();
    let mut checked_cells = Cells::new();

    // Create list of all cells to check
    let cells_to_check: Vec<Pos> = cells
        .0
        .keys()
        .map(|k| cells.get_self_and_neighbors_keys(k))
        .flatten()
        .collect();

    // Loop over all live cells
    for key in &cells_to_check {
        if checked_cells.0.contains_key(&key) {
            continue;
        }

        // Add current cell to checked_cells
        checked_cells.0.insert(*key, true);

        let count = cells.count_neighbors(key);
        let alive = *cells.0.get(&key).unwrap_or(&false);

        // println!("{:?} = {}", key, count);

        if alive && (count == 2 || count == 3) {
            // Stay alive
            next_cells.0.insert(*key, true);
        } else if !alive && count == 3 {
            // Come alive
            next_cells.0.insert(*key, true);
        } // else if (alive || !alive) { // die or stay dead }
    }

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

fn render_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_cells: Query<&Cells>,
) {
    let size = 1.5;
    let scale = 4.;

    // TODO: This can panic!
    let cells = q_cells.single();

    for k in cells.0.keys() {
        let x = k.0 as f32;
        let y = k.1 as f32;

        commands.spawn((
            VisibleCell,
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(size).into()).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                transform: Transform::from_translation(scale * Vec3::new(x, y, 0.)),
                ..default()
            },
        ));
    }
}

fn init_cells(mut commands: Commands) {
    commands.spawn(Cells::random(4000, (-60, -60, 120, 120)));
    // commands.spawn(Cells::pentadecathlon());
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, (setup, init_cells))
        .add_systems(Update, (update_world, clear_cells, render_cells))
        .run();
}
