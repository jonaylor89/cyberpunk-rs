use std::fs;
use std::path::Path;
use std::process::Command;

#[tracing::instrument]
pub fn concat(file1: &str, file2: &str) -> String {
    let input_path = Path::new("./testdata/input.txt");
    let mergelist = format!("file '{}'\nfile '{}'\n", file1, file2);
    fs::write(input_path, mergelist).expect("Failed to write merge list");
    let mut output = Command::new("ffmpeg")
        .args([
            "-loglevel",
            "error",
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            "testdata/input.txt",
            "-c",
            "copy",
            "./testdata/concat.mp3",
        ])
        .spawn()
        .expect("concat command failed to run");

    let status = output.wait();
    tracing::info!("concat transformation completed: {}", status.unwrap());

    return "concat.mp3".into()
}
