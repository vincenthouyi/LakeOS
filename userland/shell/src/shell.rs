use alloc::string::String;
use alloc::vec::Vec;

use naive::fs::{current_dir, read_dir, set_current_dir, File};
use naive::io::AsyncReadExt;
use naive::os_str::OsStr;
use naive::path::Path;

use futures_util::stream::StreamExt;

use crate::naive::os_str::OsStrExt;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
}

/// A structure representing a single shell command.
#[derive(Debug)]
struct Command<'a> {
    args: Vec<&'a str>,
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

    async fn cat(&self, path: &Path) -> Result<(), ()> {
        let mut file = File::open(path).await?;
        let mut buf = Vec::new();

        file.read_to_end(&mut buf).await.map_err(|_| ())?;
        print!("{}", String::from_utf8(buf).unwrap()).await;
        Ok(())
    }

    async fn cd<P: AsRef<Path>>(&self, pwd: P) -> Result<(), ()> {
        set_current_dir(pwd).await
    }

    async fn ls(&self, args: &[&str]) -> Result<(), ()> {
        let path = args
            .get(0)
            .map(|p| OsStr::from_bytes(p.as_bytes()).into())
            .unwrap_or_else(|| current_dir().unwrap());
        match read_dir(&path).await {
            Ok(dir) => {
                for entry in dir {
                    println!("{:?}", entry.unwrap().path()).await;
                }
            }
            Err(e) => {
                println!("{}: {:?}", path.to_str().unwrap(), e).await;
            }
        }
        Ok(())
    }

    pub async fn sleep(&self, sleep_second: u64) -> Result<(), ()> {
        naive::time::sleep_ms(sleep_second * 1000).await;
        Ok(())
    }

    pub async fn exec(&self) {
        match self.args.as_slice() {
            ["echo", args @ ..] => {
                println!("{}", args.join(" ")).await;
            }
            ["ls", args @ ..] => {
                self.ls(args).await.unwrap();
            }
            ["pwd"] => println!("{}", current_dir().unwrap().to_str().unwrap()).await,
            ["cd", args @ ..] => {
                let path = args.get(0).unwrap_or(&"/");
                let path = OsStr::from_bytes(path.as_bytes());
                self.cd(path).await.unwrap();
            }
            ["cat", args @ ..] => {
                for path in args {
                    let path = OsStr::from_bytes(path.as_bytes()).as_ref();
                    let res = self.cat(path).await;
                    if let Err(e) = res {
                        println!("Error {:?}", e).await;
                    }
                }
            }
            ["sleep", sec] => {
                if let Ok(s) = sec.parse() {
                    self.sleep(s).await.unwrap();
                }
            }
            [] => { /* Ignore empty command */ }
            cmd => {
                println!("unknown command: {:?}", cmd).await;
            }
        };
    }
}

const BS: u8 = 0x08;
const BEL: u8 = 0x07;
const LF: u8 = 0x0A;
const CR: u8 = 0x0D;
const DEL: u8 = 0x7F;
async fn read_line() -> String {
    let mut read: usize = 0;

    let mut cmd = Vec::new();
    let mut stdin = naive::io::stdin().await;
    'outer: loop {
        while let Some(b) = stdin.next().await {
            match b {
                BS | DEL if read > 0 => {
                    print!("{}", BS as char).await;
                    print!(" ").await;
                    print!("{}", BS as char).await;
                    cmd.pop();
                    read -= 1;
                }
                LF | CR => {
                    println!("").await;
                    break 'outer;
                }
                byte @ b' '..=b'~' => {
                    print!("{}", byte as char).await;
                    cmd.push(byte);
                    read += 1;
                }
                _ => {
                    print!("{}", BEL as char).await;
                }
            }
        }
    }
    String::from_utf8(cmd).unwrap()
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub async fn shell(prefix: &str) {
    loop {
        print!("{} ", prefix).await;
        match Command::parse(&read_line().await) {
            //TODO exit
            Ok(cmd) => cmd.exec().await,
            Err(Error::Empty) => {}
        }
    }
}
