fn main() {
    hrpc_build::compile_protos("src/chat.proto").expect("could not compile the proto");
}
