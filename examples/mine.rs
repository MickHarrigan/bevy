//! Testbed for my bevy <-> winit bug

use bevy::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Super>()
        .add_systems(Startup, init)
        .add_systems(
            Update,
            (
                probe,
                (Super::log, Super::update).run_if(resource_changed::<Super>),
            )
                .chain(),
        )
        .run();
}

#[derive(Resource, Default, Debug, PartialEq)]
enum Super {
    Held,
    Released,
    #[default]
    Unknown,
}

impl From<bool> for Super {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Held,
            false => Self::Released,
        }
    }
}

impl Super {
    pub fn log(this: Res<Self>) {
        info!("Super: {:?}", this)
    }

    pub fn update(this: Res<Self>, mut background: ResMut<ClearColor>) {
        let color = match *this {
            Super::Held => Srgba::RED,
            Super::Released => Srgba::GREEN,
            Super::Unknown => Srgba::BLUE,
        };

        background.0 = color.into();
    }
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn probe(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<Super>) {
    let held = keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight);

    if state.set_if_neq(held.into()) {
        info!(super_held = held, "ButtonInput changed");
    }
}
