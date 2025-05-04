//!This crate is a bevy collision detection crate.
//! This crate adds a collider and a collision event.
use bevy::prelude::*;

#[derive(Component)]
///This component is the collider.
/// This is how you use it:
/// Create a collider using [Collider::new_cuboid_collider] or [Collider::new_round_collider].
/// Add the collider to an object when spawning using [Commands::spawn] it to add collision to that object.
/// Add an [Update] system that checks for collision between two colliders using [Collider::check_collider_collision] if the event_feature is not enabled.
/// If the event feature is enabled, then add an observer to get triggered by the [CollisionEvent].
/// Add another [Update] system that updates collider position using [Collider::move_collider] if auto_move is disabled.
/// If auto_move is enabled, then move the object with the collider.
/// If you want to use the event or auto_move features, add the [ColliderPlugin] to your app.
/// WARNING!
/// Physics is not present in this library.
/// If you want a physics library, use bevy_rapier2d.
/// WARNING!
pub struct Collider(Vec<Vec2>);
impl Collider {
    ///This function creates a new cuboid collider.
    /// Here is an explanation on fow to use this function and how it works:
    /// half_x, half_y: Half extent for cuboid collider.
    /// position: The position_of_the_collider.
    /// angle: The angle of the collider.
    pub fn new_cuboid_collider(x: i32, y: i32, position: Vec2, angle: f32) -> Self {
        let mut collider_vec = Vec::new();
        let y_offset = (y * 2) / (x * 2);
        for i in 0..y_offset {
            for vec in Self::new_round_collider(x as f32, Quat::from_axis_angle(Vec3::Z,angle).mul_vec3(Vec3::new(0.0, (y_offset * i) as f32, 0.0)).truncate(), 90).0.into_iter(){
                collider_vec.push(vec + position.round());
            }
        }
        Collider(collider_vec)
    }
    ///This function cheks if the two colliders have collided.
    ///Here is how it is meant to be used:
    ///if collider1.check_collider_collision(collider2) {//Do something}
    pub fn check_collider_collision(self, other: Self) -> bool {
        let mut return_value = false;
        for vec1 in self.0.into_iter() {
            for vec2 in other.0.clone().into_iter() {
                if vec1 == vec2 {
                    return_value = true;
                    break;
                }
            }
        }
        return_value
    }
    ///Most of this function is self-explanatory except for euclid_angle witch makes points that have an angle, which is a multiple of euclid_angle detect collision.
    pub fn new_round_collider(radius: f32, position: Vec2, euclid_angle: i32) -> Self {
        let mut collider_vec = Vec::new();
        let up_vec = Vec2 { x: 0.0, y: radius};
        for angle in 0..=(360 / euclid_angle) {
            let angle2 = angle * euclid_angle;
            for i in 0..=up_vec.y.round() as i32 {
                let vec = Quat::from_axis_angle(Vec3::Z, (angle2 as f32).to_radians()).mul_vec3(Vec2::new(0.0, i as f32).extend(0.0));
                collider_vec.push(vec.truncate() + position.round());
            }
        }
        Collider(collider_vec)
    }
    /// This function moves a collider.
    pub fn move_collider(mut collider: Mut<Collider>, new_position: Vec2, old_collider_position: Vec2) {
        for vec2 in collider.0.iter_mut() {
            *vec2 += new_position.round() - old_collider_position.round()
        }
    }

    ///This function combines two colliders.
    pub fn combine_colliders(self, mut other: Self) -> Self {
        let mut collider_vec = self.0.clone();
        collider_vec.append(&mut other.0);
        Collider(collider_vec)
    }
}

impl Clone for Collider {
    fn clone(&self) -> Self {
        Collider(self.0.clone())
    }
}
#[derive(Resource, Default)]
struct OldColliderPosition(Vec<Vec2>);

#[cfg(feature = "auto_move")]
fn innit_auto_move(mut old_collider_position: ResMut<OldColliderPosition>, query: Query<&Transform, With<Collider>>) {
    for transform in query.iter() {
        old_collider_position.0.push(transform.translation.truncate());
    }
}

#[cfg(feature = "auto_move")]
fn auto_move(mut old_collider_position: ResMut<OldColliderPosition>, query: Query<&Transform, With<Collider>>, mut collider_query: Query<&mut Collider>) {
    for (i, transform) in query.iter().enumerate() {
        Collider::move_collider(collider_query.iter_mut().nth(i).unwrap(), transform.translation.truncate(), old_collider_position.0.clone().into_iter().nth(i).unwrap());
        transform.translation.truncate().clone_into(old_collider_position.0.iter_mut().nth(i).unwrap());
    }
}

///This plugin adds a collision event and automatic collider movement if the vent and auto_move features are enabled.
///It is meant to be used with [EntityCommands::observe].
///This feature is enabled with the event or the auto_move features;
#[cfg(feature = "plugin")]
pub struct ColliderPlugin;
#[cfg(feature = "plugin")]
impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "event")]
        app.add_systems(Update, event_trigger).add_event::<CollisionEvent>();

        if cfg!(feature = "auto_move") {
            app.init_resource::<OldColliderPosition>();
            app.add_systems(PostStartup,innit_auto_move);
            app.add_systems(Update, auto_move);
        }
    }
}


#[cfg(feature = "event")]
#[derive(Event)]
///This is an event that gets triggered if there is a collision between two colliders.
pub struct CollisionEvent {
    collider1: Collider,
    collider2: Collider
}

#[cfg(feature = "event")]
pub fn event_trigger(collider_query: Query<(Entity,&Collider), With<Collider>>, mut commands: Commands) {
    let collider_query2 = collider_query.clone();
    for (entity,collider) in collider_query.iter() {
        for (_, collider2) in collider_query2.iter() {
            if collider.0 != collider2.0 {
                if collider.clone().check_collider_collision(collider2.clone()) {
                    commands.trigger_targets(CollisionEvent{collider1:collider.clone(), collider2:collider2.clone()}, entity);
                }
            }
        }
    }
}

#[cfg(feature = "plugin")]
#[cfg(not(any(feature = "auto_move", feature = "event")))]
compile_error!("Either auto_move or event or both must be enabled to use the plugin");