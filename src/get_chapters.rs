use serde::{Deserialize, Serialize};
use visdom::Vis;

use crate::{global_types::Source, search};

#[derive(Debug, Deserialize, Serialize)]
pub struct GetChaptersParams{
  pub imdb_id: String,
  pub source: Source,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct ChapterData{
  pub title: String,
  pub link: String
}


pub async fn new(get_chapters_params: &GetChaptersParams) -> anyhow::Result<Vec<ChapterData>>{

  let search_params = search::SearchParams{
    source: get_chapters_params.source.clone(),
    imdb_id: get_chapters_params.imdb_id.clone()
  };

  let search_result = match search::new(&search_params).await?{
    Some(s) => s,
    None => return Ok(vec![])
  };
  

  let url = format!("https://subdl.com{}", search_result.link);
  
  let client = reqwest::Client::new();

  let res = client.get(url).send().await?;

  let html = res.text().await?;

  let vis = Vis::load(html)
    .map_err(|e| anyhow::anyhow!(e))?;

  let ele_wrap = vis.find(r#".mt-5.flex.flex-col.gap-4[style="direction:ltr"]"#);

  let a_ele_li = ele_wrap.find("a");

  let mut result = Vec::new();

  for ele_trait in a_ele_li{
    let ele = Vis::dom(&ele_trait);

    let link = match ele.attr("href") {
      Some(l) => l.to_string(),
      None => "".to_string()
    };
    
    let title = ele.find(".text-xl.font-bold").text();

    result.push(ChapterData{
      title: title.to_string(),
      link: link.to_string()
    });

  }



  return Ok(result);

}