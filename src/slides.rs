use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// T: The linked entity (Song, BibleVerse, etc.)
/// M: The media type (SongFile, PathBuf, etc.)
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct PresentationChapter<T, M> {
    pub slides: Vec<Slide<M>>,
    pub linked_entity: LinkedEntity<T, M>,
}

/// The linked entity defines a reference to a specific entity from which the presentation is derived.
/// It is most likely a song or Bible verse.
/// This crate just provides an abstract definition, the implementation is left to other Cantara crates.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum LinkedEntity<T, M> {
    /// A specific source file
    Source(T),

    /// A specific title
    Title(String),

    /// A specific media entity (e.g. song)
    Media(M),
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Slide<M> {
    pub slide_content: SlideContent,
    pub linked_file: Option<M>,
}

// --- Implementation Blocks (Where the bounds actually matter) ---
impl<T, M> PresentationChapter<T, M>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Debug,
    M: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Debug,
{
    pub fn new(slides: Vec<Slide<M>>, linked_entity: LinkedEntity<T, M>) -> Self {
        Self {
            slides,
            linked_entity,
        }
    }
}

impl<M> Slide<M>
where
    M: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Debug,
{
    pub fn new_empty_slide(black_background: bool) -> Self {
        Self {
            slide_content: SlideContent::Empty(EmptySlide { black_background }),
            linked_file: None,
        }
    }

    pub fn new_content_slide(
        main_text: String,
        spoiler_text: Option<String>,
        meta_text: Option<String>,
    ) -> Self {
        Self {
            slide_content: SlideContent::SingleLanguageMainContent(
                SingleLanguageMainContentSlide::new(
                    main_text.trim().to_string(),
                    spoiler_text.map(|s| s.trim().to_string()),
                    meta_text.map(|s| s.trim().to_string()),
                ),
            ),
            linked_file: None,
        }
    }

    pub fn new_title_slide(title_text: String, meta_text: Option<String>) -> Self {
        Self {
            slide_content: SlideContent::Title(TitleSlide {
                title_text: title_text.trim().to_string(),
                meta_text: meta_text.map(|s| s.trim().to_string()),
            }),
            linked_file: None,
        }
    }

    pub fn with_media(mut self, media: M) -> Self {
        self.linked_file = Some(media);
        self
    }

    pub fn has_spoiler(&self) -> bool {
        match &self.slide_content {
            SlideContent::SingleLanguageMainContent(s) => s.spoiler_text.is_some(),
            SlideContent::MultiLanguageMainContent(s) => !s.spoiler_text_vector.is_empty(),
            _ => false,
        }
    }

    pub fn has_meta_text(&self) -> bool {
        match &self.slide_content {
            SlideContent::SingleLanguageMainContent(s) => s.meta_text.is_some(),
            SlideContent::Title(s) => s.meta_text.is_some(),
            SlideContent::MultiLanguageMainContent(s) => s.meta_text.is_some(),
            _ => false,
        }
    }
}

// --- Content Definitions ---

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum SlideContent {
    SingleLanguageMainContent(SingleLanguageMainContentSlide),
    Title(TitleSlide),
    MultiLanguageMainContent(MultiLanguageMainContentSlide),
    SimplePicture(SimplePictureSlide),
    Empty(EmptySlide),
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct SingleLanguageMainContentSlide {
    pub main_text: String,
    pub spoiler_text: Option<String>,
    pub meta_text: Option<String>,
}

impl SingleLanguageMainContentSlide {
    fn new(main_text: String, spoiler_text: Option<String>, meta_text: Option<String>) -> Self {
        let filter = |s: Option<String>| s.filter(|v| !v.trim().is_empty());
        Self {
            main_text,
            spoiler_text: filter(spoiler_text),
            meta_text: filter(meta_text),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct MultiLanguageMainContentSlide {
    pub main_text_list: Vec<String>,
    pub spoiler_text_vector: Vec<String>,
    pub meta_text: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct EmptySlide {
    pub black_background: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TitleSlide {
    pub title_text: String,
    pub meta_text: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct SimplePictureSlide {
    pub picture_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mocking your actual domain objects
    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    pub struct Song {
        pub id: u32,
        pub title: String,
    }

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    pub struct SongFile {
        pub path: String,
    }

    // Creating type aliases makes the code much cleaner in the rest of your app
    type SongPresentation = PresentationChapter<Song, SongFile>;

    #[test]
    fn test_generic_presentation_creation() {
        let song = Song {
            id: 1,
            title: "Amazing Grace".to_string(),
        };
        let file = SongFile {
            path: "/assets/grace.mp3".to_string(),
        };

        let slide = Slide::new_content_slide(
            "Amazing grace, how sweet the sound".to_string(),
            None,
            Some("Verse 1".to_string()),
        )
        .with_media(file.clone());

        let presentation = SongPresentation::new(vec![slide], LinkedEntity::Source(song));

        assert_eq!(presentation.slides.len(), 1);
        assert!(presentation.slides[0].linked_file.is_some());

        // Verify we can access our custom SongFile data
        if let Some(ref media) = presentation.slides[0].linked_file {
            assert_eq!(media.path, "/assets/grace.mp3");
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let song = Song {
            id: 42,
            title: "Test Song".to_string(),
        };
        let presentation = PresentationChapter::<Song, SongFile>::new(
            vec![Slide::new_title_slide("Title".to_string(), None)],
            LinkedEntity::Source(song),
        );

        // Serialize to JSON
        let json = serde_json::to_string(&presentation).unwrap();

        // Deserialize back
        let deserialized: PresentationChapter<Song, SongFile> =
            serde_json::from_str(&json).unwrap();

        assert_eq!(presentation, deserialized);
        if let LinkedEntity::Source(s) = &deserialized.linked_entity {
            assert_eq!(s.id, 42);
        }
    }

    #[test]
    fn test_different_generic_types() {
        // This proves the library isn't tied to 'Song' anymore.
        // We can use simple Strings for everything.
        let presentation = PresentationChapter::<String, String>::new(
            vec![Slide::new_empty_slide(true).with_media("background.png".into())],
            LinkedEntity::Title("Simple Show".into()),
        );

        assert_eq!(
            presentation.linked_entity,
            LinkedEntity::Title("Simple Show".into())
        );
    }
}
