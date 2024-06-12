use std::net::TcpStream;

use ssh2::{Session, TraceFlags};

use self::error::SSHError;

pub mod error;

pub struct SSHConnection {
    pub user: String,
    pub password: Option<String>,
    pub target: String,
    pub debug: bool,

    pub sess: Option<ssh2::Session>,
    pub channel: Option<ssh2::Channel>,
}

impl SSHConnection {
    pub fn new(user: &str, password: Option<String>, target: &str, debug: bool) -> SSHConnection {
        return SSHConnection {
            user: String::from(user),
            password: password,
            target: String::from(target),
            debug,
            sess: None,
            channel: None,
        };
    }

    pub fn connect(&mut self) -> Result<(), SSHError> {
        let tcp = TcpStream::connect(self.target.as_str())?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        if self.debug {
            sess.trace(TraceFlags::AUTH | TraceFlags::KEX | TraceFlags::PUBLICKEY);
        };
        sess.handshake()?;
        if self.password.is_some() {
            sess.userauth_password(self.user.as_str(), self.password.clone().unwrap().as_str())?;
        } else {
            sess.userauth_agent(self.user.as_str())?;
        }

        let mut channel = sess.channel_session()?;
        channel.subsystem("netconf")?;

        self.sess = Some(sess);
        self.channel = Some(channel);

        return Ok(());
    }
}
