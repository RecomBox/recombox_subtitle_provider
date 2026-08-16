use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::{global_types::Source, manage_subtitle::SubtitleDatabaseManager};



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInstalledSubtitlesParams{
  pub source: Source,
  pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInstalledSubtitlesData{
  pub title: String,
  pub path: String
}


pub async fn new(db_manager: SubtitleDatabaseManager, params: &GetInstalledSubtitlesParams) -> anyhow::Result<HashMap<u64, GetInstalledSubtitlesData>>{

  let db = db_manager.get_db().await?;

  let source = params.source.to_string();
  let media_id = params.id.clone();

  let result = db.call(move |conn| -> Result<HashMap<u64, GetInstalledSubtitlesData>, tokio_rusqlite::rusqlite::Error> {
    let mut stmt = conn.prepare(
      "SELECT id, title, path FROM subtitles WHERE source = ?1 AND media_id = ?2"
    )?;

    let rows = stmt.query_map(
      tokio_rusqlite::rusqlite::params![source, media_id],
      |row| {
        let id: i64 = row.get(0)?;
        let title: String = row.get(1)?;
        let path: String = row.get(2)?;

        Ok((id as u64, GetInstalledSubtitlesData{ title, path }))
      }
    )?;

    let mut result: HashMap<u64, GetInstalledSubtitlesData> = HashMap::new();

    for row in rows {
      let (id, data) = row?;
      result.insert(id, data);
    }

    Ok(result)
  }).await?;

  Ok(result)

}
