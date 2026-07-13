use redb::{Database, MultimapTableDefinition};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{RwLock, Arc};
use redb::TableDefinition;

use std::fs;
use once_cell::sync::Lazy;

use crate::manage_subtitle::get_all_installed_subtitles::GetAllInstalledSubtitlesData;
use crate::manage_subtitle::get_installed_subtitles::GetInstalledSubtitlesData;

pub mod install_subtitle;
pub mod get_installed_subtitles;
pub mod remove_installed_subtitle;
pub mod get_all_installed_subtitles;

static DATABASE: Lazy<RwLock<Option<Arc<Database>>>> = Lazy::new(|| RwLock::new(None));


const DATABASE_NAME: &str = "subtitles_v2.redb";

pub const MAP_SUBTITLES_TABLE: MultimapTableDefinition<&str, u64> = MultimapTableDefinition::new("map_subtitles");
pub const INSTALLED_SUBTITLES_TABLE: TableDefinition<u64, &str> = TableDefinition::new("installed_subtitles");


pub struct SubtitleDatabaseManager{
    pub subtitle_directory: PathBuf
}

impl SubtitleDatabaseManager{
  
  pub fn get_db(self) -> anyhow::Result<Arc<Database>> {
    let db_dir = PathBuf::from(&self.subtitle_directory);

    fs::create_dir_all(&db_dir)?;

    let db_path = PathBuf::from(&db_dir)
        .join(DATABASE_NAME);
    
    if fs::exists(&db_path)? {
        let read_gaurd = DATABASE.read()
          .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        if let Some(db) = read_gaurd.clone() {
          return Ok(db);
        }
    }

    let mut write_gaurd = DATABASE.write()
      .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let raw_db = match Database::create(&db_path)
        .map_err(|e| e.to_string()){
          Ok(db) => db,
          Err(_) => {
            if db_path.exists() {
                fs::remove_file(&db_path)?;
            }
            Database::create(&db_path)?
          }
        };

    let db = Arc::new(raw_db);
    
    *write_gaurd = Some(db.clone());
    return Ok(db.clone());
    
  }

  pub async fn install(self, params: &install_subtitle::InstallSubtitleParams) -> anyhow::Result<()>{
    install_subtitle::new(self, params).await
  }

  pub async fn get_installed(self, params: &get_installed_subtitles::GetInstalledSubtitlesParams) -> anyhow::Result<HashMap<u64, GetInstalledSubtitlesData>>{
    get_installed_subtitles::new(self, params).await
  }

  pub async fn get_all_installed(self) -> anyhow::Result<Vec<GetAllInstalledSubtitlesData>>{
    get_all_installed_subtitles::new(self).await
  }

  pub async fn remove_installed(self, params: &remove_installed_subtitle::RemoveInstalledSubtitlesParams) -> anyhow::Result<()>{
    remove_installed_subtitle::new(self, params).await
  }
}
