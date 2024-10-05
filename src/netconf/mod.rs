use memmem::{Searcher, TwoWaySearcher};
use std::io::{self, Read, Write};

use quick_xml::{de::from_str, se::to_string};

mod error;
pub mod xml;

use crate::netconf::xml::{RPCError, RPCErrorInfo, RPC};

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
        };
        let hello_xml = to_string(&hello)?;
        let payload_mod = format!("{}\n]]>]]>\n", hello_xml);
        let wb = self.write(payload_mod.as_bytes())?;
        return Ok(wb);
    }

    fn read_hello(&mut self) -> NETCONFResult<Hello> {
        let str = self.read()?;
        let hello = from_str(&str)?;
        return Ok(hello);
    }

    fn send_rpc(&mut self, rpc: RPC) -> NETCONFResult<()> {
        let rpc_xml = to_string(&rpc)?;
        let payload = format!("{}\n]]>]]>\n", rpc_xml).replace("&quot;", "\"");
        // println!("{}", payload);
        let wb = self.write(payload.as_bytes())?;
        return Ok(wb);
    }

    fn read_result(&mut self) -> NETCONFResult<RPCReply> {
        let str = self.read()?;
        // println!("{}", str);
        let conf_info: RPCReply = from_str(&str)?;

        // FIXME: Errors might not come first.
        match conf_info.rpc_reply.first() {
            Some(xml::RPCReplyCommand::RPCError(x)) => {
                let rpc: RPCError = RPCError {
                    error_info: RPCErrorInfo {
                        bad_element: x.error_info.bad_element.to_string(),
                    },
                    error_severity: x.error_severity.to_string(),
                    error_path: x.error_path.to_string(),
                    error_message: x.error_message.to_string(),
                };
                return Err(rpc.into());
            }
            _ => Ok(conf_info),
        }
    }

    pub fn send_command(&mut self, command: String, format: String) -> NETCONFResult<RPCReply> {
        let c = RPC {
            rpc: RPCCommand::Command { command, format },
        };
        let _ = self.send_rpc(c)?;
        return self.read_result();
    }

    pub fn lock_configuration(&mut self) -> NETCONFResult<RPCReply> {
        let c = RPC {
            rpc: RPCCommand::LockConfiguration {},
        };
        let _ = self.send_rpc(c)?;
        return self.read_result();
    }

    pub fn unlock_configuration(&mut self) -> NETCONFResult<RPCReply> {
        let c = RPC {
            rpc: RPCCommand::UnlockConfiguration {},
        };
        let _ = self.send_rpc(c)?;
        return self.read_result();
    }

    pub fn apply_configuration(&mut self) -> NETCONFResult<RPCReply> {
        let c = RPC {
            rpc: RPCCommand::CommitConfirmedConfiguration {
                confirm_timeout: 5,
                confirmed: ConfigurationConfirmed {},
            },
        };
        let _ = self.send_rpc(c)?;
        return self.read_result();
    }

    pub fn confirm_configuration(&mut self) -> NETCONFResult<RPCReply> {
        let c = RPC {
            rpc: RPCCommand::CommitConfiguration {},
        };
        let _ = self.send_rpc(c)?;
        return self.read_result();
    }

    pub fn load_configuration(&mut self, cfg: String) -> NETCONFResult<RPCReply> {
        let c = RPC {
            rpc: RPCCommand::LoadConfiguration {
                format: "text".to_string(),
                action: "override".to_string(),
                cfg,
            },
        };
        let _ = self.send_rpc(c)?;

        return self.read_result();
    }

    pub fn diff_configuration(&mut self, format: String) -> NETCONFResult<RPCReply> {
        let c = RPC {
            rpc: RPCCommand::GetConfiguration {
                format: format,
                rollback: Some("0".to_string()),
                compare: Some("rollback".to_string()),
            },
        };
        let _ = self.send_rpc(c)?;
        return self.read_result();
    }
}
