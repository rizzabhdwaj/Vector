fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/heartbeat.proto")?;
    tonic_build::compile_protos("../worker/proto/task.proto")?;
    Ok(())
}
