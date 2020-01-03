use oxygengine_build_tools::pipeline::*;

fn main() -> Result<(), std::io::Error> {
    Pipeline::default()
        .project_source("static")
        .project_destination("static")
        .pack(PackPhase::default().path("assets").output("assets.pack"))
        .execute()
}
