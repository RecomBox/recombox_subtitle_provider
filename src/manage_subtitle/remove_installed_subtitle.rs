use serde::{Deserialize, Serialize};
use tokio_rusqlite::rusqlite::OptionalExtension;

use std::fs;
use std::path::PathBuf;

use crate::{global_types::Source, manage_subtitle::SubtitleDatabaseManager};



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoveInstalledSubtitlesParams{
  pub source: Source,
  pub id: String,
  pub subtitle_id: u64,
}



pub async fn new(db_manager: SubtitleDatabaseManager, params: &RemoveInstalledSubtitlesParams) -> anyhow::Result<()>{

  let db = db_manager.get_db().await?;

  let source = params.source.to_string();
  let media_id = params.id.clone();
  let subtitle_id = params.subtitle_id as i64;

  // Look up the file path first, since removing it from disk is a
  // blocking call we don't want to do inside the db worker thread's
  // transaction.
  let path: Option<String> = {
    let source = source.clone();
    let media_id = media_id.clone();

    db.call(move |conn| -> Result<Option<String>, tokio_rusqlite::rusqlite::Error> {
      conn.query_row(
        "SELECT path FROM subtitles WHERE id = ?1 AND source = ?2 AND media_id = ?3",
        tokio_rusqlite::rusqlite::params![subtitle_id, source, media_id],
        |row| row.get(0)
      ).optional()
    }).await?
  };

  let Some(path) = path else {
    return Ok(());
  };

  let sub_path = PathBuf::from(&path);

  if sub_path.exists() {
    fs::remove_file(&sub_path)?;
  }

  db.call(move |conn| -> Result<(), tokio_rusqlite::rusqlite::Error> {
    conn.execute(
      "DELETE FROM subtitles WHERE id = ?1 AND source = ?2 AND media_id = ?3",
      tokio_rusqlite::rusqlite::params![subtitle_id, source, media_id],
    )?;

    Ok(())
  }).await?;

  Ok(())

}
