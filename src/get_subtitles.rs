use serde::{Deserialize, Serialize};
use visdom::Vis;
use std::collections::HashMap;



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubtitleData{
  pub title: String,
  pub link: String
}


pub async fn new(link: &str) -> anyhow::Result<HashMap<String, Vec<SubtitleData>>>{
  
  let mut result = HashMap::new();
  let mut retry = 0;

  while retry < 3 && result.is_empty() {
    retry += 1;

    let url = format!("https://subdl.com{}", link);
  
    let client = reqwest::Client::new();

    let res = match client.get(url).send().await{
      Ok(r) => r,
      Err(e) => {
        eprintln!("{}", e);
        continue;
      }
    };

    let html = res.text().await?;



    let vis = Vis::load(html)
      .map_err(|e| anyhow::anyhow!(e))?;

    let wrapper = vis.find(r#"[data-list-view=""]"#);

    for lang_item_dom in wrapper.find("div"){
      let lang_item_ele = Vis::dom(&lang_item_dom);
      
      let lang = lang_item_ele.find("div").first()
        .find("h2").text()
        .to_string();


      result.insert(lang.to_string(), vec![]);

      let li_ele_list = lang_item_ele.find("div").find("li.flex.justify-between.flex-col");

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

    
  }
  


  Ok(result)

}