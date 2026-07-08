pub mod search;
pub mod global_types;
pub mod get_chapters;
pub mod get_subtitles;
pub mod manage_subtitle;


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

use super::*;

    // #[tokio::test]
    async fn search() {
        let search_params = search::SearchParams{
            source: global_types::Source::Movies,
            imdb_id: "tt7865090".to_string()
        };

        let d = search::new(&search_params).await.unwrap();
        println!("{:#?}", d);
    }

    // #[tokio::test]
    async fn get_chapters() {

        let get_chapters_params = get_chapters::GetChaptersParams{
            imdb_id: "tt7865090".to_string(),
            source: global_types::Source::Anime
        };

        let d = get_chapters::new(&get_chapters_params).await.unwrap();
        println!("{:#?}", d);
    }


    // #[tokio::test]
    async fn get_subtitles() {

        let d = get_subtitles::new("/subtitle/sd1424260/the-outlaws").await.unwrap();
        println!("{:#?}", d);
    }

    // #[tokio::test]
    async fn install_subtitle() {

        let manager = manage_subtitle::SubtitleDatabaseManager{
            subtitle_directory: PathBuf::from("/home/goodday/Code/recombox_subtitle_provider/data")
        };

        let params = manage_subtitle::install_subtitle::InstallSubtitleParams{
            source: global_types::Source::Movies,
            id: "test".to_string(),
            season_index: 0,
            episode_index: 0,
            language: "English".to_string(),
            link: "https://dl.subdl.com/subtitle/1671015-2381556.zip".to_string()
            
        };
        
        manager.install(&params).await.unwrap();
    }

    #[tokio::test]
    async fn get_installed_subtitles() {

        let manager = manage_subtitle::SubtitleDatabaseManager{
            subtitle_directory: PathBuf::from("/home/goodday/Code/recombox_subtitle_provider/data")
        };

        let params = manage_subtitle::get_installed_subtitles::GetInstalledSubtitlesParams{
            source: global_types::Source::Movies,
            id: "test".to_string(),
            season_index: 0,
            episode_index: 0,
        };
        
        let d = manager.get_installed(&params).await.unwrap();

        println!("{:#?}", d);
    }
}
