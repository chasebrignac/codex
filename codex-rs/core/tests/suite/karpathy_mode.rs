use anyhow::Result;
use codex_config::LoaderOverrides;
use codex_core::ConfigRefreshOutcome;
use codex_core::config::ConfigBuilder;
use codex_features::Feature;
use core_test_support::responses::ResponsesRequest;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse_completed;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;
use test_case::test_case;

const ENABLED_NOTICE: &str = "Karpathy Mode is enabled.";
const DISABLED_NOTICE: &str = "Karpathy Mode is disabled. The previously provided Karpathy Mode instructions no longer apply.";
const BASE_INSTRUCTIONS: &str = "Keep the configured base instructions.";
const DEVELOPER_INSTRUCTIONS: &str = "Keep the configured developer instructions.";

fn mode_messages(request: &ResponsesRequest) -> Vec<String> {
    request
        .message_input_texts("developer")
        .into_iter()
        .filter(|text| text.contains("<karpathy_mode>"))
        .collect()
}

async fn refresh_mode(test: &TestCodex, enabled: bool) -> Result<()> {
    let current = test.codex.config().await;
    std::fs::write(
        test.home.path().join("config.toml"),
        format!("[features]\nkarpathy_mode = {enabled}\n"),
    )?;
    let next = ConfigBuilder::default()
        .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
        .codex_home(test.home.path().to_path_buf())
        .fallback_cwd(Some(current.cwd.to_path_buf()))
        .build()
        .await?;
    assert!(matches!(
        test.codex.refresh_runtime_config(current, next).await,
        ConfigRefreshOutcome::Published
    ));
    Ok(())
}

#[test_case(None; "default disabled")]
#[test_case(Some(false); "explicitly disabled")]
#[test_case(Some(true); "enabled")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn karpathy_mode_config_reaches_model_without_replacing_other_instructions(
    enabled: Option<bool>,
) -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;
    let test = test_codex()
        .with_pre_build_hook(move |home| {
            if let Some(enabled) = enabled {
                std::fs::write(
                    home.join("config.toml"),
                    format!("[features]\nkarpathy_mode = {enabled}\n"),
                )
                .expect("write Karpathy Mode config");
            }
        })
        .with_config(|config| {
            config.base_instructions = Some(BASE_INSTRUCTIONS.to_string());
            config.developer_instructions = Some(DEVELOPER_INSTRUCTIONS.to_string());
        })
        .build_with_auto_env(&server)
        .await?;

    assert_eq!(
        test.config.features.enabled(Feature::KarpathyMode),
        enabled.unwrap_or(false)
    );
    test.submit_turn("explain the implementation").await?;
    test.submit_turn("continue the explanation").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    for request in &requests {
        assert_eq!(request.instructions_text(), BASE_INSTRUCTIONS);
        assert!(
            request
                .message_input_texts("developer")
                .iter()
                .any(|text| text.contains(DEVELOPER_INSTRUCTIONS))
        );
        let messages = mode_messages(request);
        assert_eq!(messages.len(), usize::from(enabled == Some(true)));
        if let Some(message) = messages.first() {
            assert!(message.contains(ENABLED_NOTICE));
            assert!(message.contains("ASD-STE100"));
            assert!(message.contains("HTML"));
            assert!(message.contains("</karpathy_mode>"));
        }
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn karpathy_mode_runtime_refresh_retires_instructions_without_repeating_updates() -> Result<()>
{
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        (1..=5)
            .map(|index| sse_completed(&format!("resp-{index}")))
            .collect(),
    )
    .await;
    let test = test_codex().build_with_auto_env(&server).await?;
    test.submit_turn("mode is initially off").await?;
    refresh_mode(&test, true).await?;
    test.submit_turn("enable visual explanations").await?;
    refresh_mode(&test, false).await?;
    test.submit_turn("return to normal explanations").await?;
    test.submit_turn("keep normal explanations").await?;
    refresh_mode(&test, true).await?;
    test.submit_turn("enable visual explanations again").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 5);
    let messages = requests.iter().map(mode_messages).collect::<Vec<_>>();
    assert!(messages[0].is_empty());
    assert_eq!(messages[1].len(), 1);
    assert!(messages[1][0].contains(ENABLED_NOTICE));
    assert_eq!(messages[2].len(), 2);
    assert!(messages[2][1].contains(DISABLED_NOTICE));
    assert!(!messages[2][1].contains(ENABLED_NOTICE));
    assert_eq!(messages[3], messages[2]);
    assert_eq!(messages[4].len(), 3);
    assert!(messages[4][2].contains(ENABLED_NOTICE));

    Ok(())
}

#[test_case(false, true; "enable on resume")]
#[test_case(true, false; "disable on resume")]
#[test_case(true, true; "unchanged on resume")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn karpathy_mode_cold_resume_uses_current_setting(
    initially_enabled: bool,
    resumed_enabled: bool,
) -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;
    let initial = test_codex()
        .with_pre_build_hook(move |home| {
            std::fs::write(
                home.join("config.toml"),
                format!("[features]\nkarpathy_mode = {initially_enabled}\n"),
            )
            .expect("write initial Karpathy Mode config");
        })
        .build_with_auto_env(&server)
        .await?;
    initial.submit_turn("before resume").await?;

    let resumed = test_codex()
        .with_pre_build_hook(move |home| {
            std::fs::write(
                home.join("config.toml"),
                format!("[features]\nkarpathy_mode = {resumed_enabled}\n"),
            )
            .expect("write resumed Karpathy Mode config");
        })
        .restart(&server, &initial)
        .await?;
    assert_eq!(
        resumed.session_configured.thread_id,
        initial.session_configured.thread_id
    );
    resumed.submit_turn("after resume").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    let initial_messages = mode_messages(&requests[0]);
    let resumed_messages = mode_messages(&requests[1]);
    assert_eq!(initial_messages.len(), usize::from(initially_enabled));
    assert_eq!(
        resumed_messages.len(),
        usize::from(initially_enabled) + usize::from(initially_enabled != resumed_enabled)
    );
    let last_message = resumed_messages.last().expect("mode instructions");
    assert!(last_message.contains(if resumed_enabled {
        ENABLED_NOTICE
    } else {
        DISABLED_NOTICE
    }));
    assert!(
        requests[1]
            .message_input_texts("user")
            .iter()
            .any(|text| text == "before resume")
    );

    Ok(())
}
