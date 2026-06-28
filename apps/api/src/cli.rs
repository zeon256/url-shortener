use argh::FromArgs;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ProgramArgs {
    pub server: ServerArgs,
    pub postgres: PostgresArgs,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct ServerArgs {
    pub port: u16,
    pub address: &'static str,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct PostgresArgs {
    pub host: &'static str,
    pub port: u16,
    pub user: &'static str,
    pub password: &'static str,
    pub db: &'static str,
    pub cert: Option<&'static str>,
}

#[allow(dead_code)]
#[derive(FromArgs, Clone, Debug)]
/// url-shortener backend
struct RawProgramArgs {
    /// port number
    #[argh(option, default = "4002", env = "PORT")]
    port: u16,

    /// ip address
    #[argh(
        option,
        default = "default_address()",
        env = "ADDRESS",
        from_str_fn(parse_static_str)
    )]
    address: &'static str,

    /// postgres host
    #[argh(option, env = "POSTGRES_HOST", from_str_fn(parse_static_str))]
    postgres_host: &'static str,

    /// postgres port
    #[argh(option, default = "5432", env = "POSTGRES_PORT")]
    postgres_port: u16,

    /// postgres user
    #[argh(option, env = "POSTGRES_USER", from_str_fn(parse_static_str))]
    postgres_user: &'static str,

    /// postgres password
    #[argh(option, env = "POSTGRES_PASSWORD", from_str_fn(parse_static_str))]
    postgres_password: &'static str,

    /// postgres database
    #[argh(option, env = "POSTGRES_DB", from_str_fn(parse_static_str))]
    postgres_db: &'static str,

    /// postgres ssl certificate
    #[argh(option, env = "POSTGRES_CERT", from_str_fn(parse_static_str))]
    postgres_cert: Option<&'static str>,
}

impl From<RawProgramArgs> for ProgramArgs {
    fn from(args: RawProgramArgs) -> Self {
        Self {
            server: ServerArgs {
                port: args.port,
                address: args.address,
            },
            postgres: PostgresArgs {
                host: args.postgres_host,
                port: args.postgres_port,
                user: args.postgres_user,
                password: args.postgres_password,
                db: args.postgres_db,
                cert: args.postgres_cert,
            },
        }
    }
}

pub fn from_env() -> ProgramArgs {
    argh::from_env::<RawProgramArgs>().into()
}

#[allow(dead_code)]
const fn default_address() -> &'static str {
    "0.0.0.0"
}

#[allow(clippy::unnecessary_wraps, dead_code)]
fn parse_static_str(s: &str) -> Result<&'static str, String> {
    let s = s.to_string().into_boxed_str();
    let s: &'static str = Box::leak(s);
    Ok(s)
}
