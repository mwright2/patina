use std::{env, error::Error, fs, path::PathBuf};

const DEFAULT_DATABASE: &str = "data/patina.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let path = env::var_os("PATINA_DATABASE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATABASE));

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let pool = patina::db::connect(&path).await?;
    println!("Database ready at {}", path.display());

    pool.close().await;
    Ok(())
}
