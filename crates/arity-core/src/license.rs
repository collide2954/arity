use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The license elements of a Creative Commons license.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CcElements {
    By,
    BySa,
    ByNd,
    ByNc,
    ByNcSa,
    ByNcNd,
}

impl CcElements {
    const ALL: [(Self, &'static str); 6] = [
        (Self::By, "by"),
        (Self::BySa, "by-sa"),
        (Self::ByNd, "by-nd"),
        (Self::ByNc, "by-nc"),
        (Self::ByNcSa, "by-nc-sa"),
        (Self::ByNcNd, "by-nc-nd"),
    ];

    fn code(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(elements, _)| *elements == self)
            .map_or("by", |(_, code)| code)
    }

    fn from_code(code: &str) -> Option<Self> {
        let code = if code == "by-nd-nc" { "by-nc-nd" } else { code };
        Self::ALL
            .iter()
            .find(|(_, candidate)| *candidate == code)
            .map(|(elements, _)| *elements)
    }
}

/// The license of a work, as its source states it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum License {
    CreativeCommons {
        elements: CcElements,
        version: Option<String>,
    },
    Cc0,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the license text is empty")]
pub struct EmptyLicense;

impl License {
    /// True when anyone can copy and share the work without asking.
    pub fn allows_redistribution(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }

    pub fn allows_commercial_use(&self) -> bool {
        match self {
            Self::CreativeCommons { elements, .. } => !matches!(
                elements,
                CcElements::ByNc | CcElements::ByNcSa | CcElements::ByNcNd
            ),
            Self::Cc0 => true,
            Self::Unknown(_) => false,
        }
    }

    pub fn allows_adaptation(&self) -> bool {
        match self {
            Self::CreativeCommons { elements, .. } => {
                !matches!(elements, CcElements::ByNd | CcElements::ByNcNd)
            }
            Self::Cc0 => true,
            Self::Unknown(_) => false,
        }
    }

    pub fn requires_attribution(&self) -> bool {
        !matches!(self, Self::Cc0)
    }

    /// A short label for people, for example `CC BY 4.0`.
    pub fn label(&self) -> String {
        match self {
            Self::CreativeCommons { elements, version } => {
                let elements = elements.code().to_uppercase();
                match version {
                    Some(version) => format!("CC {elements} {version}"),
                    None => format!("CC {elements}"),
                }
            }
            Self::Cc0 => "CC0 1.0".to_owned(),
            Self::Unknown(text) => text.clone(),
        }
    }

    pub fn url(&self) -> Option<String> {
        match self {
            Self::CreativeCommons {
                elements,
                version: Some(version),
            } => Some(format!(
                "https://creativecommons.org/licenses/{}/{version}/",
                elements.code()
            )),
            Self::CreativeCommons { version: None, .. } | Self::Unknown(_) => None,
            Self::Cc0 => Some("https://creativecommons.org/publicdomain/zero/1.0/".to_owned()),
        }
    }
}

impl FromStr for License {
    type Err = EmptyLicense;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let original = text.trim();
        if original.is_empty() {
            return Err(EmptyLicense);
        }
        let lower = original.to_lowercase();
        Ok(from_url(&lower)
            .or_else(|| from_code(&lower))
            .unwrap_or_else(|| Self::Unknown(original.to_owned())))
    }
}

fn from_url(lower: &str) -> Option<License> {
    if lower.contains("creativecommons.org/publicdomain/zero") {
        return Some(License::Cc0);
    }
    let (_, path) = lower.split_once("creativecommons.org/licenses/")?;
    let mut segments = path.split('/');
    let elements = CcElements::from_code(segments.next()?)?;
    let version = segments.next().filter(|segment| is_version(segment));
    Some(License::CreativeCommons {
        elements,
        version: version.map(str::to_owned),
    })
}

fn from_code(lower: &str) -> Option<License> {
    let code: String = lower
        .chars()
        .map(|c| if c == ' ' || c == '_' { '-' } else { c })
        .collect();
    if matches!(code.as_str(), "cc0" | "cc-0" | "cc0-1.0" | "cc-zero") {
        return Some(License::Cc0);
    }
    let rest = code.strip_prefix("cc-")?;
    let (elements, version) = match rest.rsplit_once('-') {
        Some((head, tail)) if is_version(tail) => (head, Some(tail)),
        _ => (rest, None),
    };
    Some(License::CreativeCommons {
        elements: CcElements::from_code(elements)?,
        version: version.map(str::to_owned),
    })
}

fn is_version(text: &str) -> bool {
    text.split_once('.').is_some_and(|(major, minor)| {
        !major.is_empty()
            && !minor.is_empty()
            && major.bytes().all(|b| b.is_ascii_digit())
            && minor.bytes().all(|b| b.is_ascii_digit())
    })
}

impl fmt::Display for License {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreativeCommons { elements, version } => {
                let code = elements.code().to_uppercase();
                match version {
                    Some(version) => write!(f, "CC-{code}-{version}"),
                    None => write!(f, "CC-{code}"),
                }
            }
            Self::Cc0 => f.write_str("CC0-1.0"),
            Self::Unknown(text) => f.write_str(text),
        }
    }
}

impl TryFrom<String> for License {
    type Error = EmptyLicense;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<License> for String {
    fn from(license: License) -> Self {
        license.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cc(elements: CcElements, version: Option<&str>) -> License {
        License::CreativeCommons {
            elements,
            version: version.map(str::to_owned),
        }
    }

    #[test]
    fn parses_creative_commons_urls() {
        let cases = [
            (
                "http://creativecommons.org/licenses/by/4.0/",
                cc(CcElements::By, Some("4.0")),
            ),
            (
                "https://creativecommons.org/licenses/by-nc-nd/3.0/igo/",
                cc(CcElements::ByNcNd, Some("3.0")),
            ),
            (
                "https://creativecommons.org/licenses/by-sa/2.5",
                cc(CcElements::BySa, Some("2.5")),
            ),
            (
                "https://creativecommons.org/publicdomain/zero/1.0/",
                License::Cc0,
            ),
        ];
        for (text, expected) in cases {
            assert_eq!(text.parse::<License>(), Ok(expected), "{text}");
        }
    }

    #[test]
    fn parses_spdx_identifiers_and_short_codes() {
        let cases = [
            ("CC-BY-4.0", cc(CcElements::By, Some("4.0"))),
            ("CC-BY-NC-SA-4.0", cc(CcElements::ByNcSa, Some("4.0"))),
            ("cc by", cc(CcElements::By, None)),
            ("cc by-nc", cc(CcElements::ByNc, None)),
            ("CC BY-NC-ND", cc(CcElements::ByNcNd, None)),
            ("cc_by-nd-nc", cc(CcElements::ByNcNd, None)),
            ("CC0", License::Cc0),
            ("CC0-1.0", License::Cc0),
        ];
        for (text, expected) in cases {
            assert_eq!(text.parse::<License>(), Ok(expected), "{text}");
        }
    }

    #[test]
    fn keeps_other_text_as_reported() {
        assert_eq!(
            " Free to read ".parse::<License>(),
            Ok(License::Unknown("Free to read".to_owned()))
        );
        assert_eq!(
            "cc-unknown".parse::<License>(),
            Ok(License::Unknown("cc-unknown".to_owned()))
        );
        assert_eq!("  ".parse::<License>(), Err(EmptyLicense));
    }

    #[test]
    fn displays_spdx_style_identifiers_that_parse_back() {
        for license in [
            cc(CcElements::By, Some("4.0")),
            cc(CcElements::ByNcNd, None),
            License::Cc0,
            License::Unknown("author manuscript".to_owned()),
        ] {
            assert_eq!(license.to_string().parse::<License>(), Ok(license.clone()));
        }
        assert_eq!(
            cc(CcElements::ByNc, Some("3.0")).to_string(),
            "CC-BY-NC-3.0"
        );
    }

    #[test]
    fn states_the_permissions_of_each_license() {
        let by = cc(CcElements::By, Some("4.0"));
        assert!(by.allows_redistribution());
        assert!(by.allows_commercial_use());
        assert!(by.allows_adaptation());
        assert!(by.requires_attribution());

        let nc_nd = cc(CcElements::ByNcNd, Some("4.0"));
        assert!(nc_nd.allows_redistribution());
        assert!(!nc_nd.allows_commercial_use());
        assert!(!nc_nd.allows_adaptation());

        assert!(!License::Cc0.requires_attribution());

        let unknown = License::Unknown("free to read".to_owned());
        assert!(!unknown.allows_redistribution());
        assert!(!unknown.allows_commercial_use());
        assert!(unknown.requires_attribution());
    }

    #[test]
    fn has_labels_and_urls_for_people() {
        let by = cc(CcElements::By, Some("4.0"));
        assert_eq!(by.label(), "CC BY 4.0");
        assert_eq!(
            by.url().as_deref(),
            Some("https://creativecommons.org/licenses/by/4.0/")
        );
        assert_eq!(cc(CcElements::BySa, None).label(), "CC BY-SA");
        assert_eq!(cc(CcElements::BySa, None).url(), None);
        assert_eq!(License::Cc0.label(), "CC0 1.0");
    }
}
