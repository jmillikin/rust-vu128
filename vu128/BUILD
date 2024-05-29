load(
    "@rules_rust//rust:defs.bzl",
    "rust_clippy",
    "rust_doc",
    "rust_doc_test",
    "rust_library",
    "rust_test",
)

rust_library(
    name = "vu128",
    srcs = ["vu128.rs"],
    edition = "2018",
    visibility = ["//visibility:public"],
)

rust_test(
    name = "vu128_test",
    size = "small",
    srcs = ["vu128_test.rs"],
    edition = "2018",
    deps = [":vu128"],
)

rust_clippy(
    name = "vu128_clippy",
    deps = [":vu128"],
)

rust_doc(
    name = "vu128_doc",
    crate = ":vu128",
)

rust_doc_test(
    name = "vu128_doc_test",
    crate = ":vu128",
)
