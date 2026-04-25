//! Caveman output compression for LLM responses.
//!
//! Inspired by <https://github.com/JuliusBrussee/caveman>.
//! Reduces LLM output tokens by ~75% while preserving full technical
//! accuracy. The agent responds in terse "caveman" prose — dropping
//! articles, filler, pleasantries, and hedging — but keeps every
//! technical term, code block, and error message exact.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Caveman output compression intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CavemanMode {
    /// Caveman disabled — normal verbose output.
    #[default]
    Off,
    /// No filler/hedging. Keep articles + full sentences. Professional but tight.
    Lite,
    /// Drop articles, fragments OK, short synonyms. Classic caveman. (default when enabled)
    Full,
    /// Abbreviate (DB/auth/config/req/res/fn/impl), strip conjunctions,
    /// arrows for causality (X → Y), one word when one word enough.
    Ultra,
}

impl fmt::Display for CavemanMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CavemanMode::Off => write!(f, "off"),
            CavemanMode::Lite => write!(f, "lite"),
            CavemanMode::Full => write!(f, "full"),
            CavemanMode::Ultra => write!(f, "ultra"),
        }
    }
}

impl CavemanMode {
    /// Whether caveman compression is active.
    pub fn is_active(&self) -> bool {
        !matches!(self, CavemanMode::Off)
    }
}

/// Build the caveman system-prompt injection for a given intensity.
///
/// Returns `None` when mode is `Off`.
/// The returned string is appended to the base system prompt so the LLM
/// responds in compressed caveman style.
pub fn caveman_system_prompt(mode: CavemanMode) -> Option<String> {
    if !mode.is_active() {
        return None;
    }

    let rules = match mode {
        CavemanMode::Lite => RULES_LITE,
        CavemanMode::Full => RULES_FULL,
        CavemanMode::Ultra => RULES_ULTRA,
        CavemanMode::Off => unreachable!(),
    };

    Some(format!(
        "{PREAMBLE}\n\n## Intensity: {mode}\n\n{rules}\n\n{AUTO_CLARITY}\n\n{BOUNDARIES}"
    ))
}

// ── Prompt fragments ────────────────────────────────────────────────────────

const PREAMBLE: &str = "\
OUTPUT COMPRESSION — respond terse like smart caveman. \
All technical substance stay. Only fluff die.\n\
\n\
Core rules:\n\
- Drop: articles (a/an/the), filler (just/really/basically/actually/simply), \
pleasantries (sure/certainly/of course/happy to), hedging.\n\
- Fragments OK. Short synonyms (big not extensive, fix not \"implement a solution for\").\n\
- Technical terms exact. Code blocks unchanged. Errors quoted exact.\n\
- Pattern: [thing] [action] [reason]. [next step].";

const RULES_LITE: &str = "\
No filler or hedging. Keep articles and full sentences. Professional but tight.\n\
Example — \"Why React component re-render?\"\n\
Answer: \"Your component re-renders because you create a new object reference \
each render. Wrap it in `useMemo`.\"";

const RULES_FULL: &str = "\
Drop articles, fragments OK, short synonyms. Classic caveman.\n\
Example — \"Why React component re-render?\"\n\
Answer: \"New object ref each render. Inline object prop = new ref = re-render. \
Wrap in `useMemo`.\"";

const RULES_ULTRA: &str = "\
Abbreviate (DB/auth/config/req/res/fn/impl), strip conjunctions, \
arrows for causality (X → Y), one word when one word enough.\n\
Example — \"Why React component re-render?\"\n\
Answer: \"Inline obj prop → new ref → re-render. `useMemo`.\"";

const AUTO_CLARITY: &str = "\
Auto-Clarity: drop caveman for security warnings, irreversible action \
confirmations, multi-step sequences where fragment order risks misread. \
Resume caveman after clear part done.";

const BOUNDARIES: &str = "\
Boundaries: code blocks, commit messages, and JSON output must be \
written in normal form — compression applies to prose/explanations only.";

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_off_returns_none() {
        assert!(caveman_system_prompt(CavemanMode::Off).is_none());
    }

    #[test]
    fn test_lite_returns_prompt() {
        let prompt = caveman_system_prompt(CavemanMode::Lite).unwrap();
        assert!(prompt.contains("OUTPUT COMPRESSION"));
        assert!(prompt.contains("Intensity: lite"));
        assert!(prompt.contains("Keep articles and full sentences"));
    }

    #[test]
    fn test_full_returns_prompt() {
        let prompt = caveman_system_prompt(CavemanMode::Full).unwrap();
        assert!(prompt.contains("Intensity: full"));
        assert!(prompt.contains("Drop articles, fragments OK"));
    }

    #[test]
    fn test_ultra_returns_prompt() {
        let prompt = caveman_system_prompt(CavemanMode::Ultra).unwrap();
        assert!(prompt.contains("Intensity: ultra"));
        assert!(prompt.contains("Abbreviate"));
        assert!(prompt.contains("arrows for causality"));
    }

    #[test]
    fn test_auto_clarity_present() {
        for mode in [CavemanMode::Lite, CavemanMode::Full, CavemanMode::Ultra] {
            let prompt = caveman_system_prompt(mode).unwrap();
            assert!(
                prompt.contains("Auto-Clarity"),
                "Auto-Clarity missing for {mode}"
            );
        }
    }

    #[test]
    fn test_boundaries_present() {
        for mode in [CavemanMode::Lite, CavemanMode::Full, CavemanMode::Ultra] {
            let prompt = caveman_system_prompt(mode).unwrap();
            assert!(
                prompt.contains("Boundaries"),
                "Boundaries missing for {mode}"
            );
        }
    }

    #[test]
    fn test_is_active() {
        assert!(!CavemanMode::Off.is_active());
        assert!(CavemanMode::Lite.is_active());
        assert!(CavemanMode::Full.is_active());
        assert!(CavemanMode::Ultra.is_active());
    }

    #[test]
    fn test_display() {
        assert_eq!(CavemanMode::Off.to_string(), "off");
        assert_eq!(CavemanMode::Lite.to_string(), "lite");
        assert_eq!(CavemanMode::Full.to_string(), "full");
        assert_eq!(CavemanMode::Ultra.to_string(), "ultra");
    }

    #[test]
    fn test_default_is_off() {
        assert_eq!(CavemanMode::default(), CavemanMode::Off);
    }

    #[test]
    fn test_serde_roundtrip() {
        for mode in [
            CavemanMode::Off,
            CavemanMode::Lite,
            CavemanMode::Full,
            CavemanMode::Ultra,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let back: CavemanMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, back, "roundtrip failed for {mode}");
        }
    }
}
