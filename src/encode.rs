use std::{
    fs::{self, File},
    io::{self, BufRead, Write, BufReader},
    path::PathBuf
};
use std::ffi::OsString;
use std::io::Error;
use std::process::ExitCode;
use uuid::Uuid;

use super::FilePart;
use super::CompositeFile;

#[derive(Debug)]
pub enum EncodeErrors {
    IOError(::std::io::Error),
    OsStringError(std::ffi::OsString),
    PathParseError,
}

impl From<std::io::Error> for EncodeErrors {
    fn from(value: std::io::Error) -> Self {
        EncodeErrors::IOError(value)
    }
}

impl From<std::ffi::OsString> for EncodeErrors {
    fn from(value: std::ffi::OsString) -> Self {
        EncodeErrors::OsStringError(value)
    }
}

pub fn encode_file(path: &PathBuf, size_part_opt: Option<usize>) -> Result<(), EncodeErrors> {

    if !path.is_file() {
        panic!("Предоставьте путь к файлу")
    }

    let mut size_part = size_part_opt.unwrap_or(1_073_741_824_usize);
    let mut f  = BufReader::with_capacity(
        size_part,
        File::open(&path)?
    );

    let mut com_file = CompositeFile {
        filename: path
            .file_stem()
            .ok_or(EncodeErrors::PathParseError)?
            .to_os_string()
            .into_string()?,
        file_extension:  path
            .extension()
            .ok_or(EncodeErrors::PathParseError)?
            .to_os_string()
            .into_string()?,
        file_len: f.capacity(),
        parts: vec![],
        uuid_parts: Uuid::new_v4().to_string(),
    };

    let mut number_part = 1;

    while f.has_data_left()? {

        let buffer_bytes = f.fill_buf()?;
        let buffer_bytes_len = buffer_bytes.len();

        let part = encode_part(
            &com_file.uuid_parts,
            number_part,
            buffer_bytes
        )?;

        com_file.parts.push(part);

        number_part += 1;
        f.consume(buffer_bytes_len);
    }

    encode_metafile(&com_file)?;

    Ok(())
}

fn encode_part(part_uuid: &str, part_number: u8, data: &[u8]) -> io::Result<FilePart> {

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

    let mut metafile = File::create(
        format!("build_file_{}.meta", composite_file.filename)
    )?;
    let source_filename_bytes = composite_file.filename.as_bytes();
    let source_format_bytes =  composite_file.file_extension.as_bytes();
    let parts_uuid_bytes = composite_file.uuid_parts.as_bytes();

    // Запись имени исходного файла
    metafile.write(
        &(source_filename_bytes.len() as u8).to_be_bytes()
    )?;
    metafile.write(
        source_filename_bytes
    )?;

    // Запись расширения исходного файла
    metafile.write(
        &(source_format_bytes.len() as u8).to_be_bytes()
    )?;
    metafile.write(
        source_format_bytes
    )?;

    // Запись uuid в названии частей.
    metafile.write(
        &(parts_uuid_bytes.len() as u8).to_be_bytes()
    )?;
    metafile.write(
        parts_uuid_bytes
    )?;

    // Запись всех хешей частей как массив
    metafile.write(&composite_file.parts.len().to_be_bytes())?;
    composite_file.parts
        .iter()
        .map(
            |x| metafile.write(&x.hash_bytes).unwrap()
        )
        .for_each(drop);

    Ok(())
}