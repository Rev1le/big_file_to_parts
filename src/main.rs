#![feature(path_file_prefix)]
#![feature(buf_read_has_data_left)]
#![feature(file_create_new)]

use std::{
    env,
    fs::{File},
    io::{self, BufRead, Read, Write},
    hash::Hash,
    path::{Path, PathBuf}
};

mod decode;
mod encode;

#[derive(Debug)]
pub struct FilePart {
    pub part_file: File,
    pub hash_bytes: Vec<u8>,
    pub part_file_name: String,
}

#[derive(Debug)]
pub struct CompositeFile {
    pub filename: String,
    pub file_extension: String,
    pub file_len: usize,
    pub parts: Vec<FilePart>,
    pub uuid_parts: String,
}

#[derive(Debug)]
struct Config {
    method: char,
    path: PathBuf,
    options: Options
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct Options {
    count_parts: Option<u8>,
    size_parts: Option<usize>
}

impl Config {

    pub fn new(method: char, path: &str) -> Self {
        Config {
            method,
            path: PathBuf::from(path),
            options: Options::default(),
        }
    }

    pub fn new_from_env_args() -> Self {
        let mut args_peekable = env::args().peekable();

        let mut config: Option<Config> = None;

        while let Some(arg) = args_peekable.next() {
            match arg.as_str() {
                "-e" => {
                    if let Some(path) = args_peekable.peek() {
                        config = Some(Config::new('e', path));
                    }
                },
                "-d" => {
                    if let Some(path) = args_peekable.peek() {
                        config = Some(Config::new('d', path));
                    }
                },
                "-n" => {
                    if let Some(size_parts) = args_peekable.peek() {
                        let size_parts = size_parts.parse::<usize>().unwrap();

                        match config.as_mut() {

                            Some(config) =>
                                config.options.size_parts = Some(size_parts),

                            None => {}
                        }
                    }
                }
                _ => continue
            }
        }

        match config {
            Some(config) => {
                return config
            }
            None => {
                println!("Помощь:\n\
                -e path => Для кодирования большого файла\n\
                -d path => Для декодирования файла при помощи .meta файла"
                );
                std::process::exit(1);
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            method: '_',
            path: PathBuf::new(),
            options: Options::default(),
        }
    }
}

//*
//* Format packet:
//* |---hash----|-----data-----|
//* |--[u8;16]--|-----[u8]-----|
//*
//* Format MetaFile:
//* |--file_extension_len--|--file_extension--|--count_hash--|----hash_parts----|
//* |-------usize----------|-------[u8]-------|-----usize----|--Array<[u8;16]>--|
//*

fn main() -> io::Result<()> {

    let config = Config::new_from_env_args();

    match config.method {
       'd' => decode::decode_file(&config.path).unwrap(),
       'e' => encode::encode_file(&config.path, config.options.size_parts).unwrap(),
       _ => panic!("Неподерживаемый аргумент")
    }

    Ok(())
}