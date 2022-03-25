use std::fs;

use bevy::{prelude::*, reflect::TypeRegistry, ecs::system::Command};
use bevy_kajiya::kajiya_render::KajiyaDescriptor;
use concord_logger::console_info;

enum SceneCommand {
    SaveScene(String),
}
  
impl Command for SceneCommand {
    fn write(self, world: &mut World) {
        match self {
            SceneCommand::SaveScene(scene_name) => {
                
                let path = format!("assets/scenes/{}.sav.ron", scene_name);

                let type_registry = world.get_resource::<TypeRegistry>().unwrap();
                let scene = DynamicScene::from_world(&world, type_registry);
                let scene_serialized = scene.serialize_ron(type_registry).unwrap();
                
                if let Ok(()) = fs::write(&path, scene_serialized) {
                    console_info!("Saved scene to \"{}\"", &path);
                } else {
                    console_info!("ERROR: failed to save scene to \"{}\"", &path);
                }
            },
        }
    }
}


pub fn save_scene_system(mut commands: Commands,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Return) {
        println!("Save scene command");
        commands.add(SceneCommand::SaveScene("main".to_string()));
    }
}

pub fn load_scene_system(asset_server: Res<AssetServer>, mut scene_spawner: ResMut<SceneSpawner>, kajiya: Res<KajiyaDescriptor>) {

    // Load scene source
    let path = format!("assets/scenes/{}.sav.ron", kajiya.scene_name);
    let source_scene_data = if let Ok(data) = fs::read_to_string(&path) {
        data
    } else {
        console_info!("FAILED to load scene from \"{}\"", &path);

        return;
    };
    
    // Write scene source to the active scene asset, then load the asset
    let path = format!("assets/scenes/{}.scn.ron", kajiya.scene_name);
    fs::write(&path, source_scene_data).expect("Unable to write file");

    let path = format!("scenes/{}.scn.ron", kajiya.scene_name);
    let scene_handle: Handle<DynamicScene> = asset_server.load(&path);

    scene_spawner.spawn_dynamic(scene_handle);
    console_info!("Loading scene from \"{}\"", &path);

}