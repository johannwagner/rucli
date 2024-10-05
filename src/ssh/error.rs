#[derive(Debug)]
pub enum SSHError {
    SSHError(ssh2::Error),
    IoError(std::io::Error),
}

impl From<ssh2::Error> for SSHError {
    fn from(err: ssh2::Error) -> Self {
        SSHError::SSHError(err)
    }
}

impl From<std::io::Error> for SSHError {
    fn from(err: std::io::Error) -> Self {
        SSHError::IoError(err)
    }
}
