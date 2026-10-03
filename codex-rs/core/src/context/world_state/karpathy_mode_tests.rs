use super::*;
use crate::context::world_state::WorldState;
use pretty_assertions::assert_eq;

#[test]
fn mode_changes_add_and_retire_instructions_without_duplicates() {
    let mut history = Vec::new();
    let mut previous = None;

    for (enabled, expected) in [
        (false, None),
        (true, Some(INSTRUCTIONS)),
        (true, None),
        (false, Some(REMOVAL_NOTICE)),
        (false, None),
        (true, Some(INSTRUCTIONS)),
    ] {
        let mut world_state = WorldState::default();
        world_state.add_section(KarpathyModeState::new(enabled));
        let (snapshot, fragments) =
            world_state.render_history_fragment_diff(previous.as_ref(), &history);
        let updates = fragments
            .into_iter()
            .map(ContextualUserFragment::into_boxed_response_item)
            .collect::<Vec<_>>();
        assert_eq!(
            updates,
            expected
                .map(|instructions| {
                    ContextualUserFragment::into(KarpathyModeState {
                        instructions: instructions.to_string(),
                    })
                })
                .into_iter()
                .collect::<Vec<_>>()
        );
        history.extend(updates);
        previous = Some(snapshot);
    }
}

#[test]
fn retained_instructions_are_replaced_or_retired_without_a_snapshot() {
    let retained = ContextualUserFragment::into(KarpathyModeState {
        instructions: "previous instructions".to_string(),
    });
    for (enabled, expected) in [
        (true, format!("{REPLACEMENT_NOTICE}\n\n{INSTRUCTIONS}")),
        (false, REMOVAL_NOTICE.to_string()),
    ] {
        let mut world_state = WorldState::default();
        world_state.add_section(KarpathyModeState::new(enabled));
        assert_eq!(
            world_state
                .render_history_fragment_diff(
                    /*previous*/ None,
                    std::slice::from_ref(&retained)
                )
                .1
                .into_iter()
                .map(ContextualUserFragment::into_boxed_response_item)
                .collect::<Vec<_>>(),
            vec![ContextualUserFragment::into(KarpathyModeState {
                instructions: expected,
            })]
        );
    }
}

#[test]
fn changed_instructions_replace_the_previous_version() {
    let previous_instructions = KarpathyModeState {
        instructions: "older explanation preferences".to_string(),
    };
    let mut previous = WorldState::default();
    previous.add_section(previous_instructions.clone());
    let (snapshot, _) = previous.render_full();
    let history = [ContextualUserFragment::into(previous_instructions)];
    let mut current = WorldState::default();
    current.add_section(KarpathyModeState::new(/*enabled*/ true));

    assert_eq!(
        current
            .render_history_fragment_diff(Some(&snapshot), &history)
            .1
            .into_iter()
            .map(ContextualUserFragment::into_boxed_response_item)
            .collect::<Vec<_>>(),
        vec![ContextualUserFragment::into(KarpathyModeState {
            instructions: format!("{REPLACEMENT_NOTICE}\n\n{INSTRUCTIONS}"),
        })]
    );
}
