use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const ZSH_REPO_URL: &str = "https://git.code.sf.net/p/zsh/code";
const ZSH_REVISION: &str = "zsh-5.9";

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let zsh_dir = out_dir.join("zsh");
    let header_path = zsh_dir.join("Src/Zle/zle.mdh");
    let bindings_path = out_dir.join("bindings.rs");
    let have_header = header_path.exists();
    let have_bindings = bindings_path.exists();

    if have_header && have_bindings {
        return;
    }

    if zsh_dir.exists() {
        let _ = fs::remove_dir_all(&zsh_dir);
    }

    Command::new("git")
        .current_dir(&out_dir)
        .arg("clone")
        .arg(ZSH_REPO_URL)
        .arg(&zsh_dir)
        .status()
        .unwrap();

    Command::new("git")
        .current_dir(&zsh_dir)
        .arg("checkout")
        .arg(ZSH_REVISION)
        .status()
        .unwrap();

    Command::new("sh")
        .current_dir(&zsh_dir)
        .arg("Util/preconfig")
        .status()
        .unwrap();

    Command::new("sh")
        .current_dir(&zsh_dir)
        .arg("configure")
        .status()
        .unwrap();

    Command::new("make")
        .arg("headers")
        .current_dir(zsh_dir.join("Src"))
        .status()
        .unwrap();

    if !header_path.exists() {
        panic!(
            "expected header {} to exist after configuring zsh",
            header_path.display()
        );
    }

    let bindings = bindgen::Builder::default()
        .header_contents(
            "wrapper.h",
            &format!("#include \"{}\"", header_path.to_str().unwrap()),
        )
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings!");
}
