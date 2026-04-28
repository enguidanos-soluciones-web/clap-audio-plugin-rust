fn main() {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR env var"));

    let clap_bindings = bindgen::Builder::default()
        .header("external/clap-1.2.7/include/clap/clap.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate_cstr(true)
        .generate_comments(true)
        .generate()
        .expect("clap bindgen failed");

    clap_bindings.write_to_file(out.join("clap.rs")).expect("write failed");

    cxx_build::bridge("src/dsp/nam.rs")
        .file("src/dsp/nam.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/activations.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/container.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/conv1d.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/convnet.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/dsp.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/get_dsp.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/lstm.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/ring_buffer.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/util.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/wavenet/model.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/wavenet/a2_fast.cpp")
        .file("external/neural-amp-modeler-0.5.1/NAM/wavenet/slimmable.cpp")
        .include("src/dsp")
        .include("external/neural-amp-modeler-0.5.1/NAM")
        .include("external/neural-amp-modeler-0.5.1/Dependencies/eigen")
        .include("external/neural-amp-modeler-0.5.1/Dependencies/nlohmann")
        .std("c++20")
        .flag_if_supported("-w")
        .compile("nam");

    println!("cargo::rerun-if-changed=src/dsp/nam.rs");
    println!("cargo::rerun-if-changed=src/dsp/nam.h");
    println!("cargo::rerun-if-changed=src/dsp/nam.cpp");
}
