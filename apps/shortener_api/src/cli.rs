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
    pub cors_allowed_origins: &'static [&'static str],
    /// Public host of this shortener (`host[:port]`), used to reject shortening
    /// URLs that point back at the service. Distinct from `cors_allowed_origins`.
    pub host: &'static str,
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

    /// comma-separated list of CORS-allowed origins (<scheme://host[:port]>)
    #[argh(option, env = "CORS_ALLOWED_ORIGINS", from_str_fn(parse_cors_origins))]
    cors_allowed_origins: &'static [&'static str],

    /// public host of this shortener, used to reject self-referential URLs
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
                cors_allowed_origins: args.cors_allowed_origins,
                host: args.host,
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

#[allow(clippy::unnecessary_wraps)]
fn parse_static_str(s: &str) -> Result<&'static str, String> {
    let s = s.to_string().into_boxed_str();
    let s: &'static str = Box::leak(s);
    Ok(s)
}

/// Parse a comma-separated list of CORS-allowed origins. Each token must be a
/// full origin (`scheme://host[:port]`) with an `http`/`https` scheme and no
/// path, query, fragment, or userinfo — i.e. exactly what a browser sends in the
/// `Origin` header. The original trimmed token is preserved (not re-serialized
/// via `Url`, which would append a trailing `/`) so it matches the header byte
/// for byte.
fn parse_cors_origins(s: &str) -> Result<&'static [&'static str], String> {
    let mut origins = vec![];

    for token in s.split(',') {
        let origin = token.trim();
        if origin.is_empty() {
            return Err("CORS_ALLOWED_ORIGINS must not contain empty entries".to_string());
        }

        let url = Url::parse(origin)
            .map_err(|_| format!("{origin:?} is not a valid origin (scheme://host[:port])"))?;

        // `Url::parse` gives both `https://x` and `https://x/` a `/` path, but the
        // browser `Origin` header never carries a trailing slash — reject it so the
        // stored token matches the header exactly.
        let is_origin = matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.path() == "/"
            && !origin.ends_with('/')
            && url.query().is_none()
            && url.fragment().is_none();

        if !is_origin {
            return Err(format!(
                "{origin:?} must be an origin like https://app.example.com (scheme://host[:port], no path)"
            ));
        }

        origins.push(parse_static_str(origin)?);
    }

    if origins.is_empty() {
        return Err("CORS_ALLOWED_ORIGINS must list at least one origin".to_string());
    }

    Ok(Box::leak(origins.into_boxed_slice()))
}

/// Validate the shortener's own public host: a bare `host[:port]` with no scheme,
/// path, query, fragment, or userinfo. Used to reject self-referential shortens.
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
    use super::{parse_cors_origins, parse_public_host};

    #[test]
    fn parse_cors_origins_accepts_single_and_multiple_origins() {
        assert_eq!(
            parse_cors_origins("https://example.com").expect("origin should parse"),
            ["https://example.com"]
        );
        assert_eq!(
            parse_cors_origins("https://a.example.com,http://localhost:8080")
                .expect("origins should parse"),
            ["https://a.example.com", "http://localhost:8080"]
        );
    }

    #[test]
    fn parse_cors_origins_trims_surrounding_whitespace() {
        assert_eq!(
            parse_cors_origins(" https://a.com , http://localhost:8080 ")
                .expect("origins should parse"),
            ["https://a.com", "http://localhost:8080"]
        );
    }

    #[test]
    fn parse_cors_origins_rejects_invalid_entries() {
        for value in [
            "",
            "example.com",                    // bare host, no scheme
            "https://example.com,",           // trailing empty entry
            "https://example.com/x",          // has a path
            "https://example.com/",           // trailing slash won't match Origin header
            "https://example.com?debug=true", // has a query
            "ftp://example.com",              // unsupported scheme
            "user@example.com",
        ] {
            assert!(
                parse_cors_origins(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

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
