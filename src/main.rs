use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::{env, fs};

use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use ssh2_config::{ParseRule, SshConfig};

use rucli::netconf::NETCONFClient;
use rucli::ssh::SSHConnection;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    hostname: String,

    #[arg(long, short)]
    user: Option<String>,

    #[arg(long, short, action=ArgAction::SetTrue)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Format {
    Text,
    JSON,
    // TODO: We need to get into quick-xml to find out how to handle internal XML as strings.
    // XML
}

#[derive(Subcommand)]
enum Commands {
    /// Executes an given command on the router
    Exec {
        #[clap(value_enum)]
        format: Format,

        command: Vec<String>,
    },

    /// Applies local configuration file on router
    Apply {
        local_file: String,
        confirm_timeout: Option<i32>,
    },

    /// Confirm a previously applied configuration
    Confirm,

    /// Loads local configuration onto router and shows a diff
    Check { local_file: String },
}

fn main() {
    let cli = Cli::parse();

    let ssh_user = match cli.user {
        Some(user) => user,
        None => (|| -> Option<String> {
            let mut reader = BufReader::new(
                File::open(Path::new(
                    (env::var("HOME").unwrap().to_owned() + "/.ssh/config").as_str(),
                ))
                .ok()?,
            );
            let config = SshConfig::default()
                .parse(&mut reader, ParseRule::STRICT)
                .ok()?;
            let params = config.query(&cli.hostname);

            params.user
        })()
        .unwrap_or(env::var("USER").unwrap()),
    };

    let mut ssh_connection = SSHConnection::new(
        ssh_user.as_str(),
        format!("{}:830", cli.hostname).as_str(),
        cli.debug,
    );
    ssh_connection.connect().unwrap();

    let mut netconf_session = NETCONFClient::new(ssh_connection.channel.expect(""));
    netconf_session.init().unwrap();

    match cli.command {
        Commands::Exec { format, command } => {
            let format_str = match format {
                Format::Text => "text",
                Format::JSON => "json",
            };

            let command_str = command.join(" ").to_owned();

            let r = netconf_session
                .send_command(command_str, format_str.to_owned())
                .unwrap();

            eprintln!("{}", r)
        }
        Commands::Apply {
            local_file,
            confirm_timeout,
        } => {
            let data = fs::read_to_string(local_file).unwrap();

            let _ = netconf_session.lock_configuration().unwrap();

            let load_config_reply = netconf_session.load_configuration(data).unwrap();
            eprintln!("{}", load_config_reply);

            let diff_reply = netconf_session
                .diff_configuration("text".to_string())
                .unwrap();
            println!("{}", diff_reply);

            eprintln!("Applying configuration...");

            let apply_reply = netconf_session
                .apply_configuration(confirm_timeout)
                .unwrap();
            eprintln!("{}", apply_reply);

            let _ = netconf_session.unlock_configuration().unwrap();
        }
        Commands::Confirm => {
            eprintln!("Confirming configuration");

            let confirm_reply = netconf_session.confirm_configuration().unwrap();
            eprintln!("{}", confirm_reply);
        }
        Commands::Check { local_file } => {
            let data = fs::read_to_string(local_file).unwrap();

            let _ = netconf_session.lock_configuration().unwrap();

            let load_config_reply = netconf_session.load_configuration(data).unwrap();
            eprintln!("{}", load_config_reply);

            let diff_reply = netconf_session
                .diff_configuration("text".to_string())
                .unwrap();
            println!("{}", diff_reply);

            let _ = netconf_session.unlock_configuration().unwrap();
        }
    }
}
