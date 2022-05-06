use std::path::PathBuf;
use crate::sprite::SceneData;

#[derive(Debug, Clone)]
pub struct ProjectData {
    pub path: PathBuf,
}

impl ProjectData {
    pub fn ini_path(&self) -> PathBuf {
        self.path.join("imgui.ini")
    }

    pub fn find_sprites(&self) -> Vec<(PathBuf, SceneData)> {
        let mut sprites = Vec::new();
        for d in std::fs::read_dir(self.path.join("sprites")).unwrap() {
            match d {
                Ok(dir) => {
                    match SceneData::try_load(dir.path().join("scene.yaml")) {
                        Ok(data) => sprites.push((dir.path(), data)),
                        Err(e) => println!("{:?}", e)
                    }
                }
                Err(e) => println!("{:?}", e)
            }
        }
        sprites
    }
}