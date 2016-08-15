extern crate num_cpus;

use std::thread;
use std::sync::mpsc::{Sender, Receiver, SendError, channel};

pub struct Producer<T> {
  next: usize,
  senders: Vec<Sender<T>>,
}

impl<T> Producer<T> {
  fn new(threads: usize) -> (Producer<T>, Vec<Receiver<T>>) {
    let mut txs = Vec::new();
    let mut rxs = Vec::new();

    for _ in 0..threads {
      let (tx, rx) = channel();
      txs.push(tx);
      rxs.push(rx);
    }

    (
      Producer {
        next: 0,
        senders: txs,
      },
      rxs
    )
  }

  pub fn send(&mut self, job: T) -> Result<(), SendError<T>> {
    let next = self.next;
    self.next = if next == self.senders.len() - 1 {0} else {next + 1};
    self.senders[next].send(job)
  }

  pub fn send_main(&mut self, job: T) -> Result<(), SendError<T>> {
    self.senders[0].send(job)
  }
}

impl<T> Clone for Producer<T> {
  fn clone(&self) -> Producer<T> {
    Producer {
      next: self.next,
      senders: self.senders.clone(),
    }
  }
}

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

pub struct HelperConsumer {
  shutdown_tx: Sender<()>,
}

impl HelperConsumer {
  pub fn new<C, T, R>(job_rx: Receiver<T>, result_tx: Sender<R>, mut consumer: C) -> HelperConsumer where C : Consumable<T, R> + 'static + Send, T : 'static + Send, R : 'static + Send {
    let (shutdown_tx, shutdown_rx) = channel();

    thread::Builder::new().name("World thread".to_string()).spawn(move || {
      while let Err(_) = shutdown_rx.try_recv() {
        if let Ok(job) = job_rx.recv() {
          if let Some(result) = consumer.process(job) {
            result_tx.send(result).unwrap();
          }
        }
        else {
          break;
        }
      }
      println!("shutdown");
    }).unwrap();

    HelperConsumer {
      shutdown_tx: shutdown_tx,
    }
  }
}

impl HelperConsumer {
  fn shutdown(&mut self) {
    self.shutdown_tx.send(()).unwrap();
  }
}

pub struct MainConsumer<T, R> {
  consumer: Consumer<T, R>,
  jobs: Vec<T>,
  job_rx: Receiver<T>,
  result_tx: Sender<R>,
}

impl<T, R> MainConsumer<T, R> {
  pub fn new(job_rx: Receiver<T>, result_tx: Sender<R>, consumer: Consumer<T, R>) -> MainConsumer<T, R> {
    MainConsumer {
      consumer: consumer,
      jobs: Vec::new(),
      job_rx: job_rx,
      result_tx: result_tx,
    }
  }

  pub fn process_all(&mut self) {
    while self.jobs.len() > 0 {
      let result = self.consumer.process(self.jobs.pop().unwrap());
      if let Some(result) = result {
        self.result_tx.send(result).unwrap();
      }
    }
    while let Ok(job) = self.job_rx.try_recv() {
      let result = self.consumer.process(job);
      if let Some(result) = result {
        self.result_tx.send(result).unwrap();
      }
    }
  }

  pub fn process_until<F>(&mut self, mut until: F) where F : FnMut(&T) -> bool {
    while let Ok(job) = self.job_rx.recv() {
      if until(&job) {
        let result = self.consumer.process(job);
        if let Some(result) = result {
          self.result_tx.send(result).unwrap();
        }
      }
      else {
        break;
      }
    }
  }
}

#[allow(dead_code)]
pub struct ConsumerPool<T, R> {
  threads: usize,
  producer: Producer<T>,
  main: MainConsumer<T, R>,
  helpers: Vec<HelperConsumer>,
}

impl<T, R> ConsumerPool<T, R> where T : 'static + Send, R : 'static + Send {
  pub fn new<F>(result_tx: Sender<R>, gen: F) -> ConsumerPool<T, R> where F : Fn(usize, Producer<T>) -> Consumer<T, R> + 'static + Send {
    let threads = num_cpus::get();
    // let threads = 1;

    let (producer, mut rxs) = Producer::new(threads);

    ConsumerPool {
      threads: threads,
      producer: producer.clone(),
      main: MainConsumer::new(
        rxs.remove(0),
        result_tx.clone(),
        gen(0, producer.clone())),
      helpers: (0..(threads - 1))
      .map(|i| HelperConsumer::new(
        rxs.remove(0),
        result_tx.clone(),
        gen(i + 1, producer.clone())))
      .collect(),
    }
  }

  pub fn send(&mut self, job: T) -> Result<(), SendError<T>> {
    self.producer.send(job)
  }

  pub fn process_main(&mut self) {
    self.main.process_all();
  }

  pub fn process_until<F>(&mut self, until: F) where F : FnMut(&T) -> bool {
    self.main.process_until(until);
  }

  pub fn shutdown(&mut self) {
    for helper in self.helpers.iter_mut() {
      helper.shutdown();
    }
  }
}
