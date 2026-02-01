pub struct File<'a>{
    name: &'a str,
    size: u8,
    location: &'a str,
}

pub fn read_files<'a>() -> anyhow::Result<Vec<File<'a>>> {
    let mut files = vec![];

    // TODO: Implement actual storage file reading from SD Card
    files.push(File{name: "hello_world", size: 4, location: "/scripts/hello_world.ox"});
    files.push(File{name: "basic_script", size: 3, location: "/scripts/basic_script.ox"});
    files.push(File{name: "advanced_script", size: 5, location: "/scripts/advanced_script.ox"});
    files.push(File{name: "find_files", size: 2, location: "/scripts/find_files.ox"});

    Ok(files)
}