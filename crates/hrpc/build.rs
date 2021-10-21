fn main() {
    let mut conf = prost_build::Config::new();
    conf.bytes([".hrpc.v1.Error"]);

    hrpc_build::configure()
        .compile_with_config(
            conf,
            &["hrpc-main/protocol/hrpc.proto"],
            &["hrpc-main/protocol"],
        )
        .expect("failed to compile hRPC proto");
}
