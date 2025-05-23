use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};
use std::f32::consts::{ PI, TAU };
use rand::random;
use avian3d::prelude::*;

#[derive(Component)]
struct Phys {
    pos: Vec2,
    acc: f32,
    max_acc: f32,
}

#[derive(Component)]
struct Spin;
#[derive(Component)]
struct Cam {
    r: f32
}

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
        let size = self.size;
        let idx = z * size * size + y * size + x;
        self.data[idx as usize]
    }

    pub fn map<F>(&mut self, mut func: F)
    where F: FnMut(u32, u32, u32, f32) -> f32 {
        let size = self.size;
        for i in 0..self.data.len() {
            let z = (i as u32 / (size * size)) % size;
            let y = (i as u32 / size) % size;
            let x = i as u32 % size;
            self.data[i] = func(x, y, z, self.data[i]);
        }
    }

    pub fn each<F>(&mut self, mut func: F)
    where F: FnMut(u32, u32, u32, f32) -> () {
        let size = self.size;
        for i in 0..self.data.len() {
            let z = (i as u32 / (size * size)) % size;
            let y = (i as u32 / size) % size;
            let x = i as u32 % size;
            func(x, y, z, self.data[i]);
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, (setup,add_axes))
        .add_systems(Update, (spinner, cam_follow, collides))
        .add_observer(ball_spawn)
        .run();
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut vox = VoxelGrid::new(10);
    let size = vox.size as f32;
    let hsize = size / 2.0;
    vox.map(|x, y, z, _val| {
        let xo = x as f32 - hsize;
        let yo = y as f32;
        let zo = z as f32 - hsize;
        (xo * xo + yo * yo + zo * zo).sqrt()
    });

    let mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.5, 0.5),
        ..default()
    });
    let mat_off = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.4, 0.8, 0.8),
        ..default()
    });
    let sphere = meshes.add(Sphere::default());

    vox.each(|x, y, z, val| {
        let xo = x as f32 - hsize;
        let yo = y as f32 - hsize;
        let zo = z as f32 - hsize;

        cmds.spawn((
            MeshMaterial3d(if val < 5.0 { mat.clone() } else { mat_off.clone() }),
            Mesh3d(sphere.clone()),
            Transform::from_xyz(xo, yo, zo)
                .with_scale(Vec3::splat(0.1))
        ));
    });

    cmds.spawn((
        Name::new("cam"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 20.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Dir3::Y),
        Cam { r: 20.0 }
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

    // let limit = random::<f32>() * 4.0;
    let limit = 5.0;
    let mesh = create_mesh(&vox, limit);
    cmds.spawn((
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        RigidBody::Static,
        Collider::trimesh_from_mesh(&mesh).unwrap(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(mesh)),
        CollidingEntities::default()
    ));

    for pos in [
        [-2.5, -0.5, -0.5],
        [-2.5, -0.5, -1.5],
        [-3.5, -2.5, 2.5]
    ] {
        cmds.trigger(BallSpawn {
            pos: Vec3::new(pos[0], pos[1], pos[2]),
            ptype: 1
        });
    }

    cmds.spawn((
        RigidBody::Static,
        Collider::cylinder(10.0, 0.1),
        Mesh3d(meshes.add(Cylinder::new(20.0, 0.1))),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Transform::from_xyz(0.0, -5.0, 0.0),
    ));

    for _ in 0..30 {
        cmds.trigger(BallSpawn {
            pos: Vec3::new(
               random::<f32>() * 10.0 - 5.0,
               random::<f32>() * 2.0 + 2.0,
               random::<f32>() * 10.0 - 5.0,
            ),
            ptype: 0
        });
    }
}


fn collides(query: Query<(Entity, &CollidingEntities)>) {
    for (entity, colliding_entities) in &query {
        /*println!(
            "{} is colliding with the following entities: {:?}",
            entity,
            colliding_entities,
        );*/
    }
}

fn add_axes(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let w = 0.01;
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(1.0, 0.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::new(100.0, w, w))
    ));
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.0, 0.3, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::new(w, w, 100.0))
    ));
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.0, 1.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::new(w, 100.0, w))
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

fn create_mesh(vox: &VoxelGrid, limit: f32) -> Mesh {
    let size = vox.size;
    let vol = size * size * size;
    let xo = -(size as f32 / 2.0);
    let yo = xo;
    let zo = xo;

    let mut verts: Vec<[f32; 3]> = vec![];

    for i in 0..vol {
        let val = vox.data[i as usize];
        if val > limit {
            continue;
        }

        let x = (i % size) as f32 + xo;
        let y = ((i / size) % size) as f32 + yo;
        let z = ((i / (size * size)) % size) as f32 + zo;

        // Front
        verts.push([x - 1.0, y, z]);
        verts.push([x - 1.0, y - 1.0, z]);
        verts.push([x, y - 1.0, z]);
        verts.push([x - 1.0, y, z]);
        verts.push([x, y - 1.0, z]);
        verts.push([x, y, z]);

        // Back
        verts.push([x, y, z - 1.0]);
        verts.push([x, y - 1.0, z - 1.0]);
        verts.push([x - 1.0, y - 1.0, z - 1.0]);
        verts.push([x, y, z - 1.0]);
        verts.push([x - 1.0, y - 1.0, z - 1.0]);
        verts.push([x - 1.0, y, z - 1.0]);

        // Top
        verts.push([x - 1.0, y, z]);
        verts.push([x, y, z]);
        verts.push([x, y, z - 1.0]);
        verts.push([x - 1.0, y, z]);
        verts.push([x, y, z - 1.0]);
        verts.push([x - 1.0, y, z - 1.0]);

        // Bottom
        verts.push([x, y, z - 1.0]);
        verts.push([x, y, z]);
        verts.push([x - 1.0, y - 1.0, z]);
        verts.push([x, y - 1.0, z - 1.0]);
        verts.push([x - 1.0, y - 1.0, z]);
        verts.push([x - 1.0, y - 1.0, z - 1.0]);

        // Left
        verts.push([x - 1.0, y, z - 1.0]);
        verts.push([x - 1.0, y - 1.0, z - 1.0]);
        verts.push([x - 1.0, y - 1.0, z]);
        verts.push([x - 1.0, y, z - 1.0]);
        verts.push([x - 1.0, y - 1.0, z]);
        verts.push([x - 1.0, y, z]);

        // Right
        verts.push([x, y, z]);
        verts.push([x, y - 1.0, z - 1.0]);
        verts.push([x, y, z - 1.0]);
        verts.push([x, y, z]);
        verts.push([x, y - 1.0, z]);
        verts.push([x, y - 1.0, z - 1.0]);
    }

    let len = verts.len();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(verts)
    )
    // TODO: reusue verts, hey...
    .with_inserted_indices(Indices::U32((0..=len as u32).collect()));

    mesh.compute_normals();
    mesh
}

fn cam_follow(
    mut cams: Query<(&mut Transform, &Cam)>,
    time: Res<Time>
) {
    let elapsed = time.elapsed_secs() * 0.1;
    for (mut t, cam) in cams.iter_mut() {
        t.translation.x = elapsed.sin() * cam.r;
        t.translation.z = elapsed.cos() * cam.r;
        t.translation.y = elapsed.sin() * 5.0;
        t.look_at(Vec3::new(0.0, 0.0, 0.0), Dir3::Y);
    }
}

#[derive(Debug, Event)]
struct BallSpawn {
    pos: Vec3,
    ptype: u32,
}

fn ball_spawn(
    trigger: Trigger<BallSpawn>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let pos = trigger.event().pos;
    let ptype = trigger.event().ptype;

    cmds.spawn((
        if ptype == 0 { RigidBody::Dynamic } else { RigidBody::Static },
        Collider::sphere(0.5),
        Restitution::new(0.8)
            .with_combine_rule(CoefficientCombine::Max),
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(pos),
    ));
}
