extern crate num_cpus;

use std::thread;
use std::sync::mpsc::{Sender, SendError, channel};

pub struct Consumer<T, R> {
  handle: Box<FnMut(T) -> Option<R> + 'static + Send>,
}

impl<T, R> Consumer<T, R> {
  pub fn new<F>(handle: F) -> Consumer<T, R> where F : FnMut(T) -> Option<R> + 'static + Send {
    Consumer { handle: Box::new(handle) }
  }
}

pub trait Consumable<T, R> {
  fn process(&mut self, job: T) -> Option<R>;
}

impl<T, R> Consumable<T, R> for Consumer<T, R> {
  fn process(&mut self, job: T) -> Option<R> {
    (self.handle)(job)
  }
}

pub struct HelperConsumer<T> {
  job_tx: Sender<T>,
}

impl<T> HelperConsumer<T> where T : 'static + Send {
  pub fn new<C, R>(result_tx: Sender<R>, mut consumer: C) -> HelperConsumer<T> where C : Consumable<T, R> + 'static + Send, R : 'static + Send {
    let (job_tx, job_rx) = channel();

    thread::Builder::new().name("World thread".to_string()).spawn(move || {
      loop {
        let result = consumer.process(job_rx.recv().unwrap());
        if let Some(result) = result {
          result_tx.send(result).unwrap();
        }
      }
    }).unwrap();

    HelperConsumer {
      job_tx: job_tx,
    }
  }

  pub fn send(&mut self, job: T) -> Result<(), SendError<T>> {
    self.job_tx.send(job)
  }
}

pub struct MainConsumer<T, R> {
  consumer: Consumer<T, R>,
  jobs: Vec<T>,
  result_tx: Sender<R>,
}

impl<T, R> MainConsumer<T, R> {
  pub fn new(result_tx: Sender<R>, consumer: Consumer<T, R>) -> MainConsumer<T, R> {
    MainConsumer {
      consumer: consumer,
      jobs: Vec::new(),
      result_tx: result_tx,
    }
  }

  pub fn send(&mut self, job: T) {
    self.jobs.push(job);
  }

  pub fn process_all(&mut self) {
    while self.jobs.len() > 0 {
      let result = self.consumer.process(self.jobs.pop().unwrap());
      if let Some(result) = result {
        self.result_tx.send(result).unwrap();
      }
    }
  }
}

pub struct ConsumerPool<T, R> {
  threads: usize,
  next_thread: usize,
  main: MainConsumer<T, R>,
  helpers: Vec<HelperConsumer<T>>,
}

impl<T, R> ConsumerPool<T, R> where T : 'static + Send, R : 'static + Send {
  pub fn new<F>(result_tx: Sender<R>, gen: F) -> ConsumerPool<T, R> where F : Fn() -> Consumer<T, R> + 'static + Send {
    let threads = num_cpus::get();

    ConsumerPool {
      threads: threads,
      next_thread: 0,
      main: MainConsumer::new(result_tx.clone(), gen()),
      helpers: (0..(threads - 1)).map(|_| HelperConsumer::new(result_tx.clone(), gen())).collect(),
    }
  }

  pub fn send(&mut self, job: T) -> Result<(), SendError<T>> {
    let next_thread = self.next_thread;
    self.next_thread += 1;
    if self.next_thread == self.threads {
      self.next_thread = 0;
    }

    if next_thread == 0 {
      self.main.send(job);
      Ok(())
    }
    else {
      self.helpers[next_thread - 1].send(job)
    }
  }

  pub fn process_main(&mut self) {
    self.main.process_all();
  }
}
