#![allow(dead_code, unused)]
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};

use tokio::{
    sync::{broadcast, mpsc, watch},
    task::JoinSet,
    time::{self, Instant, Sleep},
};

#[tokio::main]
async fn main() {
    // timer sender and receivers
    let (timer_add_tx, mut timer_add_rx) = mpsc::channel::<(String, Duration)>(10);
    let mut timer_tracker = TimerTracker::new(timer_add_rx);

    tokio::spawn(async move { timer_tracker.start().await });

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

    time::sleep(Duration::from_secs(2)).await;
    timer_add_tx
        .send(("Bar".to_string(), Duration::from_secs(10)))
        .await
        .unwrap();

    // TODO: This is not adding it, so apparently Timer is not dropped
    timer_add_tx
        .send(("Foo".to_string(), Duration::from_secs(1)))
        .await
        .unwrap();

    time::sleep(Duration::from_secs(30)).await
}

struct TimerTracker {
    timers: HashMap<String, broadcast::Sender<String>>,
    timer_rx: mpsc::Receiver<(String, Duration)>,
}

impl TimerTracker {
    fn new(timer_rx: mpsc::Receiver<(String, Duration)>) -> Self {
        TimerTracker {
            timers: HashMap::new(),
            timer_rx,
        }
    }

    async fn start(mut self) {
        loop {
            let (name, duration) = self.timer_rx.recv().await.unwrap();
            // Check if there is an entry in the hashmap already, if not create one.
            if !self.timers.contains_key(&name) {
                println!("adding timer {}", name);
                self.add_timer(name, duration); // new timer and sender
            } else {
                println!("Restart timer {}", name);
                // restart the timer or re-add it if it was removed
                match self.timers.get(&name).unwrap().send("restart".to_string()) {
                    Ok(_) => (),
                    Err(_) => {
                        self.add_timer(name, duration);
                    }
                };
            };
        }
    }

    fn add_timer(&mut self, name: String, duration: Duration) {
        let (restart_tx, _) = broadcast::channel::<String>(1);
        let timer = Timer::new(name.clone(), duration, restart_tx.clone());

        tokio::spawn(timer.start());
        self.timers.insert(name, restart_tx);
    }
}

struct Timer {
    name: String,
    duration: Duration,
    sender: broadcast::Sender<String>,
}

impl Timer {
    pub fn new(name: String, duration: Duration, sender: broadcast::Sender<String>) -> Self {
        Timer {
            duration,
            name,
            sender,
        }
    }

    pub async fn start(mut self) {
        loop {
            let dur = self.duration;
            let mut receiver = self.sender.subscribe();
            let mut timeout = JoinSet::new();

            timeout.spawn(async move {
                time::sleep(dur).await;
                true
            });
            timeout.spawn(async move {
                receiver.recv().await;
                false
            });

            if timeout.join_next().await.unwrap().unwrap() {
                println!("{} timer timed out!", self.name);
                return;
            } else {
                println!("{} timer restarted!", self.name)
            }
        }
    }
}
