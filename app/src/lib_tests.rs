use super::*;

/// 构造一个 CLI `agent run` 的启动模式,用于验证 headless / GUI 判定。
fn command_line_launch_mode(gui: bool) -> LaunchMode {
    let args = warp_cli::agent::RunAgentArgs {
        prompt_arg: warp_cli::agent::PromptArg {
            prompt: Some("hello".to_owned()),
            saved_prompt: None,
        },
        model: Default::default(),
        config_file: Default::default(),
        skill: None,
        name: None,
        cwd: None,
        gui,
        share: warp_cli::share::ShareArgs { share: None },
        mcp_specs: Vec::new(),
        mcp_servers: Vec::new(),
        idle_on_complete: None,
        sandboxed: false,
        bedrock_inference_role: None,
        computer_use: Default::default(),
        profile: None,
        harness: Default::default(),
    };

    LaunchMode::CommandLine {
        command: CliCommand::Agent(AgentCommand::Run(Box::new(args))),
        global_options: GlobalOptions::default(),
        debug: false,
        is_sandboxed: false,
        computer_use_override: None,
    }
}

#[test]
fn headless_modes_do_not_start_local_http_server() {
    // 同机共存的 headless 进程(daemon、proxy、CLI)不能再抢占固定端口的本地 HTTP server。
    assert!(!LaunchMode::RemoteServerDaemon.should_start_local_http_server());
    assert!(!LaunchMode::RemoteServerProxy.should_start_local_http_server());
    assert!(!command_line_launch_mode(/* gui */ false).should_start_local_http_server());
}

#[test]
fn gui_modes_start_local_http_server() {
    assert!(LaunchMode::App {
        args: Default::default(),
        api_key: None,
    }
    .should_start_local_http_server());
    assert!(LaunchMode::new_for_unit_test().should_start_local_http_server());
    // `agent run --gui` 会拉起 GUI 窗口,因此仍然需要本地 HTTP server。
    assert!(command_line_launch_mode(/* gui */ true).should_start_local_http_server());
}
