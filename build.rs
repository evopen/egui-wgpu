fn main() {
    let shader_dir = "src/shader/";
    let vs_name = "shader.vert";
    let fs_name = "shader.frag";
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let vs_spv_path = out_dir.join("vert.spv");
    let fs_spv_path = out_dir.join("frag.spv");

    println!("cargo:rerun-if-changed={}{}", shader_dir, vs_name);
    println!("cargo:rerun-if-changed={}{}", shader_dir, fs_name);

    let mut compiler = shaderc::Compiler::new().unwrap();
    let vs_spirv = compiler
        .compile_into_spirv(
            include_str!("src/shader/shader.vert"),
            shaderc::ShaderKind::Vertex,
            vs_name,
            "main",
            None,
        )
        .unwrap();
    let fs_spirv = compiler
        .compile_into_spirv(
            include_str!("src/shader/shader.frag"),
            shaderc::ShaderKind::Fragment,
            fs_name,
            "main",
            None,
        )
        .unwrap();

    std::fs::write(&vs_spv_path, vs_spirv.as_binary_u8()).unwrap();
    std::fs::write(&fs_spv_path, fs_spirv.as_binary_u8()).unwrap();

    println!("cargo:rustc-env={}={}", "vert.spv", vs_spv_path.to_str().unwrap());
    println!("cargo:rustc-env={}={}", "frag.spv", fs_spv_path.to_str().unwrap());
}
