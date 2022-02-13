fn main() {
    let mut conf = prost_build::Config::new();
    conf.bytes([".hrpc.v1.Error"]);
    conf.compile_protos(&["hrpc-main/protocol/hrpc.proto"], &["hrpc-main/protocol"])
        .expect("failed to compile hRPC proto");
}
