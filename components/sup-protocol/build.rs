use prost_build;

/// Automatically generate Rust code from our protobuf definitions at
/// compile time.
///
/// Generated code is deposited in `OUT_DIR` and automatically
/// `include!`-ed in our Rust modules, per standard Prost practice.
use tempfile;

use std::{collections::HashMap,
          fs,
          io::{Error,
               ErrorKind,
               Read,
               Result,
               Write},
          path::{Path,
                 PathBuf},
          process::Command};

use heck::SnakeCase;
use prost::Message;
use prost_build::{protoc,
                  protoc_include};
use prost_types::{DescriptorProto,
                  FileDescriptorProto,
                  FileDescriptorSet};

type Module = Vec<String>;

fn main() {
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(Serialize, Deserialize)]");
    config.type_attribute(".", "#[serde(rename_all = \"kebab-case\")]");
    config.compile_protos(&["protocols/ctl.proto",
                            "protocols/net.proto",
                            "protocols/types.proto"],
                          &["protocols/"])
          .expect("Couldn't compile protobufs!");
    compile_proto_impls(&["protocols/ctl.proto",
                          "protocols/net.proto",
                          "protocols/types.proto"],
                        &["protocols/"]).expect("Couldn't generate code!");
}

fn compile_proto_impls<P>(protos: &[P], includes: &[P]) -> Result<()>
    where P: AsRef<Path>
{
    let target = PathBuf::from("src/generated");
    let tmp = tempfile::TempDir::new()?;
    let descriptor_set = tmp.path().join("prost-descriptor-set");

    let mut cmd = Command::new(protoc());
    cmd.arg("--include_imports")
       .arg("--include_source_info")
       .arg("-o")
       .arg(&descriptor_set);

    for include in includes {
        cmd.arg("-I").arg(include.as_ref());
    }

    // Set the protoc include after the user includes in case the user wants to
    // override one of the built-in .protos.
    cmd.arg("-I").arg(protoc_include());

    for proto in protos {
        cmd.arg(proto.as_ref());
    }

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(Error::new(ErrorKind::Other,
                              format!("protoc failed: {}",
                                      String::from_utf8_lossy(&output.stderr))));
    }

    let mut buf = Vec::new();
    fs::File::open(descriptor_set)?.read_to_end(&mut buf)?;
    let descriptor_set = FileDescriptorSet::decode(buf.as_slice())?;

    let modules = generate(descriptor_set.file);
    for (module, content) in modules {
        let mut filename = module.join(".");
        filename.push_str(".impl.rs");
        let mut file = fs::File::create(target.join(filename))?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
    }

    Ok(())
}

fn generate(files: Vec<FileDescriptorProto>) -> HashMap<Module, String> {
    let mut modules = HashMap::new();
    for file in files {
        let module = module(&file);
        let mut buf = modules.entry(module).or_insert_with(String::new);
        buf.push_str("use crate::message;\n\n");
        for msg in file.message_type.iter() {
            add_message(msg, &mut buf);
        }
    }
    modules
}

pub fn module(file: &FileDescriptorProto) -> Module {
    file.package()
        .split('.')
        .filter(|s| !s.is_empty())
        .map(to_snake)
        .collect()
}

fn add_message(msg: &DescriptorProto, buf: &mut String) {
    if msg.name() != "ApplicationEnvironment" {
        buf.push_str(&format!("impl message::MessageStatic for {} {{\n", msg.name()));
        buf.push_str(&format!("    const MESSAGE_ID: &'static str = \"{}\";\n", msg.name()));
        buf.push_str("}\n");
    }
}

/// Converts a `camelCase` or `SCREAMING_SNAKE_CASE` identifier to a `lower_snake` case Rust field
/// identifier.
pub fn to_snake(s: &str) -> String {
    let mut ident = s.to_snake_case();

    // Add a trailing underscore if the identifier matches a Rust keyword
    // (https://doc.rust-lang.org/grammar.html#keywords).
    match &ident[..] {
        "abstract" | "alignof" | "as" | "become" | "box" | "break" | "const" | "continue"
        | "crate" | "do" | "else" | "enum" | "extern" | "false" | "final" | "fn" | "for" | "if"
        | "impl" | "in" | "let" | "loop" | "macro" | "match" | "mod" | "move" | "mut"
        | "offsetof" | "override" | "priv" | "proc" | "pub" | "pure" | "ref" | "return"
        | "self" | "sizeof" | "static" | "struct" | "super" | "trait" | "true" | "type"
        | "typeof" | "unsafe" | "unsized" | "use" | "virtual" | "where" | "while" | "yield" => {
            ident.push('_');
        }
        _ => (),
    }
    ident
}
