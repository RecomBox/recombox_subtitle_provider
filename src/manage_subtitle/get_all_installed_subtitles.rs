use serde::{Deserialize, Serialize};

use crate::manage_subtitle::SubtitleDatabaseManager;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetAllInstalledSubtitlesData{
  pub source: String,
  pub id: String,
  pub subtitle_id: u64,
  pub title: String,
  pub path: String
}




pub async fn new(db_manager: SubtitleDatabaseManager) -> anyhow::Result<Vec<GetAllInstalledSubtitlesData>>{

  let db = db_manager.get_db().await?;

  let result = db.call(|conn| -> Result<Vec<GetAllInstalledSubtitlesData>, tokio_rusqlite::rusqlite::Error> {
    let mut stmt = conn.prepare(
      "SELECT id, source, media_id, title, path FROM subtitles"
    )?;

    let rows = stmt.query_map([], |row| {
      let subtitle_id: i64 = row.get(0)?;
      let source: String = row.get(1)?;
      let id: String = row.get(2)?;
      let title: String = row.get(3)?;
      let path: String = row.get(4)?;

      Ok(GetAllInstalledSubtitlesData{
        source,
        id,
        subtitle_id: subtitle_id as u64,
        title,
        path
      })
    })?;

    let mut result = Vec::new();

    for row in rows {
      result.push(row?);
    }

    Ok(result)
  }).await?;

  Ok(result)

}
