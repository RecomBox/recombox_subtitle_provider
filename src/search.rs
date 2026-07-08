use serde::{Deserialize, Serialize};
use serde_json::Value;


use crate::global_types::Source;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchParams{
  pub imdb_id: String,
  pub source: Source,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchData{
  pub title: String,
  pub poster_url: String,
  pub link: String

}


pub async fn new(search_params: &SearchParams) -> anyhow::Result<Option<SearchData>>{
  
  let url = format!("https://api3.subdl.com/auto?query={}", search_params.imdb_id);
  println!("{}", url);
  let client = reqwest::Client::new();

  let res = client.get(url).send().await?;

  let data: Value = res.json().await?;

  let list = data.get("results")
    .ok_or(anyhow::anyhow!("Missing Results"))?
    .as_array()
    .ok_or(anyhow::anyhow!("Invalid Results Array"))?;

  
  let item_opt = list.get(0);

  if let Some(item) = item_opt {
    let title = item.get("name")
      .ok_or(anyhow::anyhow!("Missing Title"))?.as_str().ok_or(anyhow::anyhow!("Invalid Title"))?;
    let link = item.get("link")
      .ok_or(anyhow::anyhow!("Missing Url"))?.as_str().ok_or(anyhow::anyhow!("Invalid Url"))?;
    
    let poster_url = item.get("poster_url")
      .ok_or(anyhow::anyhow!("Missing Poster"))?.as_str().ok_or(anyhow::anyhow!("Invalid Poster"))?;
    
    let data = SearchData{
      title: title.to_string(),
      link: link.to_string(),
      poster_url: poster_url.to_string()
    };

    return Ok(Some(data));
  }else{
    return Ok(None);
  }
}