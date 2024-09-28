use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn main() -> io::Result<()> {
    let input_file_path = Path::new("src/data/world_10.txt");

    let file = File::open(&input_file_path)?;
    let reader = io::BufReader::new(file);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("coordinates.rs");

    let mut output = String::from("pub const COORDINATES: [(f64, f64); 410665] = [\n");
    for line in reader.lines() {
        let line = line?;
        if line.is_empty(){
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (lat, lon) = (parts[0].parse::<f64>().unwrap(), parts[1].parse::<f64>().unwrap());
        output.push_str(&format!("    ({:.10}, {:.10}),\n", lat, lon))
    }

    output.push_str("];\n");
    std::fs::write(&dest_path, output)?;
    Ok(())
}
