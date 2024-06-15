use async_channel::Receiver;
use monoio::utils::CtrlC;

fn main() {
    let num_threds = 6;
    let mut threads = Vec::new();
    let (s, r) = async_channel::bounded(1);
    for cpu in 0..num_threds {
        let r = r.clone();
        let s = s.clone();
        let thread = std::thread::spawn(move || {
            monoio::utils::bind_to_cpu_set(Some(cpu)).unwrap();
            let mut rt = monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
                .enable_timer()
                .with_blocking_strategy(monoio::blocking::BlockingStrategy::ExecuteLocal)
                .build()
                .unwrap();

            rt.block_on(async move {
                if cpu == 0 {
                    monoio::select! {
                        _ = simulate_command_handling(r.clone()) => {
                        },
                        _ = simulate_other_command(r) => {
                        },
                        _ = CtrlC::new().unwrap() => {
                            let thread_id = std::thread::current().id();
                            println!("Exiting... on thread: {:?}", thread_id);
                            // Simulate a broadcast of stop events
                            for _ in 0..num_threds - 1 {
                                s.send(()).await.unwrap();
                            }
                        }
                    }
                } else {
                    monoio::select! {
                        _ = simulate_command_handling(r.clone()) => {
                        }
                        _ = simulate_other_command(r) => {
                        },
                    }
                }
            })
        });
        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }
}

async fn simulate_command_handling(r: Receiver<()>) {
    let thread_id = std::thread::current().id();
    loop {
        monoio::select! {
            _ = monoio::time::sleep(std::time::Duration::from_secs(1)) => {
                println!("Simulate command from thread_id: {:?}", thread_id);
            }
            _ = r.recv() => {
                break;
            }

        }
    }
}

async fn simulate_other_command(r: Receiver<()>) {
    let thread_id = std::thread::current().id();
    loop {
        monoio::select! {
            _ = monoio::time::sleep(std::time::Duration::from_secs(2)) => {
                println!("Other simulation from thread_id: {:?}", thread_id);
            }
            _ = r.recv() => {
                break;
            }
        }
    }
}
