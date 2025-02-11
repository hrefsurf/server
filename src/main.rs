/*
    hrefsurf - main.rs
    Copyright (C) 2025 Isaac Trimble-Pederson

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

/*
    A disclaimer on the architecture of this program,

    There probably is very little good architecture here.

    I have tried to create a lot of programs in my spare time but got entirely
    demotivated trying to do things in a test-driven fashion, like at most of
    my workplaces - I do not have a ton of energy to focus everything on
    exhaustive unit-testability, and I'm not convinced this is a productive
    tradeoff in general, since that usually results in ten engineers working on
    one thing, with nobody making any level of meaningful progress, and locking
    in holier-than-thou cleanliness debates.

    I am also quite new to the Rust programming language, and my day job pulls
    me away from this to work on Swift/iOS, which have somewhat unique 
    constraints that won't help me here. So there are probably newbish patterns
    throughout this codebase.

    So what you can expect:
    - Things are not perfectly structured
    - In fact a lot of code will be bad code
    - Not everything has a corresponding unit test, and integration tests may
    be lacking in practice
    - Some implementations may be handrolled where there are existing crates
    or solutions
    - I am not a legal expert and there may be issues with being able to host
    this to EU users. I do not know if I am in full compliance with Cali.
    law either in terms of privacy/security.

    On the other hand:
    - I will try and unit test nontrivial logic that I am in charge of
    - I will try and maintain a good set of integration tests that cover
    the featureset in a reasonable manner
    
    Please try and adhere to the above if you make a contribution to the
    project.
    
    If there are ways I can concretely improve the project to make it more
    accessible, testable, quality, well-structured, let me know and 
    contribute your own PR. Otherwise, just keep the above caveats in mind -
    this is a side project from a random dude on the Internet who barely has the
    mental energy to do his day job, let alone contribute something into open
    source.

    This is not reflective of how I program at the workplace (for better and
    worse alike)

    -Isaac
*/

use clap::Parser;
use router::build_router;
use sqlx::mysql::MySqlConnectOptions;
use tokio::net::TcpListener;

mod auth; // TODO: Move to dedicated handlers
mod router;
mod state;
mod errors;
mod models;

static STARTUP_MESSAGE: &str = "
Starting up hrefsurf_server.

hrefsurf is Free Software under the GNU Affero General Public License version \
3 or later, at your option.

You are entitled to a copy of the source code of this application. If you were \
not provided it, you should be able to request a copy. If someone has provided \
you this software directly or via usage on a website, and you are unable to \
obtain the source code, REPORT THIS AT \
https://github.com/hrefsurf

If you're reading this message, whether you are a directory host, a \
contributor to this project or its source code, or a random onlooker studying \
logs or the source code - welcome! Together, let's make a better Internet. 

Have a good day :-) [this will seem sarcastic if i crash]
";

// MARK: CLI code
// TODO: Move this into its own file
#[derive(clap_derive::Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// Host address and port to run the server on. (e.g., 0.0.0.0:8080)
    #[arg(long, default_value="0.0.0.0:8080")]
    host: String,

    /// Username to use in the MariaDB connection.
    #[arg(long, default_value="root")]
    db_username: String,

    /// Password to use in the MariaDB connection.
    #[arg(long, default_value="root")]
    db_password: String,

    /// Host address of the MariaDB database.
    #[arg(long, default_value="127.0.0.1")]
    db_address: String,

    /// Database to use within the MariaDB server provided.
    #[arg(long, default_value="hrefsurf")]
    db_name: String
}

struct DbOptions {
    host: String,
    username: String,
    password: String,
    database: String
}

impl Into<MySqlConnectOptions> for DbOptions {
    fn into(self) -> MySqlConnectOptions {
        MySqlConnectOptions::new()
            .host(&self.host)
            .port(3306)
            .username(&self.username)
            .password(&self.password)
            .database(&self.database)
    }
}

// MARK: Server code
async fn bind_and_serve(
    db_options: DbOptions,
    tera: tera::Tera
) -> anyhow::Result<()> {
    // MARK: Database pool creation
    tracing::info!("Connecting to database...");
    let db_options = db_options.into();
    let db_pool = sqlx::MySqlPool::connect_with(db_options).await?;
    tracing::info!("Database connected!");

    // MARK: Bind and serve
    tracing::info!("Binding to 0.0.0.0:8080");
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    tracing::debug!("Building router...");
    let router = build_router()
    .with_state(
        state::AppState {
            db_pool: db_pool.clone(),
            tera
        }
    );

    // TODO: Handle SIGINT error
    // Will likely need to be done via channels in order to handle threading concerns
    // Also see if we can handle any other sensible shutdown protocols/signals.

    tracing::info!("Serving router.");
    let result = 
        axum::serve(listener, router)
            .await;

    tracing::warn!("Something caused the serving session to close. Closing database pool...");
    // ! Something has gone wrong if we are here. Gracefully close DB.
    db_pool.close().await;
    tracing::debug!("Pool closed!");

    // Propagate the error
    result?;
    // Below in theory would never be called, but if there is some 
    // graceful shutdown perhaps it would be? Unsure how to gracefully shut
    // down an Axum server.
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // MARK: Setup logger
    tracing_subscriber::fmt::init();
    tracing::info!("{}", STARTUP_MESSAGE);

    // MARK: Parse CLI arguments
    tracing::debug!("Parsing CLI arguments...");
    let args = Cli::parse();
    tracing::debug!("CLI arguments parsed!");

    let db_options = DbOptions {
        host: args.db_address,
        username: args.db_username,
        password: args.db_password,
        database: args.db_name
    };

    // MARK: Tera template loading
    /*
    Putting the Tera instantiation logic here for now, since this could fail.
    There might be a better place for this to go, so maybe move this later.
    TODO: Move this?
     */
    let tera = tera::Tera::new("src/templates/**/*.html");
    if let Err(err) = tera {
        tracing::error!("Could not instantiate Tera templates!");
        return Err(anyhow::anyhow!(err));
    }
    let tera = tera.unwrap();

    // MARK: Start runtime

    tracing::debug!("Spawning runtime...");
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    tracing::debug!("Tokio runtime spawned!");
    tracing::debug!("Blocking on bind_and_serve");
    let result = tokio_runtime.block_on(
        bind_and_serve(
            db_options,
            tera
        )
    );

    tracing::info!("Axum has stopped serving.");
    if let Err(e) = result {
        tracing::error!("Critical error occurred during serving: {}", e);
    }

    Ok(())
}
