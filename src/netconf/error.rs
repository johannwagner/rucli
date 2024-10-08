use super::xml::RPCError;
use crate::netconf::RPCReplyCommand;

#[derive(Debug, thiserror::Error)]
pub enum NETCONFError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    XmlError(#[from] quick_xml::Error),
    #[error("{0}")]
    XmlDeError(#[from] quick_xml::DeError),
    #[error("Missing OK")]
    MissingOk,
    #[error("Unexpected command: {0}")]
    UnexpectedCommand(RPCReplyCommand),
    #[error("{0}")]
    RpcError(#[from] RPCError),
}

pub type NETCONFResult<T> = Result<T, NETCONFError>;
