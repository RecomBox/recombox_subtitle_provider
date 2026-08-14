use serde::{Deserialize, Serialize};
use visdom::Vis;
use std::collections::HashMap;

// Most of the code here is adapted from AI
// I'm too lazy to manually parse it.

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubtitleData {
    pub title: String,
    pub link: String,
}

pub async fn new(link: &str) -> anyhow::Result<HashMap<String, Vec<SubtitleData>>> {
    let mut result: HashMap<String, Vec<SubtitleData>> = HashMap::new();
    let mut retry = 0;

    while retry < 3 && result.is_empty() {
        retry += 1;

        let url = format!("https://subdl.com{}", link);

        let client = reqwest::Client::new();

        let res = match client
            .get(&url)
            .header(
                "user-agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3",
            )
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("request error: {}", e);
                continue;
            }
        };

        let html = match res.text().await {
            Ok(h) => h,
            Err(e) => {
                eprintln!("failed reading response body: {}", e);
                continue;
            }
        };

        let vis = match Vis::load(html) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to parse html: {}", e);
                continue;
            }
        };

        // The block that lists every language is marked with data-list-view.
        let wrapper = vis.find(r#"[data-list-view=""]"#);
        if wrapper.length() == 0 {
            eprintln!("could not find [data-list-view] wrapper - page structure may have changed");
            continue;
        }

        // IMPORTANT: each language is its own *direct child* `div` of the
        // wrapper. Using `.find("div")` here (like the original code did)
        // recurses into every nested div inside the wrapper - list rows,
        // nested containers, etc. - which both massively over-iterates and,
        // combined with the `insert()` below, silently wipes out already
        // collected results whenever the same language div is revisited.
        // `.children("div")` restricts us to the top-level per-language
        // blocks only.
        let lang_blocks = wrapper.children("div");

        for lang_block_dom in lang_blocks {
            let lang_block = Vis::dom(&lang_block_dom);

            // Language name lives in the first h2 inside this block.
            let raw_lang = lang_block.find("h2").first().text();
            let lang_trimmed = raw_lang.trim();
            if lang_trimmed.is_empty() {
                // Not a language block (or structure didn't match) - skip
                // instead of creating a bogus "" entry.
                continue;
            }
            let lang = html_escape::decode_html_entities(lang_trimmed).to_string();

            // `entry().or_insert_with()` accumulates subtitles for a
            // language across iterations instead of overwriting them.
            let entry = result.entry(lang).or_insert_with(Vec::new);

            let li_ele_list = lang_block.find("li.flex.justify-between.flex-col");

            for li_dom in li_ele_list {
                let li_ele = Vis::dom(&li_dom);

                let raw_title = li_ele.find("h4").first().text();
                if raw_title.trim().is_empty() {
                    continue;
                }
                let title = html_escape::decode_html_entities(raw_title.trim()).to_string();

                let down_btn = li_ele.find(r#"button[title="Quick Download"]"#);
                if down_btn.length() == 0 {
                    // Some rows (e.g. pending uploads) have no download
                    // button - skip that row instead of erroring out and
                    // discarding everything collected so far.
                    continue;
                }

                let href = down_btn.parent("a").attr("href");
                let link = match href {
                    Some(h) => h.to_string(),
                    None => continue,
                };

                entry.push(SubtitleData { title, link });
            }
        }
    }

    Ok(result)
}