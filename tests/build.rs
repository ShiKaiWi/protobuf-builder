// Copyright 2019 PingCAP, Inc.

use protobuf_builder::Builder;

fn main() {
    Builder::new().search_dir_for_protos("proto").generate()
}
