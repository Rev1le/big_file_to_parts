use std::{fs, io, path};
use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use crate::{CompositeFile, FilePart};

struct MetaFile {

}

#[derive(Debug)]
struct HashPart([u8;16]);

impl HashPart {
    fn read(src: &mut impl Read) -> Self {
        let mut bytes = [0_u8;16];
        src.read(&mut bytes).unwrap();
        HashPart(bytes)
    }
}

pub fn decode_file(path: &PathBuf) -> io::Result<()> {


    if !path.is_file() {
        panic!("Предоставьте путь к файлу сборки")
    }

    let mut dir = dbg!(path.parent().unwrap().to_path_buf());
    dir.push("");
    let mut f = fs::File::open(&path)?;

    let filename =  dbg!(
            path
                .file_prefix()
                .unwrap()
                .to_string_lossy()
                .parse::<String>()
                .unwrap()
                .split('_')
                .last()
                .unwrap()
                .to_owned()
    );
    //let b = filename.split('_').last().unwrap();
    let file_extension = dbg!(decode_input_file_extension(&mut f));

    let mut count_parts_bytes = [0;8];
    f.read(&mut count_parts_bytes)?;

    let count_parts = <usize>::from_be_bytes(count_parts_bytes);

    let mut parts_hash = vec![];
    for _ in 0..count_parts {
        parts_hash.push(HashPart::read(&mut f));
    }

    //println!("{:?}", parts_hash);

    let mut output_file = File::create(format!("{}.{}", filename, file_extension)).unwrap();

    let mut tmp_has = [0_u8; 16];


    for part_hash in parts_hash {
        let mut dir_entres = dir.read_dir().unwrap();

        dir_entres.find(|entry| {
            if let Ok(mut f_part) = fs::File::open(dbg!(entry.as_ref().unwrap().path())) {
                f_part.read(&mut tmp_has).unwrap();

                if part_hash.0 == tmp_has {
                    let mut part_data = vec![];
                    f_part.read_to_end(&mut part_data).unwrap();
                    output_file.write(&part_data).unwrap();
                    return true

                }
            }
            return false
        });
    }

    Ok(())
}

fn decode_input_file_extension(src: &mut impl Read) -> String {
    let mut extension_bytes_len = [0_u8;8];
    src.read( &mut extension_bytes_len).unwrap();
    let mut extension_len =<usize>::from_be_bytes(extension_bytes_len);

    let mut extension_bytes = Vec::with_capacity(extension_len);
    unsafe {
        extension_bytes.set_len(extension_len);
    }

    src.read( &mut extension_bytes).unwrap();

    String::from_utf8(extension_bytes).unwrap()
}

fn decode_metafile() -> MetaFile {
    MetaFile {

    }
}

fn decode_part() -> FilePart {
    FilePart {
        part_file: fs::File::create("/").unwrap(),
        hash_bytes: vec![],
        part_file_name: "".to_string(),
    }
}