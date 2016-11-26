use physics::Rigidbody;
use gunship::*;
use gunship::math::*;
use gunship::mesh_renderer::MeshRenderer;
use gunship::resource::Mesh;
use gunship::transform::Transform;
use std::sync::Arc;

/// Represents the cylinder of a revolver, tracking the contents of each cylinder.
#[derive(Debug)]
pub struct Cylinder {
    pub transform: Transform,

    pub cylinders: Box<[Option<Cartridge>]>,

    /// The current position of the cylinder relative to the hammer.
    ///
    /// The cartridge at cylinder `position` is the one currently under the hammer.
    pub position: usize,
}

impl Cylinder {
    /// Creates a new cylinder with the specified number of cylinders.
    ///
    /// All cylinders in the new cylinder default to being empty.
    pub fn new(capacity: usize) -> Cylinder {
        let mut cylinders = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            cylinders.push(None);
        }
        debug_assert_eq!(capacity, cylinders.len());

        Cylinder {
            transform: Transform::new(),
            cylinders: cylinders.into_boxed_slice(),
            position: 0,
        }
    }

    /// Gets a reference to the cylinder that's currently under the hammer.
    pub fn current(&self) -> &Option<Cartridge> {
        &self.cylinders[self.position]
    }

    /// Gets a mutable reference to the cylinder that's currently under the hammer.
    pub fn current_mut(&mut self) -> &mut Option<Cartridge> {
        &mut self.cylinders[self.position]
    }

    /// Gets the number of cartidges the cylinder can hold.
    pub fn capacity(&self) -> usize {
        self.cylinders.len()
    }
}

#[derive(Debug)]
pub struct Revolver {
    pub transform: Transform,
    pub mesh_renderer: MeshRenderer,
    pub rigidbody: Rigidbody,

    cylinder: Cylinder,
    cylinder_offset: Vector3,
    cylinder_radius: f32,

    bullet_offset: Vector3, // TODO: Configure based on gun mesh.
    is_cocked: bool,

    bullet_mesh: Arc<Mesh>,
}

impl Revolver {
    pub fn new(
        mesh: &Mesh,
        bullet_mesh: Arc<Mesh>,
        start_pos: Point,
        start_orientation: Orientation,
    ) -> Revolver {
        let mut transform = Transform::new();
        transform.set_position(start_pos);
        transform.set_orientation(start_orientation);
        let mesh_renderer = MeshRenderer::new(&mesh, &transform);
        let rigidbody = Rigidbody::new();

        Revolver {
            transform: transform,
            mesh_renderer: mesh_renderer,
            rigidbody: rigidbody,

            cylinder: Cylinder::new(6),
            cylinder_offset: Vector3::new(0.0, 0.05, 0.0),
            cylinder_radius: 0.03,

            bullet_offset: Vector3::new(0.0, 0.04, 0.2),
            is_cocked: false,

            bullet_mesh: bullet_mesh,
        }
    }

    /// Tries to fire the gun. Returns `true` if the gun fired, `false` otherwise.
    pub fn fire(&mut self) -> bool {
        // If the hammer isn't cocked we can't fire, so do nothing.
        if !self.is_cocked {
            return false;
        }

        // TODO: Animate hammer falling.
        self.is_cocked = false;

        if let Some(cartridge) = self.cylinder.current_mut().as_mut() {
            if !cartridge.has_fired {
                // TODO: Play audio on gunshot.
                // let mut audio_source = audio_manager.get_mut(player.gun_entity).unwrap();
                // audio_source.reset();
                // audio_source.play();

                let bullet_pos = self.transform.position()
                               + (self.bullet_offset.x * self.transform.right())
                               + (self.bullet_offset.y * self.transform.up())
                               + (self.bullet_offset.z * self.transform.forward());
                let mut bullet = Bullet::new(&self.bullet_mesh, bullet_pos, self.transform.orientation());
                engine::run_each_frame(move || {
                    bullet.update();
                });

                // Empty the chartridge.
                // TODO: Change cartridge mesh to empty cartridge.
                cartridge.has_fired = true;

                return true;
            }
        }

        false
    }

    pub fn pull_hammer(&mut self) {
        // TODO: Animate hammer pulling back.
        self.is_cocked = true;
    }

    pub fn rotate_cylinder(&mut self, rotation: isize) {
        // TODO: Animate rotating the cylinder.
        let pos = self.cylinder.position as isize + rotation;
        self.cylinder.position = pos.modulo(self.cylinder.capacity() as isize) as usize;
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) -> Result<(), Cartridge> {
        // TODO: Animate loading the cartridge.
        let cylinder = self.cylinder.current_mut();
        match cylinder {
            &mut Some(_) => Err(cartridge),
            &mut None => {
                *cylinder = Some(cartridge);
                Ok(())
            }
        }
    }

    pub fn update_transforms(&mut self) {
        let capacity = self.cylinder.capacity();
        let cylinder_position = self.cylinder.position;
        let oriented_offset = self.transform.orientation() * self.cylinder_offset;
        let cylinder_center = self.transform.position() + oriented_offset;

        for (index, cylinder) in self.cylinder.cylinders.iter_mut().enumerate() {
            if let Some(cartridge) = cylinder.as_mut() {
                let pos = (index as isize - cylinder_position as isize).modulo(capacity as isize);

                let rotation = TAU / capacity as f32 * pos as f32;
                let local_orientation = Orientation::from_eulers(0.0, 0.0, rotation);

                let orientation = self.transform.orientation() + local_orientation;
                let cartridge_up_offset = orientation.up() * self.cylinder_radius;

                cartridge.transform.set_orientation(orientation);
                cartridge.transform.set_position(cylinder_center + cartridge_up_offset);
            }
        }
    }
}

/// Tracks state for the bullet cartridge when it's in the gun or the player's inventory.
#[derive(Debug)]
pub struct Cartridge {
    pub transform: Transform,
    pub mesh_renderer: MeshRenderer,

    pub has_fired: bool,
}

/// Tracks state for a bullet that's been fired.
#[derive(Debug)]
pub struct Bullet {
    transform: Transform,
    mesh_renderer: MeshRenderer,
    pub speed: f32,
}

impl Bullet {
    pub fn new(mesh: &Mesh, position: Point, orientation: Orientation) -> Bullet {
        let mut transform = Transform::new();
        transform.set_position(position);
        transform.set_orientation(orientation);

        // TODO: Remove this once we have a proper bullet mesh.
        transform.set_scale(Vector3::new(0.1, 0.1, 1.0));

        let mesh_renderer = MeshRenderer::new(mesh, &transform);

        Bullet {
            transform: transform,
            mesh_renderer: mesh_renderer,
            speed: 100.0
        }
    }

    pub fn update(&mut self) {
        let forward = self.transform.forward();
        self.transform.translate(forward * self.speed * time::delta_f32());
    }
}
