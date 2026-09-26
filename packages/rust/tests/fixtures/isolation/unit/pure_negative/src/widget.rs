pub fn widget() {}

#[cfg(test)]
mod tests {
    use clap::*;
    use syn::parse::*;
    use std::process::Command;
    use std::os::unix::process::*;

    #[test]
    fn effectful_and_unlisted() {
        let _ = clap::Command::new("tool");
        let _ = clap::Command::debug_assert;
        let _ = syn::parse_quote!(fn f() {});
        let _ = toml::to_string(&1);
        let _ = zip::ZipWriter::new_append(std::io::Cursor::new(Vec::new()));
        let _ = std::process::Command::new("tool");
        let _ = std::env::var("X");
        std::thread::sleep(std::time::Duration::from_secs(0));
    }
}
