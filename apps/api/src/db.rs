use std::future::Future;
use std::time::Duration;

use crate::cli::PostgresArgs;
use crate::errors::ShortenerError;
use crate::shortcode;
use sqlx::PgPool;
use sqlx::error::Error;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

const MAX_ATTEMPTS: usize = 5;

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub struct StoredLink {
    pub short_code: Box<str>,
    pub original_url: Box<str>,
}

#[derive(Debug)]
enum StoreLinkError {
    Collision,
    Database(sqlx::Error),
}

/// Generate short codes and try to store one until insertion succeeds.
///
/// The purpose of this helper is to separate the retry policy from the actual
/// database write. That lets unit tests cover collision, exhaustion, and fatal
/// error behavior without doing any DB I/O or depending on Postgres-specific
/// errors.
///
/// Contract:
///
/// - `generate_code` is called once per attempt.
/// - `try_store_code` receives that code and returns:
///   - `Ok(StoredLink)` when the code was stored successfully.
///   - `StoreLinkError::Collision` when the code already exists and should be
///     regenerated.
///   - `StoreLinkError::Database` for non-retryable storage failures.
/// - after `MAX_ATTEMPTS` collisions, returns `CodeGenerationExhausted`.
///
/// The public DB wrapper maps sqlx unique-constraint errors to `Collision` and
/// all other sqlx errors to `Database`.
async fn generate_unique<F, F2, Fut>(
    mut generate_code: F,
    mut try_store_code: F2,
) -> Result<StoredLink, ShortenerError>
where
    F: FnMut() -> Box<str>,
    F2: FnMut(Box<str>) -> Fut,
    Fut: Future<Output = Result<StoredLink, StoreLinkError>>,
{
    for _ in 0..MAX_ATTEMPTS {
        let code = generate_code();

        match try_store_code(code).await {
            Ok(value) => return Ok(value),
            Err(StoreLinkError::Collision) => {}
            Err(StoreLinkError::Database(error)) => return Err(ShortenerError::Database(error)),
        }
    }

    Err(ShortenerError::CodeGenerationExhausted {
        attempts: MAX_ATTEMPTS,
    })
}

#[allow(dead_code)]
pub async fn insert_link_with_retry(
    pool: &PgPool,
    original_url: &str,
) -> Result<StoredLink, ShortenerError> {
    generate_unique(shortcode::generate, |code| async move {
        let result = sqlx::query_as::<_, StoredLink>(
            r"
            INSERT INTO links (short_code, original_url)
            VALUES ($1, $2)
            RETURNING short_code, original_url
            ",
        )
        .bind(code.as_ref())
        .bind(original_url)
        .fetch_one(pool)
        .await;

        match result {
            Ok(link) => Ok(link),
            Err(error) if is_unique_violation(&error) => Err(StoreLinkError::Collision),
            Err(error) => Err(StoreLinkError::Database(error)),
        }
    })
    .await
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, Error::Database(error) if error.is_unique_violation())
}

/// Build a Postgres connection pool from the parsed config.
///
/// Connects eagerly (`connect_with(...).await`) so an unreachable or
/// misconfigured database fails fast at startup rather than on first query.
pub async fn connect(
    PostgresArgs {
        host,
        port,
        user,
        password,
        db,
        cert: _,
        pool_size,
        acquire_timeout,
    }: PostgresArgs,
) -> Result<PgPool, ShortenerError> {
    // TODO(ops): honor `cert` for TLS (PgSslMode::VerifyFull + ssl_root_cert)
    // once the production Postgres lands. Local dev uses plain connections.
    let options = PgConnectOptions::new()
        .host(host)
        .port(port)
        .username(user)
        .password(password)
        .database(db)
        .ssl_mode(PgSslMode::Prefer);

    PgPoolOptions::new()
        .max_connections(pool_size)
        .acquire_timeout(Duration::from_secs(acquire_timeout))
        .connect_with(options)
        .await
        .map_err(ShortenerError::PostgresConnect)
}

#[cfg(test)]
mod tests {
    use crate::errors::ShortenerError;
    use sqlx::Error;

    use super::{MAX_ATTEMPTS, StoreLinkError, StoredLink, generate_unique};

    fn stored_link(short_code: Box<str>) -> StoredLink {
        StoredLink {
            short_code,
            original_url: "https://example.com".into(),
        }
    }

    #[tokio::test]
    async fn generate_unique_succeeds_after_collisions() {
        let mut attempts = 0;
        let mut codes = ["first", "second", "third"].into_iter();
        let mut attempted_codes = vec![];

        let generate_code = || codes.next().unwrap().into();
        let store_code = |code: Box<str>| {
            attempts += 1;
            attempted_codes.push(code.to_string());

            async move {
                if attempts < 3 {
                    Err(StoreLinkError::Collision)
                } else {
                    Ok(stored_link(code))
                }
            }
        };

        // Force two collisions to prove each retry uses a freshly generated code.
        let link = generate_unique(generate_code, store_code)
            .await
            .expect("insert should eventually succeed");

        assert_eq!(link.short_code, "third".into());
        assert_eq!(attempts, 3);
        assert_eq!(attempted_codes, ["first", "second", "third"]);
    }

    #[tokio::test]
    async fn generate_unique_exhausts_after_max_collisions() {
        let mut attempts = 0;
        let mut code_index = 0;
        let mut attempted_codes = vec![];

        let generate_code = || {
            let code = format!("code-{code_index}").into_boxed_str();
            code_index += 1;
            code
        };
        let store_code = |code: Box<str>| {
            attempted_codes.push(code.to_string());
            attempts += 1;
            async { Err(StoreLinkError::Collision) }
        };

        // Keep colliding to prove the loop stops instead of retrying forever.
        let error = generate_unique(generate_code, store_code)
            .await
            .expect_err("collisions should exhaust attempts");

        assert!(matches!(
            error,
            ShortenerError::CodeGenerationExhausted {
                attempts: MAX_ATTEMPTS
            }
        ));
        assert_eq!(attempts, MAX_ATTEMPTS);
        assert_eq!(
            attempted_codes,
            ["code-0", "code-1", "code-2", "code-3", "code-4"]
        );
    }

    #[tokio::test]
    async fn generate_unique_returns_non_collision_error() {
        let mut attempts = 0;
        let mut attempted_codes = vec![];

        let generate_code = || "fatal".into();
        let store_code = |code: Box<str>| {
            attempted_codes.push(code.to_string());
            attempts += 1;

            async { Err(StoreLinkError::Database(Error::RowNotFound)) }
        };

        // Non-collision storage errors should return immediately and not retry.
        let error = generate_unique(generate_code, store_code)
            .await
            .expect_err("fatal error should return immediately");

        assert!(matches!(
            error,
            ShortenerError::Database(Error::RowNotFound)
        ));
        assert_eq!(attempts, 1);
        assert_eq!(attempted_codes, ["fatal"]);
    }

    #[tokio::test]
    async fn generate_unique_does_not_generate_after_success() {
        let mut generated_codes = 0;

        let generate_code = || {
            generated_codes += 1;
            "success".into()
        };
        let store_code = |code| async move { Ok(stored_link(code)) };

        // A first-attempt success should not consume extra generated codes.
        let link = generate_unique(generate_code, store_code)
            .await
            .expect("insert should succeed");

        assert_eq!(link.short_code, "success".into());
        assert_eq!(generated_codes, 1);
    }
}
