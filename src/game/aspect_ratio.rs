use crate::engine_constants::DataType;
use crate::framework::viewport::ResolvedAspect;

/// User-facing aspect-ratio setting. Stored as a string in `settings.json` and parsed back into
/// this enum at runtime so unknown or malformed values fall back to `Default` without breaking
/// the settings file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectRatio {
    /// Edition-dependent default, resolved via [`DataType`] at runtime.
    Default,
    /// Fill the window; viewport takes the window aspect.
    Unrestricted,
    /// Fill the window exactly, pixel-for-pixel (pre-refactor behaviour).
    Stretch,
    /// Lock to a specific aspect.
    Locked { w: u32, h: u32 },
}

impl AspectRatio {
    /// Parse a settings-file string into an `AspectRatio`.
    /// Accepts `"default"`, `"unrestricted"`, `"stretch"`, and `"W:H"` forms.
    /// Falls back to `Default` on any parse error.
    pub fn parse(s: &str) -> AspectRatio {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("default") {
            return AspectRatio::Default;
        }
        if trimmed.eq_ignore_ascii_case("unrestricted") {
            return AspectRatio::Unrestricted;
        }
        if trimmed.eq_ignore_ascii_case("stretch") {
            return AspectRatio::Stretch;
        }

        if let Some((w_str, h_str)) = trimmed.split_once(':') {
            if let (Ok(w), Ok(h)) = (w_str.trim().parse::<u32>(), h_str.trim().parse::<u32>()) {
                if w > 0 && h > 0 {
                    return AspectRatio::Locked { w, h };
                }
            }
        }

        log::warn!("Invalid aspect ratio {:?}, falling back to default.", s);
        AspectRatio::Default
    }

    pub fn stringify(&self) -> String {
        match self {
            AspectRatio::Default => "default".to_owned(),
            AspectRatio::Unrestricted => "unrestricted".to_owned(),
            AspectRatio::Stretch => "stretch".to_owned(),
            AspectRatio::Locked { w, h } => format!("{}:{}", w, h),
        }
    }

    /// Resolves `Default` to an edition-specific locked aspect. Other variants pass through.
    pub fn resolve(self, data_type: Option<DataType>) -> ResolvedAspect {
        match self {
            AspectRatio::Unrestricted => ResolvedAspect::Unrestricted,
            AspectRatio::Stretch => ResolvedAspect::Stretch,
            AspectRatio::Locked { w, h } => ResolvedAspect::Locked { w, h },
            AspectRatio::Default => match data_type {
                Some(DataType::Switch) => ResolvedAspect::Locked { w: 16, h: 9 },
                Some(DataType::CS3D) => ResolvedAspect::Locked { w: 5, h: 4 },
                _ => ResolvedAspect::Locked { w: 4, h: 3 },
            },
        }
    }
}
