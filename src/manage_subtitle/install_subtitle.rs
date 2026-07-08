use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_vec};
use base64::{engine::general_purpose, Engine as _};

use std::path::PathBuf;
use std::fs;
use std::io;
use snowid::SnowID;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use zip::ZipArchive;

use crate::manage_subtitle::INSTALLED_SUBTITLES_TABLE;
use crate::manage_subtitle::MAP_SUBTITLES_TABLE;
use crate::{global_types::Source, manage_subtitle::SubtitleDatabaseManager};



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallSubtitleParams{
  pub source: Source,
  pub id: String,
  pub season_index: usize,
  pub episode_index: usize,
  pub language: String,
  pub link: String,
}


pub async fn new(db_manager: SubtitleDatabaseManager, install_subtitle_params: &InstallSubtitleParams) -> anyhow::Result<()>{

  let client = reqwest::Client::new();

  let subtitle_dir = PathBuf::from(&db_manager.subtitle_directory)
    .join(&install_subtitle_params.source.to_string())
    .join(&install_subtitle_params.id)
    .join(&install_subtitle_params.season_index.to_string())
    .join(&install_subtitle_params.episode_index.to_string());

  fs::create_dir_all(&subtitle_dir)?;

  let download_dir = subtitle_dir.join("download");

  fs::create_dir_all(&download_dir)?;

  let generator = SnowID::new(1)?;
  let zip_id = generator.generate();

  let zip_path = download_dir.join(format!("{}.zip", zip_id));
  

  let res = client.get(install_subtitle_params.link.clone()).send().await?;
  
  let mut file = tokio::fs::File::create(&zip_path).await?;

  
  let mut stream = res.bytes_stream();

  while let Some(chunk) = stream.next().await {
    let data = chunk?;
    file.write_all(&data).await?;
  }

  let target_dir = download_dir.join(zip_id.to_string());

  fs::create_dir_all(&target_dir)?;

  let mut archive = ZipArchive::new(fs::File::open(&zip_path)?)?;

  for i in 0..archive.len() {
    let mut entry = archive.by_index(i)?;
    let outpath = PathBuf::from(&target_dir).join(entry.name());

    if entry.is_dir() {
      std::fs::create_dir_all(&outpath)?;
    } else {
      if let Some(parent) = outpath.parent() {
          std::fs::create_dir_all(parent)?;
      }
      let mut outfile = fs::File::create(&outpath)?;
      io::copy(&mut entry, &mut outfile)?;
    }
  }

  let mut sub_path_vec = Vec::new();

  for entry in walkdir::WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
    let path = entry.path();

    if path.is_file() {
      if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        
        let ext = ext.to_lowercase();
        if ["srt", "vtt", "ass", "ssa"].contains(&ext.as_str()) {
          sub_path_vec.push(path.to_path_buf());
        }
      }
    }
  }
  
  let db = db_manager.get_db()?;

  let write_txn = db.begin_write()?;
  {
    let raw_key = to_string(&[
      install_subtitle_params.source.to_string(),
      install_subtitle_params.id.clone(),
      install_subtitle_params.season_index.to_string(),
      install_subtitle_params.episode_index.to_string()
    ])?;

    let base64_encoded_map_key = general_purpose::STANDARD.encode(raw_key.as_bytes());


    let mut installed_subtitles_table = write_txn.open_table(INSTALLED_SUBTITLES_TABLE)?;
    let mut map_subtitles_table = write_txn.open_multimap_table(MAP_SUBTITLES_TABLE)?;
    
    for sub_path in sub_path_vec{
      let generator = SnowID::new(1)?;
      let sub_id = generator.generate();

      let file_stem = sub_path.file_stem()
        .ok_or("")
        .map_err(|e| anyhow::anyhow!(e))?
        .to_string_lossy().to_string();

      let file_ext = sub_path.extension()
        .ok_or("")
        .map_err(|e| anyhow::anyhow!(e))?
        .to_string_lossy().to_string();

      let title = format!("{} | {}", file_stem, install_subtitle_params.language);

      let move_path = subtitle_dir.join(format!("{}.{}", sub_id, file_ext));

      fs::create_dir_all(&subtitle_dir)?;

      fs::rename(&sub_path, &move_path)?;

      let encoded_value = to_vec(&[
        title,
        move_path.to_string_lossy().to_string(),
      ])?;

      installed_subtitles_table.insert(sub_id, encoded_value.as_slice())?;

      map_subtitles_table.insert(base64_encoded_map_key.as_str(), sub_id)?;
    }

  }
  write_txn.commit()?;

  fs::remove_dir_all(download_dir)?;

  Ok(())

}