use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::prelude::*;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

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

    println!(
        "\n Downloading {} files with max {} concurrent connections\n",
        ITEMS, MAX_CONCURRENT
    );
    // set a style for all progress bar
    // a list of template keys can be found here:
    // https://docs.rs/indicatif/latest/indicatif/index.html#templates
    let pb_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:50.cyan/blue} {pos:>7}/{len:7} {msg} ",
    )
    .unwrap()
    .progress_chars("##-");

    // create a vec containing our "downloads" ... simple integer
    // let ids: Vec<u64> = (0..ITEMS).into_iter().collect();
    let files: Vec<Uuid> = (0..ITEMS).into_iter().map(|_| Uuid::new_v4()).collect();

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

    let mut last_item = false;

    // iterate over our downloads vec and
    // spawn a background task for each download (do_stuff)
    // Does not spawn more tasks than MAX_CONCURRENT "allows"
    for (index, uuid) in files.iter().enumerate() {
        if index == files.len() - 1 {
            last_item = true;
        }

        // create a progress bar for each download and set the style
        // using insert_before() so that pb_main stays below the other progress bars
        let pb_task = multi_pg.insert_before(&pb_main, ProgressBar::new(STEPS));
        pb_task.set_style(pb_style.clone());

        // spawns a background task immediatly no matter if the future is awaited
        // https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html#method.spawn
        set.spawn(do_stuff(*uuid, index, STEPS, pb_task));

        // when limit is reached, wait until a running task finishes
        // await the future (join_next().await) and get the execution result
        // here result would be a download id(u64), as you can see in signature of do_stuff
        while set.len() >= MAX_CONCURRENT || last_item {
            match set.join_next().await {
                Some(_res) => {
                    // let foo = res.unwrap()
                    /* do something with foo */
                }
                None => {
                    break;
                }
            };
            pb_main.inc(1);
        }
    }
    pb_main.finish_with_message("All Downloads finished");
}

async fn do_stuff(uuid: Uuid, index: usize, steps: u64, pb_task: ProgressBar) -> Uuid {
    // set a {msg} for the task progress bar, appears right next to the progress indicator
    pb_task.set_message(format!("RECV file # {} with uuid:{}", index, uuid));

    // we create a loop with sleep to simulate download progress
    // using rand with a range (in millisecs) to create "download duration"
    // calculate "tick size" for each progress bar step "download duration" / "# of steps in pb_task"
    let num = rand::thread_rng().gen_range(steps..=5000);
    let tick = num / steps;

    // heavy downloading ...
    for _ in 0..steps {
        sleep(Duration::from_millis(tick)).await;
        pb_task.inc(1);
    }
    // finish the task progress bar
    // pb_task could also be returned from this function
    // and then used in the while loop the future is finally awaited
    pb_task.finish_with_message(format!("DONE file # {} with uuid:{}", index, uuid));
    uuid
}
