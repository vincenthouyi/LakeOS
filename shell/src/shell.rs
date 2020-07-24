use std::vec::Vec;
// use crate::prelude::*;
//use std::path::PathBuf;
//use super::FILE_SYSTEM;
//use fat32::traits::{FileSystem, Dir, Entry};
//use std::io;
//use std::str;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
}

/// A structure representing a single shell command.
#[derive(Debug)]
struct Command<'a> {
    args: Vec<&'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str) -> Result<Command<'a>, Error> {
        let args: Vec<&str> = s.split(' ').filter(|x| !x.is_empty()).collect();

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

//    /// Returns this command's path. This is equivalent to the first argument.
//    fn path(&self) -> &str {
//        self.args[0]
//    }
//
//    fn cat(&self, pwd: &mut PathBuf) -> Result<(), ()> {
//        use std::io::{Read};
//        let dir_buf = PathBuf::from(self.args[1]);
//        let abs_path = {
//            if dir_buf.is_absolute() {
//                dir_buf 
//            } else {
//                let mut tmp_pwd = pwd.clone();
//                tmp_pwd.push(dir_buf);
//                tmp_pwd
//            }
//        };
//
//        FILE_SYSTEM.open(abs_path)
//            .and_then(|dir_entry| 
//                      dir_entry.into_file().ok_or(io::Error::new(io::ErrorKind::Other, 
//                                                                "Is a directory")))
//            .and_then(|mut file| {
//                let mut buf = Vec::new();
//                match file.read_to_end(&mut buf) {
//                    Ok(n) => {
//                        let s = str::from_utf8(&buf[..n]).unwrap();
//                        println!("{}", s);
//                        Ok(())
//                    }
//                    Err(e) => {
//                        Err(e)
//                    }
//                }
//            })
//            .map_err(|e| {
//                match e.kind() {
//                    io::ErrorKind::Other => println!("Is a directory"),
//                    _ => {}
//                }
//                ()
//            })
//    }
//
//    fn cd(&self, pwd: &mut PathBuf) -> Result<(), ()> {
//        let dir_buf = PathBuf::from(if self.args.len() < 1 {
//            "/" 
//        } else {
//            self.args[1]
//        });
//        
//        let abs_path = {
//            if dir_buf.is_absolute() {
//                dir_buf 
//            } else {
//                let mut tmp_pwd = pwd.to_path_buf();
//                tmp_pwd.push(dir_buf);
//                tmp_pwd
//            }
//        };
//        
//        let dir_entry = FILE_SYSTEM.open(abs_path.clone());
//        if dir_entry.is_err() {
//            println!("cd: no such file or directory: {}", self.args[1]);
//            return Err(());
//        }
//
//        if dir_entry.unwrap().as_dir().is_some() {
//            pwd.set_file_name(abs_path);
//            return Ok(());
//        } else {
//            println!("cd: not a directory: {}", self.args[1]);
//            return Err(());
//        }
//    }
//
//    fn ls(&self, pwd: &mut PathBuf) -> Result<(),()> {
//        use fat32::traits::Metadata;
//
//        let mut args = &self.args[1..];
//        let all = args.get(0).and_then(|&arg| {
//            if arg == "-a" {
//                args = &args[1..];
//                Some(true)
//            } else {
//                None
//            }
//        }).unwrap_or(false);
//
//        let dir_buf = if args.len() < 1 {
//            pwd.clone()
//        } else {
//            PathBuf::from(args[0])
//        };
//        
//        let abs_path = {
//            if dir_buf.is_absolute() {
//                dir_buf 
//            } else {
//                let mut tmp_pwd = pwd.clone();
//                tmp_pwd.push(dir_buf);
//                tmp_pwd
//            }
//        };
//    
//        FILE_SYSTEM.open(abs_path.clone())
//            .and_then(|dir_entry| 
//                      dir_entry.into_dir().ok_or(io::Error::new(io::ErrorKind::Other, 
//                                                                "not dir")))
//            .and_then(|dir| dir.entries())
//            .and_then(|entries| {
//                for e in entries {
//                    if all || !e.metadata().hidden() {
//                        println!("{}", e.name());
//                    }
//                }
//                Ok(())
//            })
//            .map_err(|e| {
//                match e.kind() {
//                    io::ErrorKind::Other => println!("ls: not supported {}", args[0]),
//                    io::ErrorKind::NotFound => println!("ls: no such file or directory: {}", args[0]),
//                    _ => {},
//                }
//                ()
//            })
//    }
//
//    pub fn exec(&self, pwd: &mut PathBuf) {
//        match self.path() {
//            "echo" => {
//                let len = self.args.len();
//                for i in 1..(len - 1) {
//                    print!("{} ", self.args.as_slice()[i]);
//                }
//                println!("{}", self.args.as_slice()[len - 1]);
//                Ok(())
//            }
//            "ls" => self.ls(pwd),
//            "pwd" => { println!("{}", pwd.to_str().unwrap()); Ok(()) }
//            "cd" => self.cd(pwd),
//            "cat" => self.cat(pwd),
//            cmd => {
//                println!("unknown command: {}", cmd);
//                Ok(())
//            }
//        
//        }.unwrap_or(());
//    }
}

const BS: u8 = 0x08;
const BEL: u8 = 0x07;
const LF: u8 = 0x0A;
const CR: u8 = 0x0D;
const DEL: u8 = 0x7F;
use std::string::String;
fn read_line() -> String {
    use std::io::Read;
    let mut read = 0;

    let mut cmd = std::vec::Vec::new();
    'outer: loop {
        for b in std::io::stdin().bytes() {
            match b.unwrap() {
                BS | DEL if read > 0 => {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    read -= 1;
                }
                LF | CR => {
                    println!("");
                    break 'outer;
                }
                byte @ b' ' ..= b'~' => {
                    print!("{}", byte as char);
                    cmd.push(byte);
                    read += 1;
                }
                _ => {
                    print!("{}", BEL as char);
                }
            }
        }
    }
    String::from_utf8(cmd).unwrap()
}

const MAXBUF: usize = 512;
const MAXARGS: usize = 64;
/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) {
//    let mut pwd = PathBuf::from("/");

    loop {
//        print!("{} {}", pwd.to_str().unwrap(), prefix);
        print!("{} ", prefix);
        match Command::parse(&read_line()) {
            //TODO exit
//            Ok(cmd) => cmd.exec(&mut pwd),
            Ok(cmd) => { println!("command: {:?}", cmd) },
            Err(Error::Empty) => { }
        }
    }
}
