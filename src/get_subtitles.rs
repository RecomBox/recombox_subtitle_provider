use serde::{Deserialize, Serialize};
use visdom::Vis;
use std::collections::HashMap;



#[derive(Debug, Deserialize, Serialize)]
pub struct SubtitleData{
  pub title: String,
  pub link: String
}


pub async fn new(link: &str) -> anyhow::Result<HashMap<String, Vec<SubtitleData>>>{


  let url = format!("https://subdl.com{}", link);
  
  let client = reqwest::Client::new();

  let res = client.get(url).send().await?;

  let html = res.text().await?;

  let vis = Vis::load(html)
    .map_err(|e| anyhow::anyhow!(e))?;

  let wrapper = vis.find(r#"[dir="ltr"]"#).find("div").first();

  let mut result = HashMap::new();

  for lang_item_dom in wrapper.find(".flex.flex-col.mt-4.select-none"){
    let lang_item_ele = Vis::dom(&lang_item_dom);

    let lang = lang_item_ele.find("h2.text-lg.font-semibold").text();

    result.insert(lang.to_string(), vec![]);

    let li_ele_list = lang_item_ele.find("li.flex.justify-between.flex-col");

    for li_dom in li_ele_list{
      let li_ele = Vis::dom(&li_dom);
      let raw_title = li_ele.find("h4").text();
      let title = html_escape::decode_html_entities(&raw_title).to_string();

      let down_btn = li_ele.find(r#"button[title="Quick Download"]"#);

      let link = down_btn.parent("a")
        .attr("href")
        .ok_or(anyhow::anyhow!("Missing Link"))?
        .to_string();
      
      result.get_mut(&lang).unwrap().push(SubtitleData{
        title: title,
        link: link
      });
      
    }

  }


  Ok(result)

}