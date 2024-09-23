use std::io::{BufReader, Read, Write};
use std::sync::Arc;
use std::thread;
use anyhow::Error;
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use tokio::sync::Mutex;
use crate::shell_list::SystemShell;

pub struct PtyConn {
   pub pty: Arc<Mutex<PtyPair>>,
   pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
   pub reader: Arc<Mutex<BufReader<Box<dyn Read + Send>>>>,
}

impl PtyConn {
  pub fn open(shell: SystemShell)-> Result<PtyConn, Error> {
      let pty_system = native_pty_system();

      let pty_pair = pty_system
          .openpty(PtySize {
              rows: 24,
              cols: 80,
              pixel_width: 0,
              pixel_height: 0,
          })
          .unwrap();
      let mut cmd = CommandBuilder::new(shell.command);
      cmd.cwd(shell.cwd);
      for item in shell.env {
          if let Some((key, value)) = item.split_once('=') {
              // 处理找到 '=' 的情况
              println!("键: {}, 值: {}", key, value);
              cmd.env(key,value);
          }
      }

      let child = pty_pair
          .slave
          .spawn_command(cmd)
          .map_err(|err| err.to_string());

      thread::spawn(move || {
          let status = child.expect("REASON").wait().unwrap();
          println!("{}",status);
      });
      let reader = pty_pair.master.try_clone_reader().unwrap();
      let writer = pty_pair.master.take_writer().unwrap();
      Ok(PtyConn{
          pty: Arc::new(Mutex::new(pty_pair)),
          writer: Arc::new(Mutex::new(writer)),
          reader: Arc::new(Mutex::new(BufReader::new(reader)))
      })
  }
}