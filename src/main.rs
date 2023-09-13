#![allow(dead_code, unused)]
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};

use tokio::{
    sync::{mpsc, watch},
    task::JoinSet,
    time::{self, Instant, Sleep},
};

#[tokio::main]
async fn main() {
    // timer sender and receivers
    let (timer_add_tx, mut timer_add_rx) = mpsc::channel::<(String, Duration)>(10);
    //let (timer_timeout_tx, timer_timeout_rx) = watch::channel("".to_string());

    let timers = tokio::spawn(async move {
        let mut timers = JoinSet::new();

        loop {
            let (name, duration) = timer_add_rx.recv().await.unwrap();
            let mut timer = Timer::new(name, duration);
            timers.spawn(timer.start());
        }
    });

    timer_add_tx
        .send(("Foo".to_string(), Duration::from_secs(1)))
        .await
        .unwrap();
    timer_add_tx
        .send(("Bar".to_string(), Duration::from_secs(10)))
        .await
        .unwrap();
    timer_add_tx
        .send(("Baz".to_string(), Duration::from_secs(2)))
        .await
        .unwrap();

    timers.await.unwrap();
}

struct TimerTracker {
    timers: HashMap<String, Timer>,
    reset_rx: mpsc::Receiver<String>,
}

struct Timer {
    name: String,
    duration: Duration,
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
    timer: Sleep,
}

impl Timer {
    pub fn new(name: String, duration: Duration) -> Self {
        let (sender, receiver) = mpsc::channel(1);
        Timer {
            duration,
            name,
            sender,
            receiver,
            timer: time::sleep(duration),
        }
    }

    pub async fn start(self) {
        let sender = self.sender.clone();
        self.timer.await;
        println!("{} done sleeping!", self.name);
        sender.send(self.name).await;
    }

    // pub async fn reset(self) {
    //     self.timer.reset();
    // }
}

struct FutureTimer {
    name: String,
    duration: Duration,
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
    timer: Sleep,
}

impl Future for FutureTimer {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}
