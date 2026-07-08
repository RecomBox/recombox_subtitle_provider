use serde::{Deserialize, Serialize};
use serde_json::{to_string, from_slice};
use base64::{engine::general_purpose, Engine as _};

use redb::ReadableDatabase;
use std::collections::HashMap;

use crate::{global_types::Source, manage_subtitle::{INSTALLED_SUBTITLES_TABLE, MAP_SUBTITLES_TABLE, SubtitleDatabaseManager}};



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInstalledSubtitlesParams{
  pub source: Source,
  pub id: String,
  pub season_index: usize,
  pub episode_index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInstalledSubtitlesData{
  pub title: String,
  pub path: String
}


pub async fn new(db_manager: SubtitleDatabaseManager, params: &GetInstalledSubtitlesParams) -> anyhow::Result<HashMap<u64, GetInstalledSubtitlesData>>{

  
  let db = db_manager.get_db()?;

  let read_txn = db.begin_read()?;
  
  let raw_key = to_string(&[
    params.source.to_string(),
    params.id.clone(),
    params.season_index.to_string(),
    params.episode_index.to_string()
  ])?;

  let base64_encoded_map_key = general_purpose::STANDARD.encode(raw_key.as_bytes());


  let installed_sub_table = read_txn.open_table(INSTALLED_SUBTITLES_TABLE)?;
  let map_sub_table = read_txn.open_multimap_table(MAP_SUBTITLES_TABLE)?;


  let values = map_sub_table.get(base64_encoded_map_key.as_str())?.into_iter();

  let mut result: HashMap<u64, GetInstalledSubtitlesData> = HashMap::new();

  for entry in values{
    let sub_id = entry?;


    let sub = match installed_sub_table.get(sub_id.value())? {
      Some(sub) => sub,
      None => continue,
    };

    let sub_value:Vec<&str> = from_slice(&sub.value())?;

    result.insert(sub_id.value(), GetInstalledSubtitlesData{
      title: sub_value[0].to_string(),
      path: sub_value[1].to_string()
    });
  }
  


  return Ok(result);

}