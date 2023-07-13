use std::fmt::Display;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "hello")]
pub struct Hello {
    pub capabilities: Capabilities,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Capabilities {
    pub capability: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "rpc")]
pub struct RPC {
    pub rpc: RPCCommand,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum RPCCommand {
    #[serde(rename = "get-configuration")]
    GetConfiguration {
        #[serde(rename = "@format")]
        format: String,

        #[serde(rename = "@rollback")]
        rollback: Option<String>,

        #[serde(rename = "@compare")]
        compare: Option<String>,
    },
    #[serde(rename = "command")]
    Command {
        #[serde(rename = "@format")]
        format: String,

        #[serde(rename = "$text")]
        command: String,
    },
    #[serde(rename = "lock-configuration")]
    LockConfiguration {},

    #[serde(rename = "unlock-configuration")]
    UnlockConfiguration {},

    #[serde(rename = "load-configuration")]
    LoadConfiguration {
        #[serde(rename = "@format")]
        format: String,

        #[serde(rename = "@action")]
        action: String,

        #[serde(rename = "configuration-text")]
        cfg: String,
    },

    #[serde(rename = "commit-configuration")]
    CommitConfiguration {},

    #[serde(rename = "commit-configuration")]
    CommitConfirmedConfiguration {
        confirmed: ConfigurationConfirmed,

        #[serde(rename = "confirm-timeout")]
        confirm_timeout: i32,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigurationConfirmed {}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigurationInformation {
    #[serde(rename = "configuration-text")]
    pub configuration_text: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "rpc-reply")]
pub struct RPCReply {
    #[serde(rename = "$value", default)]
    pub rpc_reply: Vec<RPCReplyCommand>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum RPCReplyCommand {
    #[serde(rename = "output")]
    Output {
        #[serde(rename = "$text")]
        text: String,
    },

    #[serde(rename = "load-configuration-results")]
    LoadConfigurationResults(LoadConfigurationResults),

    #[serde(rename = "configuration-information")]
    ConfigurationInformation {
        #[serde(rename = "configuration-output")]
        configuration_output: String,
    },

    #[serde(rename = "rpc-error")]
    RPCError(RPCError),

    #[serde(rename = "ok")]
    Ok,

    #[serde(rename = "$text")]
    Other(String),
}

#[derive(Debug, Deserialize, Serialize)]

pub struct LoadConfigurationResults {
    #[serde(rename = "$value")]
    load_configuration_results: Vec<LoadConfigurationResultsEnum>,
}

#[derive(Debug, Deserialize, Serialize)]

pub enum LoadConfigurationResultsEnum {
    #[serde(rename = "ok")]
    Ok,

    #[serde(rename = "rpc-error")]
    RPCError(RPCError),
}

impl Display for RPCReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for reply in &self.rpc_reply {
            _ = writeln!(f, "{}", reply)
        }

        Ok(())
    }
}

impl Display for RPCReplyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RPCReplyCommand::Output { text } => {
                write!(f, "{}", text)
            }
            RPCReplyCommand::Other(text) => {
                write!(f, "{}", text)
            }
            RPCReplyCommand::ConfigurationInformation {
                configuration_output: configuration_information,
            } => {
                write!(f, "{}", configuration_information)
            }
            RPCReplyCommand::Ok => {
                write!(f, "Executed Successfully!")
            }
            RPCReplyCommand::RPCError(err) => {
                write!(f, "{}", err)
            }
            RPCReplyCommand::LoadConfigurationResults(x) => {
                for elem in &x.load_configuration_results {
                    match elem {
                        LoadConfigurationResultsEnum::Ok => {
                            writeln!(f, "{}", "Success!")?;
                        }
                        LoadConfigurationResultsEnum::RPCError(error) => {
                            writeln!(f, "{}", error)?;
                        }
                    }
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]

pub struct RPCErrorList {
    element: Vec<RPCError>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RPCError {
    #[serde(rename = "error-severity")]
    pub error_severity: String,
    #[serde(rename = "error-path")]
    pub error_path: String,
    #[serde(rename = "error-message")]
    pub error_message: String,
    #[serde(rename = "error-info")]
    pub error_info: RPCErrorInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RPCErrorInfo {
    #[serde(rename = "bad-element")]
    pub bad_element: String,
}

impl Display for RPCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} at {} {}",
            self.error_severity, self.error_path, self.error_info.bad_element
        )?;
        writeln!(f, "{}", self.error_message)?;
        Ok(())
    }
}
