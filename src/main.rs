use anyhow::Result;
use env_logger;

fn main() -> Result<()> {
    env_logger::init();
    file2ddl::run()
}
