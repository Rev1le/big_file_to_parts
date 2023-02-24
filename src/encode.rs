use std::{fs, io};
use std::fs::File;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use uuid::Uuid;
use super::FilePart;
use super::CompositeFile;


pub fn encode_file(path: &PathBuf, size_part_opt: Option<usize>) -> io::Result<()> {

    let size_part = if let Some(size_parts) = size_part_opt {
        size_parts
    } else {
        1_073_741_824
    };

    let f = File::open(dbg!(&path))?;
    let mut f  = io::BufReader::with_capacity(size_part, f);

    let mut com_file = CompositeFile {
        filename: path.file_prefix().unwrap().to_os_string().into_string().unwrap(),
        file_extension:  path.extension().unwrap().to_os_string().into_string().unwrap(),
        parts: vec![],
    };

    let mut number_part = 1;
    while f.has_data_left().unwrap() {
        let buffer_bytes = f.fill_buf()?;
        let buffer_bytes_len = buffer_bytes.len();
        let part = encode_part(number_part, buffer_bytes)?;
        com_file.parts.push(part);

        number_part += 1;
        f.consume(buffer_bytes_len);
    }

    encode_metafile(&com_file);

    Ok(())
}

fn encode_part(part_number: u8, data: &[u8]) -> io::Result<FilePart> {

    let part_uuid = Uuid::new_v4();
    let part_file_name = format!("{}_{}.part", part_uuid, part_number);
    let hash_bytes = md5::compute(&part_file_name).0.to_vec();

    let part_bytes = vec![hash_bytes.as_slice(), data]
        .into_iter()
        .flatten()
        .copied()
        .collect::<Vec<u8>>();

    let mut part_file = File::create_new(&part_file_name)?;
    part_file.write(&part_bytes)?;

    println!("Файл с частью данными был создан => {}", &part_file_name);

    Ok(FilePart {
        part_file,
        hash_bytes,
        part_file_name,
    })
}

fn encode_metafile(composite_file: &CompositeFile) -> io::Result<()> {
    let mut f = fs::File::create(format!("build_file_{}.meta", composite_file.filename))?;
    let file_format_bytes =  composite_file.file_extension.as_bytes();

    f.write(&file_format_bytes.len().to_be_bytes())?;
    f.write(file_format_bytes)?;
    f.write(&composite_file.parts.len().to_be_bytes())?;
    composite_file.parts.iter().map(|x| f.write(&x.hash_bytes).unwrap()).collect::<Vec<_>>();

    Ok(())

}