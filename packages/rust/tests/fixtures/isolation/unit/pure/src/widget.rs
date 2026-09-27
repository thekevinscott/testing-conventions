pub fn widget() {}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn constructs_values() {
        let _ = clap::Command::new("tool");
        let _ = clap::Arg::new("name");
        let _ = clap::Error::new(ErrorKind::InvalidValue);
        let _ = syn::parse_str::<syn::File>("fn f() {}");
        let _ = syn::parse_file("fn f() {}");
        let _ = toml::from_str::<toml::Value>("a = 1");
        let _ = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let _ = zip::write::SimpleFileOptions::default();
        let _ = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let _ = tar::Builder::new(Vec::new());
        let _ = tar::Header::new_gnu();
        let _ = std::process::ExitStatus::from_raw(0);
        widget();
    }
}
