fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "../proto/feather/v1/common.proto",
                "../proto/feather/v1/job.proto",
                "../proto/feather/v1/queue.proto",
                "../proto/feather/v1/worker.proto",
            ],
            &["../proto"],
        )?;
    Ok(())
}
