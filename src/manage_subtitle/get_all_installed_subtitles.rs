use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_str};
use base64::{engine::general_purpose, Engine as _};
use redb::{ReadableDatabase, ReadableMultimapTable};


use crate::{ manage_subtitle::{INSTALLED_SUBTITLES_TABLE, MAP_SUBTITLES_TABLE, SubtitleDatabaseManager}};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetAllInstalledSubtitlesData{
  pub source: String,
  pub id: String,
  pub subtitle_id: u64,
  pub title: String,
  pub path: String
}




pub async fn new(db_manager: SubtitleDatabaseManager) -> anyhow::Result<Vec<GetAllInstalledSubtitlesData>>{

  
  let db = db_manager.get_db()?;

  let read_txn = db.begin_read()?;
  
  // let raw_key = to_string(&[
  //   params.source.to_string(),
  //   params.id.clone(),
  //   params.season_index.to_string(),
  //   params.episode_index.to_string()
  // ])?;

  // let base64_encoded_map_key = general_purpose::STANDARD.encode(raw_key.as_bytes());


  let installed_sub_table = read_txn.open_table(INSTALLED_SUBTITLES_TABLE)?;
  let map_sub_table = read_txn.open_multimap_table(MAP_SUBTITLES_TABLE)?;

  let mut result = Vec::new();

  for entry in map_sub_table.iter()?{
    let (key, value_entry) = entry?;

    let base64_dencoded_map_key = general_purpose::STANDARD.decode(key.value())?;
    let raw_key = String::from_utf8(base64_dencoded_map_key)?;

    let serde_key:Vec<&str> = from_str(&raw_key)?;

    let source = serde_key[0].to_string();
    let id = serde_key[1].to_string();

    for value in value_entry{
      let sub_id = value?.value();

      let sub = match installed_sub_table.get(sub_id)? {
        Some(sub) => sub,
        None => continue,
      };

      let base64_decoded_value = general_purpose::STANDARD.decode(sub.value())?;

      let sub_value:Vec<&str> = from_slice(&base64_decoded_value)?;

      result.push(GetAllInstalledSubtitlesData{
        source:source.clone(),
        id:id.clone(),
        subtitle_id:sub_id,
        title: sub_value[0].to_string(),
        path: sub_value[1].to_string()
      });

    }

  }
  

  Ok(result)

}