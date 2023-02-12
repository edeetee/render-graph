use std::env;
use std::path::PathBuf;
use std::path::Path;

//shamelessly stolen from https://github.com/simlay/uikit-sys/blob/master/build.rs

fn sdk_path(sdk: &str) -> Result<String, std::io::Error> {
    use std::process::Command;

    let output = Command::new("xcrun")
        .args(&["--sdk", sdk, "--show-sdk-path"])
        .output()?
        .stdout;
    let prefix_str = std::str::from_utf8(&output).expect("invalid output from `xcrun`");
    Ok(prefix_str.trim_end().to_string())
}

fn main() {
    // Generate one large set of bindings for all frameworks.
    //
    // We do this rather than generating a module per framework as some frameworks depend on other
    // frameworks and in turn share types. To ensure all types are compatible across each
    // framework, we feed all headers to bindgen at once.
    //
    // Only link to each framework and include their headers if their features are enabled and they
    // are available on the target os.
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");

    let sdk_dir = PathBuf::from(sdk_path("macosx").unwrap());
    let frameworks_path = PathBuf::from("System/Library/Frameworks/");
    let gl_path = PathBuf::from("System/Library/Frameworks/OpenGL.framework/Headers/");

    dbg!(&sdk_dir, &gl_path);

    let full_gl_path = sdk_dir.join(gl_path);
    let full_gl_path_str = full_gl_path.to_str().unwrap();

    let full_frameworks_path = sdk_dir.join(frameworks_path);
    let full_frameworks_path = full_frameworks_path.to_str().unwrap();

    // Begin building the bindgen params.
    let mut builder = bindgen::Builder::default();

    // println!("cargo:rustc-link-search={full_gl_path_str}");

    let clang_args = vec![
        // format!("-isystem{full_gl_path_str}"),
        // "-framework", "OpenGL",
        format!("-F{full_frameworks_path}"),
        "-isysroot".into(), sdk_dir.to_str().unwrap().into(),
        "-x".into(), "objective-c".into()
        // "-x".to_string(), "c++".to_string(),
        // // "-std=c++14".to_string(),
        // "-IFFGLSDK/Include".to_string()
    ];

    dbg!(&clang_args);

    builder = builder
        .clang_args(&clang_args)
        .objc_extern_crate(true)
        .layout_tests(false)
        // .block_extern_crate(true)
        // .generate_block(true)
        .rustfmt_bindings(true)
        // time.h as has a variable called timezone that conflicts with some of the objective-c
        // calls from NSCalendar.h in the Foundation framework. This removes that one variable.
        // .blocklist_item("timezone")
        // // https://github.com/rust-lang/rust-bindgen/issues/1705
        // .blocklist_item("IUIStepper")
        // .blocklist_function("dividerImageForLeftSegmentState_rightSegmentState_")
        // .blocklist_item("objc_object")
        // .blocklist_type("NSImage_")
        // .blocklist_type("NSScreen_")
        .header("wrapper.h");
        // .header_contents("UIKit.h", "#include<UIKit/UIKit.h>");

    // Generate the bindings.
    let bindings = builder.generate().expect("unable to generate bindings");

    // Get the cargo out directory.
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("env variable OUT_DIR not found"));

    // Write them to the crate root.
    bindings
        .write_to_file(out_dir.join("ffgl.rs"))
        .expect("could not write bindings");
}