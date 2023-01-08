use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::prelude::*;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

// This example shows a way to use indicatif with async tokio.
// You can imagine that this simulates the progress of a file downloader.
// I use sleep with a random duration, to let it look more realistic

// Thanks to alice for helping me with tokio::spawn https://ryhl.io
// https://users.rust-lang.org/t/limited-concurrency-for-future-execution-tokio/87171/7

// 1. We will have a main progress bar (pb_main). It stays at the bottom
// and shows the overall progress
// 2. We have a progress bar for each download task (pb_task).
// The pb_main and pb_task are both "just" progress bars,
// which are collect inside a MultiProgress (multi_pb).
// All progress bars use the same style (pb_style).

// The following variables can be adjusted:
// - ITEMS: Controls the number of total "downloads" and length of pb_main
// - MAX_CONCURRENT: Controls how many concurrent downloads are allowed
// - STEPS: Controls the length of pb_task

#[tokio::main]
async fn main() {
    // adjust these constants to change program behavior
    const ITEMS: u64 = 10;
    const MAX_CONCURRENT: usize = 3;
    const STEPS: u64 = 100;

    // set a style for all progress bar
    // a list of template keys can be found here:
    // https://docs.rs/indicatif/latest/indicatif/index.html#templates
    let pb_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:50.cyan/blue} {pos:>7}/{len:7} {msg} ",
    )
    .unwrap()
    .progress_chars("##-");

    // create a vec containing our "downloads" ... simple integer
    let ids: Vec<u64> = (0..ITEMS).into_iter().collect();

    // create a struct to manage our progress bars -> indicatif::MultiProgress
    let multi_pg = MultiProgress::new();

    // create a progress bar to track overall status
    let pb_main = multi_pg.add(ProgressBar::new(ITEMS));
    pb_main.set_style(pb_style.clone());
    pb_main.set_message("total  ");

    // Make the main progress bar render immediately rather than waiting for the
    // first task to finish.
    pb_main.tick();
    // tokio::task::JoinSet
    // setup the JoinSet to manage the join handles for our futures
    let mut set = JoinSet::new();

    // iterate over our downloads vec and
    // spawn a background task for each download (do_stuff)
    // Does not spawn more tasks than MAX_CONCURRENT "allows"
    for id in ids {
        // create a progress bar for each download and set the style
        // using insert_before() so that pb_main stays below the other progress bars
        let pb_task = multi_pg.insert_before(&pb_main, ProgressBar::new(STEPS));
        pb_task.set_style(pb_style.clone());

        // when limit is reached, wait until a running task finishes
        // await the future (join_next().await) and get the execution result
        // here result would be a download id(u64), as you can see in signature of do_stuff
        while set.len() >= MAX_CONCURRENT {
            // why unwrap().unwrap() => return value is wrapped in option and result
            // bla: Option<Result<u64, JoinError>> = set.join_next().await;
            // bla: Result<u64, JoinError> = set.join_next().await.unwrap();
            // bla: u64 = set.join_next().await.unwrap().unwrap(); or JoinError
            // you could use the result here. if there is no result () (unit type) is returned
            let _bla = set.join_next().await.unwrap().unwrap();
            /* use bla here */
            // we should replace the unwrap() above with some error handling
            // but for now we just increase the main progress by one
            pb_main.inc(1);
        }

        // spawns a background task immediatly no matter if the future is awaited
        // https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html#method.spawn
        set.spawn(do_stuff(id, STEPS, pb_task));
    }

    // some futures will be left over from the above loop
    // the number of "left overs" should be equal to MAX_CONCURRENT
    // await the future (join_next().await) and get the execution result
    while let Some(res) = set.join_next().await {
        let _ = res.unwrap();
        pb_main.inc(1);
    }
    // finally we can finish the pb_main and change the msg from "total" to "All Downloads fnished!"
    pb_main.finish_with_message("All Downloads finished");
}

async fn do_stuff(id: u64, steps: u64, pb_task: ProgressBar) -> u64 {
    // set a {msg} for the task progress bar, appears right next to the progress indicator
    pb_task.set_message(format!("RECV app with id:{}", id));

    // we create a loop with sleep to simulate download progress
    // using rand with a range (in millisecs) to create "download duration"
    // calculate "tick size" for each progress bar step "download duration" / "# of steps in pb_task"
    let num = rand::thread_rng().gen_range(steps..=10000);
    let tick = num / steps;

    // heavy downloading ...
    for _ in 0..steps {
        sleep(Duration::from_millis(tick)).await;
        pb_task.inc(1);
    }
    // finish the task progress bar
    // pb_task could also be returned from this function
    // and then used in the while loop the future is finally awaited
    pb_task.finish_with_message(format!("DONE app with id:{}", id));
    id
}
