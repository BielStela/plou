use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn main() -> io::Result<()> {
    println!("cargo::rerun-if-env-changed=WORLD_SRC");
    let path = env::var("WORLD_SRC").unwrap_or_else(|_| "./data/world_10.txt".to_string());
    let input_file_path = Path::new(path.as_str());
    let file = File::open(&input_file_path).expect(format!("File not found").as_str());
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .filter(|l| !l.as_ref().unwrap().is_empty())
        .collect::<Result<_, _>>()
        .unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("coordinates.rs");

    let mut output = String::from(format!(
        "pub const COORDINATES: [(f64, f64); {}] = [\n",
        lines.len()
    ));
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (lat, lon) = (
            parts[0].parse::<f64>().unwrap(),
            parts[1].parse::<f64>().unwrap(),
        );
        output.push_str(&format!("    ({:.10}, {:.10}),\n", lat, lon))
    }

    output.push_str("];\n");
    std::fs::write(&dest_path, output)?;
    Ok(())
}
