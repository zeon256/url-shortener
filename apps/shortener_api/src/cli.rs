use argh_env::FromArgs;
use url::Url;

#[derive(Clone, Debug)]
pub struct ProgramArgs {
    pub server: ServerArgs,
    pub postgres: PostgresArgs,
}

#[derive(Clone, Copy, Debug)]
pub struct ServerArgs {
    pub port: u16,
    pub address: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct PostgresArgs {
    pub host: &'static str,
    pub port: u16,
    pub user: &'static str,
    pub password: &'static str,
    pub db: &'static str,
    // TODO(ops): wire TLS (PgSslMode::VerifyFull + ssl_root_cert) once the prod
    // Postgres lands; unused for now since local dev uses plain connections.
    #[allow(dead_code)]
    pub cert: Option<&'static str>,
    pub pool_size: u32,
    pub acquire_timeout: u64,
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

    /// public host used when generating short URLs
    #[argh(option, env = "HOST", from_str_fn(parse_public_host))]
    host: &'static str,

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

    /// postgres connection pool size
    #[argh(option, default = "5", env = "POSTGRES_POOL_SIZE")]
    postgres_pool_size: u32,

    /// postgres acquire timeout in seconds
    #[argh(option, default = "5", env = "POSTGRES_ACQUIRE_TIMEOUT")]
    postgres_acquire_timeout: u64,
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
                pool_size: args.postgres_pool_size,
                acquire_timeout: args.postgres_acquire_timeout,
            },
        }
    }
}

pub fn from_env() -> ProgramArgs {
    argh_env::from_env::<RawProgramArgs>().into()
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

fn parse_public_host(s: &str) -> Result<&'static str, String> {
    if s.is_empty() {
        return Err("HOST must not be empty".to_string());
    }

    if s.trim() != s {
        return Err("HOST must not contain leading or trailing whitespace".to_string());
    }

    if s.contains('/') {
        return Err("HOST must not include a scheme or path".to_string());
    }

    let url = Url::parse(&format!("https://{s}"))
        .map_err(|_| "HOST must be a host name with an optional port".to_string())?;
    let host_only = url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none();

    if !host_only {
        return Err(
            "HOST must be a host name with an optional port, without scheme or path".to_string(),
        );
    }

    parse_static_str(s)
}

#[cfg(test)]
mod tests {
    use super::parse_public_host;

    #[test]
    fn parse_public_host_accepts_host_with_optional_port() {
        for host in ["example.com", "api.example.com:443", "localhost:4002"] {
            assert_eq!(parse_public_host(host).expect("host should parse"), host);
        }
    }

    #[test]
    fn parse_public_host_rejects_url_or_path_values() {
        for host in [
            "",
            "https://example.com",
            "example.com/path",
            "example.com/",
            "example.com?debug=true",
            "user@example.com",
        ] {
            assert!(
                parse_public_host(host).is_err(),
                "{host:?} should be rejected"
            );
        }
    }
}
