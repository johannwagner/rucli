use memmem::{Searcher, TwoWaySearcher};
use std::io::{self, Read, Write};

use quick_xml::{de::from_str, se::to_string};

mod error;
pub mod xml;

use crate::netconf::error::NETCONFError;
use crate::netconf::xml::LoadConfigurationResultsEnum;
use crate::netconf::xml::RPCReplyCommand;
use crate::netconf::xml::RPC;

use self::{
    error::NETCONFResult,
    xml::{ConfigurationConfirmed, Hello, RPCCommand, RPCReply},
};

pub struct NETCONFClient {
    // FIXME: Technically, this could be generic.
    channel: ssh2::Channel,
}

impl NETCONFClient {
    pub fn new(channel: ssh2::Channel) -> NETCONFClient {
        return NETCONFClient { channel };
    }

    pub fn init(&mut self) -> NETCONFResult<()> {
        self.send_hello()?;
        self.read_hello()?;

        return Ok(());
    }

    pub fn read(&mut self) -> io::Result<String> {
        let mut read_buffer: Vec<u8> = vec![];

        let mut buffer = [0u8; 128];
        let search = TwoWaySearcher::new("]]>]]>".as_bytes());
        while search.search_in(&read_buffer).is_none() {
            let bytes = self.channel.read(&mut buffer)?;
            read_buffer.extend(&buffer[..bytes]);
        }
        let pos = search.search_in(&read_buffer).unwrap();
        let resp = String::from_utf8(read_buffer[..pos].to_vec()).unwrap();
        // 6: ]]>]]>
        read_buffer.drain(0..(pos + 6));
        Ok(resp)
    }

    fn write(&mut self, payload: &[u8]) -> io::Result<()> {
        self.channel.write_all(payload)
    }

    fn send_hello(&mut self) -> NETCONFResult<()> {
        let hello = xml::Hello {
            capabilities: xml::Capabilities {
                capability: vec!["urn:ietf:params:netconf:base:1.0".to_owned()],
            },
            namespace: None,
            session_id: None,
        };
        let hello_xml = to_string(&hello)?;
        let payload_mod = format!("{}\n]]>]]>\n", hello_xml);
        //println!("{}", payload_mod);
        let wb = self.write(payload_mod.as_bytes())?;
        return Ok(wb);
    }

    fn read_hello(&mut self) -> NETCONFResult<Hello> {
        let str = self.read()?;
        //eprintln!("{}", str);
        let hello = from_str(&str)?;
        return Ok(hello);
    }

    fn send_rpc(&mut self, rpc: RPC) -> NETCONFResult<()> {
        let rpc_xml = to_string(&rpc)?;
        let payload = format!("{}\n]]>]]>\n", rpc_xml).replace("&quot;", "\"");
        //println!("{}", payload);
        let wb = self.write(payload.as_bytes())?;
        return Ok(wb);
    }

    fn read_result(&mut self) -> NETCONFResult<impl Iterator<Item = RPCReplyCommand>> {
        let str = self.read()?;
        //eprintln!("{}", str);
        Ok(from_str::<RPCReply>(&str)?.rpc_reply.into_iter())
    }

    pub fn send_command(&mut self, command: String, format: String) -> NETCONFResult<String> {
        let c = RPC {
            rpc: RPCCommand::Command {
                command,
                format: format.clone(),
            },
        };
        let _ = self.send_rpc(c)?;
        let mut output = None;
        for result in self.read_result()? {
            match result {
                RPCReplyCommand::RPCError(error) => {
                    if error.error_severity == "warning" {
                        let mut msg = "Warning: ".to_string();
                        if let Some(error_path) = error.error_path {
                            msg.push_str(&error_path);
                            msg.push_str(&" ");
                        }
                        msg.push_str(&error.error_message);
                        eprintln!("{}", msg);
                    } else {
                        return Err(error.into());
                    }
                }
                RPCReplyCommand::Other(text) if output.is_none() && format == "json" => {
                    output = Some(text)
                }
                RPCReplyCommand::Output { text } if output.is_none() && format == "text" => {
                    output = Some(text)
                }
                other => return Err(NETCONFError::UnexpectedCommand(other)),
            }
        }
        output.ok_or(NETCONFError::MissingOk)
    }

    pub fn lock_configuration(&mut self) -> NETCONFResult<()> {
        let c = RPC {
            rpc: RPCCommand::LockConfiguration {},
        };
        let _ = self.send_rpc(c)?;
        for result in self.read_result()? {
            match result {
                other => return Err(NETCONFError::UnexpectedCommand(other)),
            }
        }
        Ok(())
    }

    pub fn unlock_configuration(&mut self) -> NETCONFResult<()> {
        let c = RPC {
            rpc: RPCCommand::UnlockConfiguration {},
        };
        let _ = self.send_rpc(c)?;
        for result in self.read_result()? {
            match result {
                RPCReplyCommand::Ok => {} // sometimes sent, sometimes not
                other => return Err(NETCONFError::UnexpectedCommand(other)),
            }
        }
        Ok(())
    }

    pub fn apply_configuration(&mut self, confirm_timeout: Option<i32>) -> NETCONFResult<()> {
        if let Some(confirm_timeout) = confirm_timeout {
            let c = RPC {
                rpc: RPCCommand::CommitConfirmedConfiguration {
                    confirm_timeout,
                    confirmed: ConfigurationConfirmed {},
                },
            };
            let _ = self.send_rpc(c)?;
        } else {
            let c = RPC {
                rpc: RPCCommand::CommitConfiguration {},
            };
            let _ = self.send_rpc(c)?;
        }
        let mut ok = None;
        for result in self.read_result()? {
            match result {
                RPCReplyCommand::RPCError(error) => {
                    if error.error_severity == "warning" {
                        let mut msg = "Warning: ".to_string();
                        if let Some(error_path) = error.error_path {
                            msg.push_str(&error_path);
                            msg.push_str(&" ");
                        }
                        msg.push_str(&error.error_message);
                        eprintln!("{}", msg);
                    } else {
                        return Err(error.into());
                    }
                }
                RPCReplyCommand::Other(_) => {} // ???
                RPCReplyCommand::Ok => ok = Some(()),
                other => return Err(NETCONFError::UnexpectedCommand(other)),
            }
        }
        ok.ok_or(NETCONFError::MissingOk)
    }

    pub fn confirm_configuration(&mut self) -> NETCONFResult<()> {
        let c = RPC {
            rpc: RPCCommand::CommitConfiguration {},
        };
        let _ = self.send_rpc(c)?;
        let mut ok = None;
        for result in self.read_result()? {
            match result {
                RPCReplyCommand::RPCError(error) => {
                    if error.error_severity == "warning" {
                        let mut msg = "Warning: ".to_string();
                        if let Some(error_path) = error.error_path {
                            msg.push_str(&error_path);
                            msg.push_str(&" ");
                        }
                        msg.push_str(&error.error_message);
                        eprintln!("{}", msg);
                    } else {
                        return Err(error.into());
                    }
                }
                RPCReplyCommand::Ok => ok = Some(()),
                other => return Err(NETCONFError::UnexpectedCommand(other)),
            }
        }
        ok.ok_or(NETCONFError::MissingOk)
    }

    pub fn load_configuration(&mut self, cfg: String) -> NETCONFResult<()> {
        let c = RPC {
            rpc: RPCCommand::LoadConfiguration {
                format: "text".to_string(),
                action: "update".to_string(),
                cfg,
            },
        };
        let _ = self.send_rpc(c)?;

        let mut load_config_result = None;
        for result in self.read_result()? {
            match result {
                RPCReplyCommand::LoadConfigurationResults(results) => {
                    load_config_result = Some(results);
                }
                other => return Err(NETCONFError::UnexpectedCommand(other)),
            }
        }
        let mut ok = None;
        for result in load_config_result
            .ok_or(NETCONFError::MissingOk)?
            .load_configuration_results
        {
            match result {
                LoadConfigurationResultsEnum::RPCError(error) => {
                    if error.error_severity == "warning" {
                        let mut msg = "Warning: ".to_string();
                        if let Some(error_path) = error.error_path {
                            msg.push_str(&error_path);
                            msg.push_str(&" ");
                        }
                        msg.push_str(&error.error_message);
                        eprintln!("{}", msg);
                    } else {
                        return Err(error.into());
                    }
                }
                LoadConfigurationResultsEnum::Ok => ok = Some(()),
            }
        }
        ok.ok_or(NETCONFError::MissingOk)
    }

    pub fn diff_configuration(&mut self, format: String) -> NETCONFResult<String> {
        let c = RPC {
            rpc: RPCCommand::GetConfiguration {
                format: format,
                rollback: Some("0".to_string()),
                compare: Some("rollback".to_string()),
            },
        };
        let _ = self.send_rpc(c)?;
        let mut diff_result = None;
        for result in self.read_result()? {
            match result {
                RPCReplyCommand::ConfigurationInformation {
                    configuration_output,
                } => {
                    diff_result = Some(configuration_output);
                }
                other => return Err(NETCONFError::UnexpectedCommand(other)),
            }
        }
        diff_result.ok_or(NETCONFError::MissingOk)
    }
}
