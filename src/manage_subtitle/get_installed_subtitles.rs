use redb::ReadableTable;
use serde::{Deserialize, Serialize};
use serde_json::{to_string, from_slice};
use base64::{engine::general_purpose, Engine as _};

use redb::{ReadableDatabase, TableDefinition};
use std::collections::HashMap;

use crate::{global_types::Source, manage_subtitle::SubtitleDatabaseManager};



#[derive(Debug, Deserialize, Serialize)]
pub struct GetInstalledSubtitlesParams{
  pub source: Source,
  pub id: String,
  pub season_index: usize,
  pub episode_index: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetInstalledSubtitlesData{
  pub title: String,
  pub path: String
}


pub async fn new(db_manager: SubtitleDatabaseManager, params: &GetInstalledSubtitlesParams) -> anyhow::Result<HashMap<u64, GetInstalledSubtitlesData>>{

  
  let db = db_manager.get_db()?;

  let read_txn = db.begin_read()?;
  
  let raw_table = to_string(&[
    params.source.to_string(),
    params.id.clone(),
    params.season_index.to_string(),
    params.episode_index.to_string()
  ])?;

  let base64_encoded_table = general_purpose::STANDARD.encode(raw_table.as_bytes());

  let table_template: TableDefinition<u64, &[u8]> = TableDefinition::new(&base64_encoded_table);

  let table = read_txn.open_table(table_template)?;

  let mut result: HashMap<u64, GetInstalledSubtitlesData> = HashMap::new();

  for sub in table.iter()?{
    let (key, value) = sub?;
    let sub_value:Vec<&str> = from_slice(&value.value())?;


    let data = GetInstalledSubtitlesData{
      title: sub_value[0].to_string(),
      path: sub_value[1].to_string()
    };

    result.insert(key.value(), data);
  }
    


  return Ok(result);

}