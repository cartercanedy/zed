use schemars::{gen::SchemaSettings, JsonSchema};
use serde::{Deserialize, Serialize};
use zed_actions::RevealTarget;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use util::serde::default_true;

use crate::{HideStrategy, RevealStrategy, Shell, TaskTemplate, TaskTemplates, TaskType};

impl Default for DebugConnectionType {
    fn default() -> Self {
        DebugConnectionType::TCP(TCPHost::default())
    }
}

/// Represents the host information of the debug adapter
#[derive(Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Debug)]
pub struct TCPHost {
    /// The port that the debug adapter is listening on
    ///
    /// Default: We will try to find an open port
    pub port: Option<u16>,
    /// The host that the debug adapter is listening too
    ///
    /// Default: 127.0.0.1
    pub host: Option<Ipv4Addr>,
    /// The max amount of time in milliseconds to connect to a tcp DAP before returning an error
    ///
    /// Default: 2000ms
    pub timeout: Option<u64>,
}

impl TCPHost {
    /// Get the host or fallback to the default host
    pub fn host(&self) -> Ipv4Addr {
        self.host.unwrap_or_else(|| Ipv4Addr::new(127, 0, 0, 1))
    }
}

/// Represents the attach request information of the debug adapter
#[derive(Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Copy, Debug)]
pub struct AttachConfig {
    /// The processId to attach to, if left empty we will show a process picker
    #[serde(default)]
    pub process_id: Option<u32>,
}

/// Represents the type that will determine which request to call on the debug adapter
#[derive(Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DebugRequestType {
    /// Call the `launch` request on the debug adapter
    #[default]
    Launch,
    /// Call the `attach` request on the debug adapter
    Attach(AttachConfig),
}

impl DebugRequestType {
    /// return the `AttachConfig` of a `DebugRequestType`, if it exists
    pub fn attach_config(&self) -> Option<AttachConfig> {
        match self {
            Self::Attach(config) => Some(*config),
            _ => None
        }
    }
}

/// The Debug adapter to use
#[derive(Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Debug)]
#[serde(rename_all = "lowercase", tag = "adapter")]
pub enum DebugAdapterKind {
    /// Manually setup starting a debug adapter
    /// The argument within is used to start the DAP
    Custom(CustomArgs),
    /// Use debugpy
    Python(TCPHost),
    /// Use vscode-php-debug
    Php(TCPHost),
    /// Use vscode-js-debug
    Javascript(TCPHost),
    /// Use delve
    Go(TCPHost),
    /// Use lldb
    Lldb,
    /// Use GDB's built-in DAP support
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    Gdb,
    /// Used for integration tests
    #[cfg(any(test, feature = "test-support"))]
    Fake,
}

impl DebugAdapterKind {
    /// Returns the display name for the adapter kind
    pub fn display_name(&self) -> &str {
        match self {
            Self::Custom(_) => "Custom",
            Self::Python(_) => "Python",
            Self::Php(_) => "PHP",
            Self::Javascript(_) => "JavaScript",
            Self::Lldb => "LLDB",
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Gdb => "GDB",
            Self::Go(_) => "Go",
            #[cfg(any(test, feature = "test-support"))]
            Self::Fake => "Fake",
        }
    }
}

/// Custom arguments used to setup a custom debugger
#[derive(Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Debug)]
pub struct CustomArgs {
    /// The connection that a custom debugger should use
    #[serde(flatten)]
    pub connection: DebugConnectionType,
    /// The cli command used to start the debug adapter e.g. `python3`, `node` or the adapter binary
    pub command: String,
    /// The cli arguments used to start the debug adapter
    pub args: Option<Vec<String>>,
    /// The cli envs used to start the debug adapter
    pub envs: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct DebugShellTask {
    /// The path to an executable to run
    pub command: String,
    /// Additional command-line args to pass to command upon invocation
    pub args: Option<Vec<String>>,
    /// Optional environment variables to set for the command invocation
    pub envs: Option<HashMap<String, String>>
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DebugAuxiliaryTask {
    /// Human readable name of the task to display in the UI.
    pub label: String,
    /// Executable command to spawn.
    pub command: String,
    /// Arguments to the command.
    #[serde(default)]
    pub args: Vec<String>,
    /// Env overrides for the command, will be appended to the terminal's environment from the settings.
    #[serde(default)]
    pub env: ::collections::HashMap<String, String>,
    /// Current working directory to spawn the command into, defaults to current project root.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Whether to use a new terminal tab or reuse the existing one to spawn the process.
    #[serde(default)]
    pub use_new_terminal: bool,
    /// Whether to allow multiple instances of the same task to be run, or rather wait for the existing ones to finish.
    #[serde(default)]
    pub allow_concurrent_runs: bool,
    /// What to do with the terminal pane and tab, after the command was started:
    /// * `always` — always show the task's pane, and focus the corresponding tab in it (default)
    // * `no_focus` — always show the task's pane, add the task's tab in it, but don't focus it
    // * `never` — do not alter focus, but still add/reuse the task's tab in its pane
    #[serde(default)]
    pub reveal: RevealStrategy,
    /// Where to place the task's terminal item after starting the task.
    /// * `dock` — in the terminal dock, "regular" terminal items' place (default).
    /// * `center` — in the central pane group, "main" editor area.
    #[serde(default)]
    pub reveal_target: RevealTarget,
    /// What to do with the terminal pane and tab, after the command had finished:
    /// * `never` — do nothing when the command finishes (default)
    /// * `always` — always hide the terminal tab, hide the pane also if it was the last tab in it
    /// * `on_success` — hide the terminal tab on task success only, otherwise behaves similar to `always`.
    #[serde(default)]
    pub hide: HideStrategy,
    /// Which shell to use when spawning the task.
    #[serde(default)]
    pub shell: Shell,
    /// Whether to show the task line in the task output.
    #[serde(default = "default_true")]
    pub show_summary: bool,
    /// Whether to show the command line in the task output.
    #[serde(default = "default_true")]
    pub show_command: bool,
}

impl Into<TaskTemplate> for DebugAuxiliaryTask {
    /// Translate from debug definition to a task template
    fn into(self) -> TaskTemplate {
        TaskTemplate {
            task_type: TaskType::Script,
            label: self.label,
            command: self.command,
            args: self.args,
            cwd: self.cwd,
            shell: self.shell,
            allow_concurrent_runs: self.allow_concurrent_runs,
            env: self.env,
            hide: self.hide,
            reveal: self.reveal,
            reveal_target: self.reveal_target,
            show_command: self.show_command,
            show_summary: self.show_summary,
            use_new_terminal: self.use_new_terminal,
            tags: Vec::new()
        }
    }
}

/// Represents the configuration for the debug adapter
#[derive(Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct DebugAdapterConfig {
    /// Name of the debug task
    pub label: String,
    /// The type of adapter you want to use
    #[serde(flatten)]
    pub kind: DebugAdapterKind,
    /// The type of request that should be called on the debug adapter
    #[serde(default)]
    pub request: DebugRequestType,
    /// The program that you trying to debug
    pub program: Option<String>,
    /// The current working directory of your project
    pub cwd: Option<PathBuf>,
    /// Additional initialization arguments to be sent on DAP initialization
    pub initialize_args: Option<serde_json::Value>,
    /// Optional command to invoke prior to beginning debug session, e.g. `cargo build`
    pub pre_debug_task: Option<DebugAuxiliaryTask>,
    /// Optional command to invoke after completion of a debug session
    pub post_debug_task: Option<DebugAuxiliaryTask>
}

/// Represents the type of the debugger adapter connection
#[derive(Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Debug)]
#[serde(rename_all = "lowercase", tag = "connection")]
pub enum DebugConnectionType {
    /// Connect to the debug adapter via TCP
    TCP(TCPHost),
    /// Connect to the debug adapter via STDIO
    STDIO,
}

/// This struct represent a user created debug task
#[derive(Deserialize, Serialize, PartialEq, Eq, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct DebugTaskDefinition {
    /// The adapter to run
    #[serde(flatten)]
    kind: DebugAdapterKind,
    /// The type of request that should be called on the debug adapter
    #[serde(default)]
    request: DebugRequestType,
    /// Name of the debug task
    label: String,
    /// Program to run the debugger on
    program: Option<String>,
    /// The current working directory of your project
    cwd: Option<String>,
    /// Additional initialization arguments to be sent on DAP initialization
    initialize_args: Option<serde_json::Value>,
    /// Optional command to invoke prior to beginning debug session, e.g. `cargo build`
    pre_debug_task: Option<DebugAuxiliaryTask>,
    /// Optional command to invoke after completion of a debug session
    post_debug_task: Option<DebugAuxiliaryTask>
}

impl Into<DebugAdapterConfig> for DebugTaskDefinition {
    fn into(self) -> DebugAdapterConfig {
        DebugAdapterConfig {
            label: self.label,
            kind: self.kind,
            request: self.request,
            program: self.program,
            cwd: self.cwd.map(PathBuf::from).take_if(|p| p.exists()),
            initialize_args: self.initialize_args,
            pre_debug_task: self.pre_debug_task,
            post_debug_task: self.post_debug_task
        }
    }
}

impl DebugTaskDefinition {
    /// Translate from debug definition to a task template
    pub fn to_task_template(self) -> TaskTemplate {
        TaskTemplate {
            task_type: TaskType::Debug(self.clone().into()),
            label: self.label,
            command: String::new(),
            args: Vec::new(),
            cwd: self.cwd,
            ..Default::default()
        }
    }
}

/// A group of Debug Tasks defined in a JSON file.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct DebugTaskFile(pub Vec<DebugTaskDefinition>);

impl DebugTaskFile {
    /// Generates JSON schema of Tasks JSON template format.
    pub fn generate_json_schema() -> serde_json_lenient::Value {
        let schema = SchemaSettings::draft07()
            .with(|settings| settings.option_add_null_type = false)
            .into_generator()
            .into_root_schema_for::<Self>();

        serde_json_lenient::to_value(schema).unwrap()
    }
}

impl TryFrom<DebugTaskFile> for TaskTemplates {
    type Error = anyhow::Error;

    fn try_from(value: DebugTaskFile) -> Result<Self, Self::Error> {
        let templates = value
            .0
            .into_iter()
            .map(|debug_definition| debug_definition.to_task_template())
            .collect();

        Ok(Self(templates))
    }
}
