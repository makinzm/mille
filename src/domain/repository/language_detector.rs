/// Maps a file extension to a language name.
pub trait LanguageDetector {
    fn detect_from_extension(&self, ext: &str) -> Option<String>;
}
