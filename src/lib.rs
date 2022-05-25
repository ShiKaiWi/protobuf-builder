//! Builder for generating Rust code from protos
//!
//! These functions panic liberally, they are designed to be used from build
//! scripts, not in production.
//!
//! Some codes are borrowed from <https://github.com/tikv/protobuf-build/blob/4e57d66934a5f45774ad41bbc8650028c430ad66/src/lib.rs>

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use protoc::Protoc;

fn check_and_get_protoc_bin_path() -> PathBuf {
    let path = protoc_bin_vendored::protoc_bin_path().unwrap();
    assert!(Protoc::from_path(&path).version().unwrap().is_3());
    path
}

/// Rust code builder for protos
pub struct Builder {
    /// Cargo output directory for protos
    out_dir: String,
    /// Protobuf files to generate
    files: Vec<String>,
    /// Protobuf include directory
    include_dir: String,
}

impl Builder {
    /// Create a new Builder
    pub fn new() -> Self {
        Self {
            out_dir: format!(
                "{}/protos",
                env::var("OUT_DIR").expect("No OUT_DIR defined")
            ),
            files: Vec::new(),
            include_dir: "protos".to_string(),
        }
    }

    /// Generate Rust code
    pub fn generate(&self) {
        assert!(!self.files.is_empty(), "No files specified for generation");

        self.prepare_out_dir();
        self.generate_files();
        self.generate_mod_file();
    }

    /// Set `out_dir`, default is `$OUT_DIR/protos`
    pub fn out_dir(&mut self, out_dir: impl Into<String>) -> &mut Self {
        self.out_dir = out_dir.into();
        self
    }

    fn prepare_out_dir(&self) {
        if Path::new(&self.out_dir).exists() {
            fs::remove_dir_all(&self.out_dir).unwrap();
        }
        fs::create_dir_all(&self.out_dir).unwrap();
    }

    fn generate_files(&self) {
        protoc_grpcio::compile_grpc_protos(
            // inputs
            &self.files,
            // includes
            &[&self.include_dir],
            // output
            &self.out_dir,
            // customizations
            None,
            // protoc path
            Some(Protoc::from_path(&check_and_get_protoc_bin_path())),
        )
        .expect("Failed to compile protobuf and grpc files");
    }

    fn generate_mod_file(&self) {
        let mut f = File::create(format!("{}/mod.rs", self.out_dir)).unwrap();

        let mut modules: Vec<_> = self
            .list_rs_files()
            .filter_map(|path| {
                let name = path.file_stem().unwrap().to_str().unwrap();
                if name == "mod" {
                    return None;
                }

                Some(name.to_owned())
            })
            .collect();

        modules.sort();

        for module in modules {
            writeln!(f, "pub mod {};", module).unwrap();
        }
    }

    // List all `.rs` files in `out_dir`
    fn list_rs_files(&self) -> impl Iterator<Item = PathBuf> {
        fs::read_dir(&self.out_dir)
            .expect("Couldn't read directory")
            .filter_map(|e| {
                let path = e.expect("Couldn't list file").path();
                if path.extension() == Some(std::ffi::OsStr::new("rs")) {
                    Some(path)
                } else {
                    None
                }
            })
    }

    /// Finds proto files to operate on in the `proto_dir` directory.
    pub fn search_dir_for_protos(&mut self, proto_dir: &str) -> &mut Self {
        self.files = fs::read_dir(proto_dir)
            .expect("Couldn't read proto directory")
            .filter_map(|e| {
                let e = e.expect("Couldn't list file");
                let path = e.path();
                if e.file_type().expect("File broken").is_dir()
                    || path.extension() != Some(std::ffi::OsStr::new("proto"))
                {
                    None
                } else {
                    Some(format!("{}/{}", proto_dir, e.file_name().to_string_lossy()))
                }
            })
            .collect();
        self
    }
}
