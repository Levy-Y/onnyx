pub struct File {
    name: String,
    size: u8,
    location: String,
}

pub fn read_files<'a>() -> anyhow::Result<Vec<File>> {
    let mut files = vec![];

    // TODO: Implement actual storage reading from SD Card
    files.push(File {
        name: "hello_world".to_string(),
        size: 4,
        location: "/scripts/hello_world.ox".to_string(),
    });
    files.push(File {
        name: "basic_script".to_string(),
        size: 3,
        location: "/scripts/basic_script.ox".to_string(),
    });
    files.push(File {
        name: "advanced_script".to_string(),
        size: 5,
        location: "/scripts/advanced_script.ox".to_string(),
    });
    files.push(File {
        name: "find_files".to_string(),
        size: 2,
        location: "/scripts/find_files.ox".to_string(),
    });

    Ok(files)
}

pub fn get_file_content(file: String) -> anyhow::Result<String> {
    // TODO: Implement actual file reading from SD Card
    Ok(String::from("KEY GUI"))
}