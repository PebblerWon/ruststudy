use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

fn main2() {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let handle = thread::spawn(move || {
        for i in v.iter() {
            println!("hi number {:?} from the spawned thread!", { i });
            thread::sleep(Duration::from_millis(1));
        }
    });
    for i in 1..5 {
        println!("hi number {i} from the main thread!");
        thread::sleep(Duration::from_millis(1));
    }

    handle.join().unwrap();
    println!("Hello, world!");

    let (tx, rx) = mpsc::channel();
    let tx1 = tx.clone();
    thread::spawn(move || {
        let val = vec![
            String::from("hi"),
            String::from("hello"),
            String::from("toby"),
            String::from("today"),
            String::from("is"),
            String::from("beauty!"),
        ];
        for v in val {
            tx.send(v).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });
    thread::spawn(move || {
        let val = vec![
            String::from("2hi"),
            String::from("2hello"),
            String::from("2toby"),
            String::from("2today"),
            String::from("2is"),
            String::from("2beauty!"),
        ];
        for v in val {
            tx1.send(v).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });
    // let received = rx.recv().unwrap();
    for rec in rx {
        println!("Received: {}", rec);
    }
}

fn main() {
    let m = Mutex::new(5);
    {
        let mut num = m.lock().unwrap();
        *num = 6;
    }
    println!("m = {:?}", m);

    let counter = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Result: {}", *counter.lock().unwrap());
}
