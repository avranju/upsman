use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use rups::{blocking::Connection, Auth, ConfigBuilder};

#[derive(Debug, Parser)]
#[clap(version)]
struct Opt {
    /// NUT UPS server host name
    #[clap(long, short)]
    server: String,

    /// NUT UPS server TCP port
    #[clap(long, short)]
    port: u16,

    /// Name of the UPS
    #[clap(long, short)]
    ups_name: String,

    /// NUT server user name that has the permission to run INSTCMD
    #[clap(long, short = 'n')]
    username: Option<String>,

    /// NUT server password
    #[clap(long, short = 'w')]
    password: Option<String>,

    /// Enable debug output of network traffic
    #[clap(long, short, action)]
    debug: bool,

    /// Command to run,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, PartialEq, PartialOrd, Eq, Ord)]
enum Command {
    /// Turn load off on UPS
    LoadOff,

    /// Turn load on on UPS
    LoadOn,

    /// Fetch usage data
    Usage {
        /// Allowed values: voltage_in, voltage_out, current_out, power
        usage_types: Vec<UsageType>,
    },
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
enum UsageType {
    VoltageIn,
    VoltageOut,
    CurrentOut,
    Power,
}

impl FromStr for UsageType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vin" | "volt_in" | "voltage_in" => Ok(UsageType::VoltageIn),
            "vout" | "volt_out" | "voltage_out" => Ok(UsageType::VoltageOut),
            "cout" | "cur_out" | "current_out" => Ok(UsageType::CurrentOut),
            "pwr" | "power" => Ok(UsageType::Power),
            _ => Err(anyhow!("Invalid usage type value.")),
        }
    }
}

impl From<&UsageType> for &'static str {
    fn from(ut: &UsageType) -> Self {
        match ut {
            UsageType::VoltageIn => "input.voltage",
            UsageType::VoltageOut => "output.voltage",
            UsageType::CurrentOut => "output.current",
            UsageType::Power => "",
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let auth = opt
        .username
        .as_ref()
        .map(|username| Auth::new(username.clone(), opt.password.as_ref().map(Clone::clone)));
    let config = ConfigBuilder::new()
        .with_host((opt.server.clone(), opt.port).try_into()?)
        .with_auth(auth)
        .with_debug(opt.debug)
        .build();
    let mut connection = Connection::new(&config)?;

    match opt.command {
        Command::LoadOn => load_on(&mut connection, &opt)?,
        Command::LoadOff => load_off(&mut connection, &opt)?,
        Command::Usage { ref usage_types } => usage(&mut connection, &opt, usage_types)?,
    }

    Ok(())
}

fn load_on(connection: &mut Connection, opt: &Opt) -> Result<()> {
    Ok(connection.run_command(&opt.ups_name, Some("load.on"))?)
}

fn load_off(connection: &mut Connection, opt: &Opt) -> Result<()> {
    Ok(connection.run_command(&opt.ups_name, Some("load.off"))?)
}

fn usage(connection: &mut Connection, opt: &Opt, usage_types: &Vec<UsageType>) -> Result<()> {
    for ut in usage_types {
        if *ut == UsageType::Power {
            let voltage_out = parse_var::<f64>(connection, opt, &UsageType::VoltageOut)?;
            let current_out = parse_var::<f64>(connection, opt, &UsageType::CurrentOut)?;
            let power = voltage_out * current_out;
            println!("power: {power:.2} W");
        } else {
            let var_name = ut.into();
            let var_value = connection.get_var(&opt.ups_name, var_name)?;
            println!("{var_value}");
        }
    }

    Ok(())
}

fn parse_var<T: FromStr>(
    connection: &mut Connection,
    opt: &Opt,
    usage_type: &UsageType,
) -> Result<T> {
    let usage_type = usage_type.into();
    Ok(connection
        .get_var(&opt.ups_name, usage_type)?
        .to_string()
        .splitn(2, ": ")
        .nth(1)
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| anyhow!("Variable {} not found", usage_type))?)
}
