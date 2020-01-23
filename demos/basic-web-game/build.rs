fn main() -> Result<(), std::io::Error> {
    oxygengine_build_tools::pipeline::Pipeline::from_file("pipeline.json", true)?.execute()?;
    Ok(())
}
