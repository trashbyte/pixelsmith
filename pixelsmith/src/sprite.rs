use std::path::PathBuf;
use yaml_rust::{EmitError, ScanError};
use crate::lights::LightingInfo;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug)]
pub enum SceneLoadError {
    YamlScanError(ScanError),
    YamlEmitError(EmitError),
    SerdeYamlError(serde_yaml::Error),
    Io(std::io::Error),
    WrongNumberOfDocuments
}
impl From<ScanError> for SceneLoadError {
    fn from(e: ScanError) -> Self {
        SceneLoadError::YamlScanError(e)
    }
}
impl From<EmitError> for SceneLoadError {
    fn from(e: EmitError) -> Self {
        SceneLoadError::YamlEmitError(e)
    }
}
impl From<std::io::Error> for SceneLoadError {
    fn from(e: std::io::Error) -> Self {
        SceneLoadError::Io(e)
    }
}
impl From<serde_yaml::Error> for SceneLoadError {
    fn from(e: serde_yaml::Error) -> Self {
        SceneLoadError::SerdeYamlError(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    pub viewports_open: [bool; 4],
    pub lighting: LightingInfo
}

impl SceneData {
    pub fn try_load(path: PathBuf) -> Result<Self, SceneLoadError> {
        let docs = yaml_rust::YamlLoader::load_from_str(std::fs::read_to_string(path)?.as_str())?;
        if docs.len() != 1 {
            return Err(SceneLoadError::WrongNumberOfDocuments);
        }
        let mut yaml_str = String::new();
        yaml_rust::YamlEmitter::new(&mut yaml_str).dump(&docs[0])?;
        let data = serde_yaml::from_str::<SceneData>(yaml_str.as_str())?;

        Ok(data)
    }
}
