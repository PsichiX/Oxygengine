use std::{env::var, path::Path};

fn main() -> Result<(), std::io::Error> {
    let out_dir = var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = Path::new(&out_dir).join("static/assets");
    let dest_path = Path::new(&out_dir).join("static/assets.pack");

    oxygengine_build_tools::pack::pack_assets_and_write_to_file(src_path, dest_path, false)
}
