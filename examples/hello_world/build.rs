fn main() {
    hrpc_build::compile_protos("src/hello.proto").expect("could not compile the proto");
}
