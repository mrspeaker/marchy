use bevy::{
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};
use std::f32::consts::{ PI, TAU };
use rand::random;

#[derive(Component)]
struct Spin;

struct VoxelGrid {
    size: u32,
    data: Vec<f32>
}

impl VoxelGrid {
    pub fn new(size: u32) -> Self {
        VoxelGrid {
            size,
            data: vec![0.0; (size * size * size) as usize]
        }
    }

    pub fn read(&self, x: u32, y: u32, z: u32) -> f32 {
        let idx = z * self.size * self.size + y * self.size + x;
        self.data[idx as usize]
    }

    pub fn map<F>(&mut self, mut func: F)
    where F: FnMut(u32, u32, u32, f32) -> f32 {
        for i in 0..self.data.len() {
            let z = (i as u32 / (self.size * self.size)) % self.size;
            let y = (i as u32 / self.size) % self.size;
            let x = i as u32 % self.size;
            self.data[i] = func(x, y, z, self.data[i]);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup,add_axes))
        .add_systems(Update, spinner)
        .run();
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut vox = VoxelGrid::new(10);
    /*vox.map(|_x, _y, _z, _val| {
        random::<f32>() * 2.0 - 1.0
    });*/
    let size = vox.size as f32;
    vox.map(|x, y, z, _val| {
        let xo = x as f32 - size / 2.0;
        let yo = y as f32 - size / 2.0;
        let zo = z as f32 - size / 2.0;
        (xo * xo + yo * yo + zo * zo).sqrt()
    });

    cmds.spawn((
        Name::new("cam"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 20.0)
    ));

    cmds.insert_resource(AmbientLight {
        color: Color::linear_rgb(1.0,1.0, 1.0),
        brightness: 100.0,
        ..default()
    });

    cmds.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },

    ));

    cmds.spawn((
        Mesh3d(meshes.add(create_mesh(&vox))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn add_axes(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(-25.0, 0.0, 0.0)
            .with_scale(Vec3::new(100.0, 0.1, 0.1))
    ));
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(0.0, 0.0, -50.0)
            .with_scale(Vec3::new(0.1, 0.1, 100.0))
    ));
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::new(0.1, 100.0, 0.1))
    ));
}

fn spinner(
    mut spinners: Query<&mut Transform, With<Spin>>,
    time: Res<Time>
){
    let dt = time.delta_secs();
    for mut t in spinners.iter_mut() {
        t.rotate_y(TAU * dt * 0.02);
        t.rotate_x(TAU * dt * 0.03);
        t.rotate_z(TAU * dt * 0.01);
    }
}

#[rustfmt::skip]
fn create_mesh(vox: &VoxelGrid) -> Mesh {

    let size = vox.size;
    let vol = size * size * size;
    let xo = -(size as f32 / 2.0);
    let yo = xo;
    let zo = xo;

    let mut verts: Vec<[f32; 3]> = vec![];

    let limit = random::<f32>() * 5.0 + 2.0;

    for i in 0..vol {
        let val = vox.data[i as usize];
        if val > limit {
            continue;
        }

        let x = (i % size) as f32 + xo;
        let y = ((i / size) % size) as f32 + yo;
        let z = ((i / (size * size)) % size) as f32 + zo;

        // Front
        verts.push([x, y + 1.0, z]);
        verts.push([x, y, z]);
        verts.push([x + 1.0, y, z]);
        verts.push([x, y + 1.0, z]);
        verts.push([x + 1.0, y, z]);
        verts.push([x + 1.0, y + 1.0, z]);

        // Back
        verts.push([x + 1.0, y + 1.0, z - 1.0]);
        verts.push([x + 1.0, y, z - 1.0]);
        verts.push([x, y, z - 1.0]);
        verts.push([x + 1.0, y + 1.0, z - 1.0]);
        verts.push([x, y, z - 1.0]);
        verts.push([x, y + 1.0, z - 1.0]);

        // Top
        verts.push([x, y + 1.0, z]);
        verts.push([x + 1.0, y + 1.0, z]);
        verts.push([x + 1.0, y + 1.0, z - 1.0]);
        verts.push([x, y + 1.0, z]);
        verts.push([x + 1.0, y + 1.0, z - 1.0]);
        verts.push([x, y + 1.0, z - 1.0]);

        // Bottom
        verts.push([x + 1.0, y, z - 1.0]);
        verts.push([x + 1.0, y, z]);
        verts.push([x, y, z]);
        verts.push([x + 1.0, y, z - 1.0]);
        verts.push([x, y, z]);
        verts.push([x, y, z - 1.0]);

        // Left
        verts.push([x, y + 1.0, z - 1.0]);
        verts.push([x, y, z - 1.0]);
        verts.push([x, y, z]);
        verts.push([x, y + 1.0, z - 1.0]);
        verts.push([x, y, z]);
        verts.push([x, y + 1.0, z]);

        // Right
        verts.push([x + 1.0, y + 1.0, z]);
        verts.push([x + 1.0, y, z - 1.0]);
        verts.push([x + 1.0, y + 1.0, z - 1.0]);
        verts.push([x + 1.0, y + 1.0, z]);
        verts.push([x + 1.0, y, z]);
        verts.push([x + 1.0, y, z - 1.0]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(verts)
    );

    mesh.compute_normals();
    mesh
}
