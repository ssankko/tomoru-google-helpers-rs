use std::{
    collections::HashMap,
    process::{exit, Command},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(_) = std::fs::File::open("./src/generated.rs") {
        return Ok(());
    }

    tonic_build::configure().build_server(false).compile(
        &[
            "googleapis/google/logging/v2/logging.proto",
            "googleapis/google/cloud/speech/v1/cloud_speech.proto",
            "googleapis/google/cloud/texttospeech/v1/cloud_tts.proto",
            "googleapis/google/cloud/tasks/v2beta3/cloudtasks.proto",
        ],
        &["googleapis/"],
    )?;

    place_in_src();

    Ok(())
}

fn place_in_src() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
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
    std::fs::write("./src/generated.rs", result).unwrap();

    let result = Command::new("rustfmt")
        .arg("--emit")
        .arg("files")
        .arg("--edition")
        .arg("2018")
        .arg("./src/generated.rs")
        .output();

    match result {
        Err(e) => {
            eprintln!("error running rustfmt: {:?}", e);
            exit(1)
        }
        Ok(output) => {
            if !output.status.success() {
                let err = String::from_utf8(output.stderr).unwrap();
                panic!(err);
            }
        }
    }

    std::fs::remove_dir_all(out_dir).unwrap();
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
