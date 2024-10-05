use super::xml::RPCError;
use crate::netconf::RPCReplyCommand;

#[derive(Debug)]
pub enum NETCONFError {
    IoError(std::io::Error),
    XmlError(quick_xml::Error),
    XmlDeError(quick_xml::DeError),
    MissingOk,
    UnexpectedCommand(RPCReplyCommand),
    RpcError(RPCError),
}

pub type NETCONFResult<T> = Result<T, NETCONFError>;

impl From<std::io::Error> for NETCONFError {
    fn from(err: std::io::Error) -> Self {
        NETCONFError::IoError(err)
    }
}

impl From<quick_xml::Error> for NETCONFError {
    fn from(err: quick_xml::Error) -> Self {
        NETCONFError::XmlError(err)
    }
}

impl From<quick_xml::DeError> for NETCONFError {
    fn from(err: quick_xml::DeError) -> Self {
        NETCONFError::XmlDeError(err)
    }
}

impl From<RPCError> for NETCONFError {
    fn from(err: RPCError) -> Self {
        NETCONFError::RpcError(err)
    }
}
