use redb::ReadableTable;
use serde::{Deserialize, Serialize};
use serde_json::{to_string, from_slice};
use base64::{engine::general_purpose, Engine as _};

use std::fs;
use std::path::PathBuf;

use crate::{global_types::Source, manage_subtitle::{INSTALLED_SUBTITLES_TABLE, MAP_SUBTITLES_TABLE, SubtitleDatabaseManager}};



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoveInstalledSubtitlesParams{
  pub source: Source,
  pub id: String,
  pub season_index: usize,
  pub episode_index: usize,
  pub subtitle_id: u64,
}



pub async fn new(db_manager: SubtitleDatabaseManager, params: &RemoveInstalledSubtitlesParams) -> anyhow::Result<()>{
  
  
  let db = db_manager.get_db()?;

  let write_txn = db.begin_write()?;
  {
    let raw_table = to_string(&[
      params.source.to_string(),
      params.id.clone(),
      params.season_index.to_string(),
      params.episode_index.to_string()
    ])?;

    let base64_encoded_table = general_purpose::STANDARD.encode(raw_table.as_bytes());

    let mut installed_sub_table = write_txn.open_table(INSTALLED_SUBTITLES_TABLE)?;
    let mut map_sub_table = write_txn.open_multimap_table(MAP_SUBTITLES_TABLE)?;

    
    {
      let sub = match installed_sub_table.get(params.subtitle_id)? {
        Some(sub) => sub,
        None => return Ok(()),
      };

      let sub_value:Vec<&str> = from_slice(&sub.value())?;

      let sub_path = PathBuf::from(&sub_value[1]);

      if sub_path.exists() {
        fs::remove_file(&sub_path)?;
      }
    }

    installed_sub_table.remove(params.subtitle_id)?;

    map_sub_table.remove(base64_encoded_table.as_str(), params.subtitle_id)?;

  }
  write_txn.commit()?;
    


  Ok(())

}