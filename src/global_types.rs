use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Source{
  Anime,
  Movies,
  TV
}

impl Source{
  pub fn to_string(&self) -> String{
    match self {
      Source::Anime => "anime".to_string(),
      Source::Movies => "movies".to_string(),
      Source::TV => "tv".to_string()
    }
  }

  pub fn from_str(s: &str) -> Option<Source>{
    match s {
      "anime" => Some(Source::Anime),
      "movies" => Some(Source::Movies),
      "tv" => Some(Source::TV),
      _ => None
    }
  }
}