use std::time::Duration;

use trpl::{Either, Html};

async fn page_title(url: &str) -> (&str, Option<String>) {
    let response = trpl::get(url).await;

    let response_txt = response.text().await;

    let title = Html::parse(&response_txt)
        .select_first("title")
        .map(|el| el.inner_html());
    (url, title)
}

fn main() {
    trpl::run(async {
        // trpl::spawn_task(async {

        // });
        let handle = async {
            for i in 1..10 {
                println!("hi number {i} from the first task!");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let handle2 = async {
            for i in 1..5 {
                println!("hi number {i} from the second task!");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        trpl::join(handle, handle2).await;
    })
}
fn main2() {
    println!("Hello, world!");
    let args: Vec<String> = std::env::args().collect();
    trpl::run(async {
        let url1 = &args[1];
        let url2 = &args[2];
        let title1 = page_title(url1);
        let title2 = page_title(url2);

        let (url, maybe_title) = match trpl::race(title1, title2).await {
            Either::Left(left) => left,
            Either::Right(r) => r,
        };
        println!("{url} returned first");
        match maybe_title {
            Some(title) => println!("Title: {title}"),
            None => println!("No title found"),
        }
    })
    // let response = trpl::get
}
