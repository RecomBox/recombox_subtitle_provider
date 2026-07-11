use serde::{Deserialize, Serialize};
use serde_json::{from_slice};

use redb::{ReadableDatabase, ReadableTable};


use crate::{ manage_subtitle::{INSTALLED_SUBTITLES_TABLE, SubtitleDatabaseManager}};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetAllInstalledSubtitlesData{
  pub id: u64,
  pub title: String,
  pub path: String
}




pub async fn new(db_manager: SubtitleDatabaseManager) -> anyhow::Result<Vec<GetAllInstalledSubtitlesData>>{

  
  let db = db_manager.get_db()?;

  let read_txn = db.begin_read()?;
  

  let installed_sub_table = read_txn.open_table(INSTALLED_SUBTITLES_TABLE)?;

  let mut result = Vec::new();

  for guard in installed_sub_table.iter()?{
    let (k, v) = guard?;

    let sub_value:Vec<&str> = from_slice(&v.value())?;

    result.push(GetAllInstalledSubtitlesData{
      id: k.value(),
      title: sub_value[0].to_string(),
      path: sub_value[1].to_string()
    })

  }


  Ok(result)

}