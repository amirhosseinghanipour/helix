use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

mod commands;
mod core;
mod utils;

use commands::*;
use core::repository::Repository;

#[derive(Parser)]
#[command(name = "hx")]
#[command(about = "🚀 A modern, fast Git alternative with better UX")]
#[command(version = "0.1.0")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Helix repository
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Add files to staging area
    Add {
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Commit staged changes
    Commit {
        #[arg(short, long)]
        message: String,
    },
    /// Show repository status
    Status,
    /// Show commit history
    Log {
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Create a new branch
    Branch {
        #[arg(default_value = "")]
        name: String,
    },
    /// Switch between branches
    Checkout {
        branch: String,
    },
    /// Merge branches
    Merge {
        branch: String,
    },
    /// Clone a repository
    Clone {
        url: String,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Push changes to remote
    Push {
        #[arg(long)]
        force: bool,
        #[arg(long)]
        remote: Option<String>,
        #[arg(long)]
        refspec: Option<String>,
    },
    /// Pull changes from remote
    Pull {
        #[arg(long)]
        remote: Option<String>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        rebase: bool,
    },
    /// Show differences
    Diff {
        #[arg(default_value = "")]
        path: Option<PathBuf>,
    },
    /// Reset repository state
    Reset {
        #[arg(default_value = "HEAD")]
        target: String,
        #[arg(long, default_value = "mixed")]
        mode: Option<String>,
    },
    /// Add a remote repository
    Remote {
        #[arg(short, long)]
        add: Option<String>,
        #[arg(short, long)]
        url: Option<String>,
    },
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        subcommand: AuthSubcommand,
    },
    /// Restore files from the last commit
    Restore {
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Key management
    Keygen,
    KeyShow,
    KeyImport {
        path: String,
    },
    KeyExport {
        path: String,
    },
    /// Visualize the commit DAG
    Dag,
}

#[derive(Subcommand)]
enum AuthSubcommand {
    /// Add authentication for a host
    Add {
        host: String,
        #[arg(long)]
        token: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        ssh_key: Option<String>,
    },
    /// Remove authentication for a host
    Remove {
        host: String,
    },
    /// List configured authentication
    List,
    /// Test authentication for a host
    Test {
        host: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Print beautiful header
    if let Commands::Init { .. } = &cli.command {
        println!("{}", "🚀 Helix - Modern Version Control".bold().blue());
        println!("{}", "=".repeat(40).blue());
    }

    match &cli.command {
        Commands::Init { path } => {
            init::init_repository(path).await?;
        }
        Commands::Add { paths } => {
            let mut repo = Repository::open(".")?;
            add::add_files(&mut repo, paths).await?;
        }
        Commands::Commit { message } => {
            let mut repo = Repository::open(".")?;
            let keypair =
                utils::key_utils::load_keypair().expect("No keypair found. Run 'hx keygen' first.");
            commit::commit_changes(&mut repo, message, &keypair).await?;
        }
        Commands::Status => {
            let repo = Repository::open(".")?;
            status::show_status(&repo).await?;
        }
        Commands::Log { limit } => {
            let repo = Repository::open(".")?;
            log::show_log(&repo, *limit).await?;
        }
        Commands::Branch { name } => {
            let mut repo = Repository::open(".")?;
            if name.is_empty() {
                branch::list_branches(&repo).await?;
            } else {
                branch::create_branch(&mut repo, name).await?;
            }
        }
        Commands::Checkout { branch } => {
            let mut repo = Repository::open(".")?;
            checkout::checkout_branch(&mut repo, branch).await?;
        }
        Commands::Merge { branch } => {
            let mut repo = Repository::open(".")?;
            merge::merge_branch(&mut repo, branch).await?;
        }
        Commands::Clone { url, path } => {
            clone::clone_repository(url, path).await?;
        }
        Commands::Push { force, remote, refspec } => {
            let repo = Repository::open(".")?;
            push::push_with_options(&repo, *force, remote.as_deref(), refspec.as_deref()).await?;
        }
        Commands::Pull { remote, branch, rebase } => {
            let repo = Repository::open(".")?;
            pull::pull_with_options(&repo, remote.as_deref(), branch.as_deref(), *rebase).await?;
        }
        Commands::Diff { path } => {
            let repo = Repository::open(".")?;
            diff::show_diff(&repo, path.as_ref().map(|v| &**v)).await?;
        }
        Commands::Reset { target, mode } => {
            let mut repo = Repository::open(".")?;
            let mode = mode.clone().unwrap_or("mixed".to_string());
            reset::reset_repository(&mut repo, target, &mode).await?;
        }
        Commands::Remote { add, url } => {
            let mut repo = Repository::open(".")?;
            if let (Some(name), Some(remote_url)) = (add, url) {
                repo.add_remote(&name, &remote_url)?;
                println!(
                    "{}",
                    format!("Added remote '{}' -> {}", name, remote_url)
                        .green()
                        .bold()
                );
            } else {
                println!("{}", "Usage: hx remote --add <name> --url <url>".yellow());
            }
        }
        Commands::Auth { subcommand } => {
            let mut auth_manager = utils::auth::AuthManager::new()?;
            match subcommand {
                AuthSubcommand::Add { host, token, username, password, ssh_key } => {
                    let mut config = utils::auth::AuthConfig::new(&host);
                    
                    if let Some(token_val) = token {
                        config = config.with_token(&token_val);
                    } else if let (Some(user), Some(pass)) = (username, password) {
                        config = config.with_basic_auth(&user, &pass);
                    } else if let Some(key_path) = ssh_key {
                        config = config.with_ssh(Some(std::path::PathBuf::from(key_path)));
                    } else {
                        config = config.with_ssh(None);
                    }
                    
                    auth_manager.add_config(&host, config)?;
                    println!("{}", format!("Added authentication for {}", host).green().bold());
                }
                AuthSubcommand::Remove { host } => {
                    auth_manager.remove_config(&host)?;
                    println!("{}", format!("Removed authentication for {}", host).green().bold());
                }
                AuthSubcommand::List => {
                    // TODO: Implement list functionality
                    println!("{}", "Authentication configurations:".bold());
                    println!("(List functionality not yet implemented)");
                }
                AuthSubcommand::Test { host } => {
                    // TODO: Implement test functionality
                    println!("{}", format!("Testing authentication for {}", host).bold());
                    println!("(Test functionality not yet implemented)");
                }
            }
        }
        Commands::Restore { paths } => {
            let repo = Repository::open(".")?;
            restore::restore_files(&repo, paths.clone()).await?;
        }
        Commands::Keygen => {
            let _key = utils::key_utils::generate_and_save_keypair()?;
            println!("{}", "Keypair generated and saved!".green().bold());
        }
        Commands::KeyShow => {
            if utils::key_utils::keypair_exists() {
                let key = utils::key_utils::load_keypair()?;
                println!("Public key: {:x?}", key.verifying_key().to_bytes());
            } else {
                println!("No keypair found. Run 'hx keygen' to generate one.");
            }
        }
        Commands::KeyImport { path } => {
            utils::key_utils::import_keypair(path)?;
            println!("{}", "Keypair imported!".green().bold());
        }
        Commands::KeyExport { path } => {
            utils::key_utils::export_keypair(path)?;
            println!("{}", "Keypair exported!".green().bold());
        }
        Commands::Dag => {
            let repo = Repository::open(".")?;
            log::show_dag(&repo).await?;
        }
    }

    Ok(())
}
