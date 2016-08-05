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

pub struct HelperConsumer<T> {
  job_tx: Sender<T>,
}

impl<T> HelperConsumer<T> where T : 'static + Send {
  pub fn new<C, R>(job_tx: Sender<T>, job_rx: Receiver<T>, result_tx: Sender<R>, mut consumer: C) -> HelperConsumer<T> where C : Consumable<T, R> + 'static + Send, R : 'static + Send {
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
    while let Ok(job) = self.job_rx.try_recv() {
      let result = self.consumer.process(job);
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
  pub fn new<F>(result_tx: Sender<R>, gen: F) -> ConsumerPool<T, R> where F : Fn(usize, Producer<T>) -> Consumer<T, R> + 'static + Send {
    let threads = num_cpus::get();

    let (producer, mut rxs) = Producer::new(threads);

    ConsumerPool {
      threads: threads,
      next_thread: 0,
      main: MainConsumer::new(
        rxs.remove(0),
        result_tx.clone(),
        gen(0, producer.clone())),
      helpers: (0..(threads - 1))
      .map(|i| HelperConsumer::new(
        producer.senders[i + 1].clone(),
        rxs.remove(0),
        result_tx.clone(),
        gen(i + 1, producer.clone())))
      .collect(),
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
