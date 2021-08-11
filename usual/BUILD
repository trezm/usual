load("@rules_rust//rust:rust.bzl", "rust_library")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "usual",
    srcs = [
        "src/base.rs",
        "src/lib.rs",
    ],
    crate_features = [
        "tokio-postgres",
    ],
    edition = "2018",
    proc_macro_deps = [
        "//usual/usual-proc",
    ],
    deps = [
        "//usual/usual/cargo:anyhow",
        "//usual/usual/cargo:chrono",
        "//usual/usual/cargo:dotenv",
        "//usual/usual/cargo:serde",
        "//usual/usual/cargo:tokio",
        "//usual/usual/cargo:tokio_postgres",
    ],
)
