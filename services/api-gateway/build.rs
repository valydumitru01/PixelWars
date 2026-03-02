fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &[
        "../../proto/auth.proto",
        "../../proto/canvas.proto",
        "../../proto/chat.proto",
        "../../proto/voting.proto",
        "../../proto/groups.proto",
    ];
    tonic_build::configure()
        .build_server(false)   // gateway only needs clients
        .build_client(true)
        .compile_protos(proto_files, &["../../proto"])?;
    Ok(())
}
