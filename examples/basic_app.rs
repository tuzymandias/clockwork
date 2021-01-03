use clockwork::ClockworkApp;
use clockwork::ClockworkHandle;
use clockwork::{Configurable, Runnable};
use serde::Deserialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::macros::support::{Future, Pin};
use tokio::time::{sleep, Duration, Instant};
use tracing::info;

#[derive(Deserialize, Clone)]
struct MultiEchoConfig {
    every_half_second: String,
    every_second: String,
    run_duration_secs: usize,
}

struct MultiEchoApp {
    conf: Arc<MultiEchoConfig>,
}

impl Configurable for MultiEchoApp {
    type Config = MultiEchoConfig;

    fn from(app_conf: Self::Config) -> Self {
        Self {
            conf: Arc::new(app_conf),
        }
    }
}

impl Runnable for MultiEchoApp {
    fn setup(&self, handle: ClockworkHandle) {
        let half_second = Duration::from_millis(500);
        let one_second = Duration::from_secs(1);

        let count = Arc::new(AtomicU32::new(0));

        {
            let count = count.clone();
            let conf = self.conf.clone();

            handle.schedule_repeating_task_at(
                move || {
                    info!(
                        "{}: {}",
                        conf.every_half_second,
                        count.fetch_add(1, Ordering::SeqCst)
                    )
                },
                Instant::now().checked_add(half_second).unwrap(),
                half_second,
            );
        }

        {
            let count = count.clone();
            let conf = Arc::clone(&self.conf);

            handle.schedule_repeating_task_at(
                move || {
                    info!(
                        "{}: {}",
                        conf.every_second,
                        count.fetch_add(1, Ordering::SeqCst)
                    )
                },
                Instant::now().checked_add(one_second).unwrap(),
                one_second,
            );
        }
    }

    fn shutdown(&self) {
        info!("MultiEchoApp shut down!");
    }

    fn run<'a>(&'a self, handle: ClockworkHandle) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let duration = Duration::from_secs(self.conf.run_duration_secs as u64);
        Box::pin(async move {
            sleep(duration).await;
            handle.stop()
        })
    }
}

fn main() {
    let conf_str = r#"
        [app]
        every_half_second = 'Hello'
        every_second = 'World'
        run_duration_secs = 10
        
        [logger]
        write_target = 'STDOUT'
    "#
    .to_string();

    let app: ClockworkApp<MultiEchoApp> = ClockworkApp::from_config_str(conf_str);
    app.start();
}
