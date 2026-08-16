use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::fs;
use once_cell::sync::Lazy;
use tokio_rusqlite::Connection;

use crate::manage_subtitle::get_all_installed_subtitles::GetAllInstalledSubtitlesData;
use crate::manage_subtitle::get_installed_subtitles::GetInstalledSubtitlesData;

pub mod install_subtitle;
pub mod get_installed_subtitles;
pub mod remove_installed_subtitle;
pub mod get_all_installed_subtitles;

static DATABASE: Lazy<RwLock<Option<Arc<Connection>>>> = Lazy::new(|| RwLock::new(None));

const DATABASE_NAME: &str = "subtitles_v2.sqlite";

pub struct SubtitleDatabaseManager{
    pub subtitle_directory: PathBuf
}

impl SubtitleDatabaseManager{

  pub async fn get_db(&self) -> anyhow::Result<Arc<Connection>> {
    {
      let read_guard = DATABASE.read()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

      if let Some(db) = read_guard.clone() {
        return Ok(db);
      }
    }

    let mut write_guard = DATABASE.write()
      .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Another task may have created the connection while we were
    // waiting on the write lock.
    if let Some(db) = write_guard.clone() {
      return Ok(db);
    }

    let db_dir = PathBuf::from(&self.subtitle_directory);

    fs::create_dir_all(&db_dir)?;

    let db_path = db_dir.join(DATABASE_NAME);

    let conn = Connection::open(&db_path).await?;

    conn.call(|conn| {
      conn.execute(
        "CREATE TABLE IF NOT EXISTS subtitles (
          id       INTEGER PRIMARY KEY,
          source   TEXT NOT NULL,
          media_id TEXT NOT NULL,
          title    TEXT NOT NULL,
          path     TEXT NOT NULL
        )",
        [],
      )?;

      conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_subtitles_source_media_id
          ON subtitles (source, media_id)",
        [],
      )?;

      Ok(())
    }).await?;

    let db = Arc::new(conn);

    *write_guard = Some(db.clone());

    Ok(db)
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
