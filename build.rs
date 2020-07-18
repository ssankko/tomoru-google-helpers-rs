use std::{collections::HashMap, process::Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "_rpc")]
    let builder = tonic_build::configure().build_server(false);

    #[cfg(all(feature = "_google", feature = "_rpc"))]
    // if let Err(_) = std::fs::File::open("./src/google/generated.rs") {
    {
        println!("building shit for google");
        builder.clone().compile(
            &[
                "apis/google/logging/v2/logging.proto",
                "apis/google/cloud/speech/v1/cloud_speech.proto",
                "apis/google/cloud/texttospeech/v1/cloud_tts.proto",
                "apis/google/cloud/tasks/v2beta3/cloudtasks.proto",
            ],
            &["apis/"],
        )?;
        println!("shit for google was built");

        place_in_src("google/generated");
    }
    // }

    #[cfg(all(feature = "_yandex", feature = "_rpc"))]
    // if let Err(_) = std::fs::File::open("./src/yandex/generated.rs") {
    {
        std::thread::sleep(std::time::Duration::from_millis(20));
        println!("building shit for yandex");
        builder.compile(
            &["apis/yandex/cloud/ai/stt/v2/stt_service.proto"],
            &["apis/"],
        )?;
        println!("shit for yandex was built");

        place_in_src("yandex/generated");
    }
    // }
    Ok(())
}

fn place_in_src(file_name: &str) {
    println!("file_name = {}", file_name);
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("out_dir = {}", out_dir);
    let files = std::fs::read_dir(&out_dir).unwrap();
    // extract file names from output directory
    let file_names = files
        // prost constructs file names based on a grpc package name, which
        // in turn must be valid utf-8 identifier, so i use to_string_lossy fearlessly
        .map(|x| x.unwrap().file_name().to_string_lossy().to_string());

    // --------
    // traverse all files and construct tree-like structure of namespaces
    // --------
    let mut tree = TreeEntry::Branch(Default::default());
    for file_name in file_names {
        let mut current_branch = &mut tree;
        // split names by dot.
        // `tonic_build` uses dots to represent namespaces
        // for example google.logging.v2.rs will become
        // [google, logging, v2, rs]
        for part in file_name.split('.') {
            if part == "rs" {
                *current_branch = TreeEntry::Node(file_name.to_string());
                continue;
            }

            if let None = current_branch.get(part) {
                current_branch.insert(part.to_owned(), TreeEntry::Branch(Default::default()));
            }
            current_branch = current_branch.get_mut(part).unwrap();
        }
    }
    // --------

    // simple recursive function to construct mod tree based on a
    // tree built earlier
    fn construct(tree_entry: Box<TreeEntry>, result: &mut String, out_dir: &str) {
        match *tree_entry {
            TreeEntry::Node(node) => {
                let contents = std::fs::read_to_string(&format!("{}/{}", out_dir, node)).unwrap();
                result.push_str(&contents);
            }
            TreeEntry::Branch(branch) => {
                for (name, child) in branch {
                    result.push_str(&format!("pub mod {} {{", name));
                    construct(child, result, out_dir);
                    result.push_str("}");
                }
            }
        }
    };

    let mut result = String::new();
    construct(Box::new(tree), &mut result, &out_dir);
    std::fs::write(format!("./src/{}.rs", file_name), result).unwrap();

    if std::env::var("NO_FMT_ON_GENERATED").is_err() {
        let result = Command::new("rustfmt")
            .arg("--emit")
            .arg("files")
            .arg("--edition")
            .arg("2018")
            .arg(format!("./src/{}.rs", file_name))
            .output();

        match result {
            Err(e) => {
                println!("error running rustfmt: {:?}", e);
            }
            Ok(output) => {
                if !output.status.success() {
                    let err = String::from_utf8(output.stderr).unwrap();
                    println!("rustfmt returned unsuccessful status: {}", err);
                }
            }
        }
    }

    for entry in std::fs::read_dir(out_dir).unwrap() {
        let entry = entry.unwrap();
        std::fs::remove_file(entry.path()).unwrap();
    }
}

enum TreeEntry {
    Node(String),
    Branch(HashMap<String, Box<TreeEntry>>),
}

impl TreeEntry {
    fn get_mut(&mut self, part: &str) -> Option<&mut Box<TreeEntry>> {
        match self {
            TreeEntry::Branch(tree) => tree.get_mut(part),
            _ => panic!(),
        }
    }

    fn get(&mut self, part: &str) -> Option<&Box<TreeEntry>> {
        match self {
            TreeEntry::Branch(tree) => tree.get(part),
            _ => panic!(),
        }
    }

    fn insert(&mut self, part: String, node: TreeEntry) {
        match self {
            TreeEntry::Branch(tree) => {
                tree.insert(part, Box::new(node));
            }
            _ => panic!(),
        }
    }
}
