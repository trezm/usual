load("@rules_rust//rust:defs.bzl", "rust_proc_macro")

package(default_visibility = ["//visibility:public"])

rust_proc_macro(
    name = "usual-proc",
    srcs = ["src/lib.rs"],
    edition = "2018",
    deps = [
        "//usual/usual-proc/cargo:proc_macro2",
        "//usual/usual-proc/cargo:quote",
        "//usual/usual-proc/cargo:regex",
        "//usual/usual-proc/cargo:syn",
    ],
)
