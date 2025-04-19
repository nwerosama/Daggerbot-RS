fn main() { tonic_build::compile_protos("../protobufs/monica.proto").unwrap(); }
