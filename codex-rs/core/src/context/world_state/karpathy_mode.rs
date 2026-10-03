//! Refreshes the opt-in explanation style, including after resume or compaction.

use super::PreviousSectionState;
use super::SectionTransition;
use super::WorldStateHash;
use super::WorldStateSection;
use super::WorldStateUpdate;
use crate::context::ContextualUserFragment;
use codex_protocol::models::ContentItemKind;
use serde::Deserialize;
use serde::Serialize;

const INSTRUCTIONS: &str = r#"Karpathy Mode is enabled.

Make your work easy for the user to understand. Follow these explanation preferences while completing the requested task:

- Write in plain English, roughly 80% of the way toward ASD-STE100 Simplified Technical English. Prefer familiar words, short sentences, active voice, and one idea at a time. Define necessary technical terms. Keep exact code, identifiers, and technical facts precise; do not claim formal ASD-STE100 compliance.
- Use diagrams or images when they clarify structure, relationships, or a process. Choose a format the user can view; a compact text or Mermaid diagram may be enough.
- For concepts that benefit from exploration, create a small interactive HTML explanation with useful controls and concrete examples. Prefer a self-contained local file and give the user a link to it.
- For complex topics that benefit from a guided sequence, consider a short visual explainer video with clear narration when suitable tools are available. Use configured narration tools or local alternatives when available. Never claim to have created or verified media that you did not create or inspect.
- Choose the smallest explanation that makes the result clear. Simple answers and small changes usually need only text. Do not generate extra artifacts for their own sake or let them delay the requested work. Respect the user's requested format, time constraints, and available tools; the mode does not grant permission to spend money or publish artifacts.

Connect explanations to the actual result: what changed, why it works, and what was checked. When you create an artifact, inspect it and share its location along with a concise explanation."#;

const REPLACEMENT_NOTICE: &str =
    "These Karpathy Mode instructions replace all previously provided Karpathy Mode instructions.";
const REMOVAL_NOTICE: &str = "Karpathy Mode is disabled. The previously provided Karpathy Mode instructions no longer apply.";

#[derive(Clone, Debug)]
pub(crate) struct KarpathyModeState {
    instructions: String,
}

impl KarpathyModeState {
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            instructions: if enabled { INSTRUCTIONS } else { "" }.to_string(),
        }
    }
}

impl ContextualUserFragment for KarpathyModeState {
    fn content_kind(&self) -> ContentItemKind {
        ContentItemKind("karpathy_mode.instructions".to_string())
    }

    fn role(&self) -> &'static str {
        "developer"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        ("<karpathy_mode>", "</karpathy_mode>")
    }

    fn body(&self) -> String {
        format!("\n{}\n", self.instructions)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct KarpathyModeSnapshot {
    // Preserve the disabled state: JSON null would delete the entire section.
    instructions: Option<WorldStateHash>,
}

impl WorldStateSection for KarpathyModeState {
    const ID: &'static str = "karpathy_mode";
    type Snapshot = KarpathyModeSnapshot;

    fn matches_legacy_fragment(role: &str, text: &str) -> bool {
        role == "developer" && Self::matches_text(text)
    }

    fn has_retained_fragment_matcher() -> bool {
        true
    }

    fn matches_retained_fragment(role: &str, text: &str) -> bool {
        Self::matches_legacy_fragment(role, text)
    }

    fn render_diff(
        &self,
        previous: PreviousSectionState<'_, Self::Snapshot>,
    ) -> SectionTransition<Self::Snapshot> {
        let current = KarpathyModeSnapshot {
            instructions: (!self.instructions.is_empty())
                .then(|| WorldStateHash::from_fragment(self)),
        };
        if matches!(previous, PreviousSectionState::Known(previous) if previous == &current) {
            return (None, Vec::new());
        }
        let previous_had_instructions = match previous {
            PreviousSectionState::Absent => false,
            PreviousSectionState::Unknown => true,
            PreviousSectionState::Known(previous) => previous.instructions.is_some(),
        };
        let instructions = match (self.instructions.as_str(), previous_had_instructions) {
            ("", false) => return (Some(current), Vec::new()),
            ("", true) => REMOVAL_NOTICE.to_string(),
            (instructions, true) => format!("{REPLACEMENT_NOTICE}\n\n{instructions}"),
            (instructions, false) => instructions.to_string(),
        };
        (
            Some(current),
            vec![WorldStateUpdate::fragment(Self { instructions })],
        )
    }
}

#[cfg(test)]
#[path = "karpathy_mode_tests.rs"]
mod tests;
