use std::path::Path;

pub trait Exporter {
    fn export<T>(&self, output_dir: T) -> anyhow::Result<()>
    where T: AsRef<Path>;
}

