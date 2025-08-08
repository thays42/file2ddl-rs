use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();
    file2ddl::run()
}
