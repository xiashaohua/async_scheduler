use crate::job::{JobLocked, JobType};
use chrono::Utc;
use tokio::sync::RwLock;
use std::sync::{Arc};
use tokio::task::JoinHandle;
use uuid::Uuid;


/// The JobScheduler contains and executes the scheduled jobs.
pub struct JobsSchedulerLocked(Arc<RwLock<JobScheduler>>);

impl Clone for JobsSchedulerLocked {
    fn clone(&self) -> Self {
        JobsSchedulerLocked(self.0.clone())
    }
}

#[derive(Default)]
pub struct JobScheduler {
    jobs: Vec<JobLocked>,
}

unsafe impl Send for JobScheduler {}

impl Default for JobsSchedulerLocked {
    fn default() -> Self {
        Self::new()
    }
}

impl JobsSchedulerLocked {

    /// Create a new `JobSchedulerLocked`.
    pub fn new() -> JobsSchedulerLocked {
        let r = JobScheduler {
            ..Default::default()
        };
        JobsSchedulerLocked(Arc::new(RwLock::new(r)))
    }

    /// Add a job to the `JobScheduler`
    ///
    /// ```rust,ignore
    /// use tokio_cron_scheduler::{Job, JobScheduler, JobToRun};
    /// let mut sched = JobScheduler::new();
    /// sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 10 seconds!");
    /// }));
    /// ```
    pub async fn add(&mut self, job: JobLocked) -> Result<(), Box<dyn std::error::Error + '_>> {
        {
            let mut self_w = self.0.write().await;
            self_w.jobs.push(job);
        }
        Ok(())
    }

    /// Remove a job from the `JobScheduler`
    ///
    /// ```rust,ignore
    /// use tokio_cron_scheduler::{Job, JobScheduler, JobToRun};
    /// let mut sched = JobScheduler::new();
    /// let job_id = sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 10 seconds!");
    /// }));
    /// sched.remove(job_id).await;
    /// ```
    pub async fn remove(&mut self, to_be_removed: &Uuid) -> Result<(), Box<dyn std::error::Error + '_>> {
        {
            let mut ws = self.0.write().await;
            let mut removed: Vec<JobLocked> = vec![];


            for f in &ws.jobs {

                //let f = f.0.read().await;
                let not_to_be_removed = if let f = f.0.read().await {
                    f.job_id().eq(&to_be_removed)
                }else{
                    false
                };
                if !not_to_be_removed {
                    let f = f.0.clone();
                    removed.push(JobLocked(f))
                }
            }

            for job in removed {
                let mut job_r = job.0.write().await;
                job_r.set_stopped();
                let job_type = job_r.job_type();
                if matches!(job_type, JobType::OneShot) || matches!(job_type, JobType::Repeated) {
                    job_r.abort_join_handle();
                }
            }
        }
        Ok(())
    }

    /// The `tick` method increments time for the JobScheduler and executes
    /// any pending jobs. It is recommended to sleep for at least 500
    /// milliseconds between invocations of this method.
    /// This is kept public if you're running this yourself. It is better to
    /// call the `start` method if you want all of this automated for you.
    ///
    /// ```rust,ignore
    /// loop {
    ///     sched.tick().await;
    ///     std::thread::sleep(Duration::from_millis(500)).await;
    /// }
    /// ```
    pub async fn tick(&mut self) -> Result<(), Box<dyn std::error::Error + '_>> {
        let l = self.clone();
        {
            let mut ws = self.0.write().await;
            for jl in ws.jobs.iter_mut() {
                if jl.tick().await {
                    let ref_for_later = jl.0.clone();
                    let jobs = l.clone();
                    // tokio::spawn(async move {
                    let e = ref_for_later.write().await;
                    if let mut w = e {
                        let jt = w.job_type();
                        if matches!(jt, JobType::OneShot) {
                            let mut jobs = jobs.clone();
                            let job_id = w.job_id();
                            tokio::spawn(async move {
                                if let Err(e) = jobs.remove(&job_id).await {
                                    eprintln!("Error removing job {:}", e);
                                }
                            });
                        }
                        w.run(jobs).await;
                    }
                    // });
                }
            }
        }

        Ok(())
    }


    /// The `start` spawns a Tokio task where it loops. Every 500ms it
    /// runs the tick method to increment any
    /// any pending jobs.
    ///
    /// ```rust,ignore
    /// if let Err(e) = sched.start().await {
    ///         eprintln!("Error on scheduler {:?}", e);
    ///     }
    /// ```
    pub async fn start(&self)  {
        let jl: JobsSchedulerLocked = self.clone();
        let jh: JoinHandle<()> = tokio::spawn(async move {
            loop {
                tokio::time::sleep(core::time::Duration::from_millis(500)).await;
                let mut jsl = jl.clone();
                let tick = jsl.tick().await;
                if let Err(e) = tick {
                    eprintln!("Error on job scheduler tick {:?}", e);
                    break;
                }
            }
        });
        
    }

    /// The `time_till_next_job` method returns the duration till the next job
    /// is supposed to run. This can be used to sleep until then without waking
    /// up at a fixed interval.AsMut
    ///
    /// ```rust, ignore
    /// loop {
    ///     sched.tick();
    ///     std::thread::sleep(sched.time_till_next_job());
    /// }
    /// ```
    pub async fn time_till_next_job(
        &self,
    ) -> Result<std::time::Duration, Box<dyn std::error::Error + '_>> {
        let r = self.0.read().await;
        if r.jobs.is_empty() {
            // Take a guess if there are no jobs.
            return Ok(std::time::Duration::from_millis(500));
        }
        let now = Utc::now();


        let mut time_d = vec![];
        for j in r.jobs.iter() {
            let diff = {
                let j = j.0.read().await;
                j.schedule()
                    .and_then(|s| s.upcoming(Utc)
                        .take(1)
                        .find(|_| true)
                        .map(|next| next - now))

                };
            time_d.push(diff);
        }

        let min = time_d
        .iter()
        .filter(|d| d.is_some())
        .map(|d| d.unwrap())
        .min();
        // let min = r
        //     .jobs
        //     .iter()
        //     .map(|j| async move {
        //         let diff = {
        //             let j = j.0.read().await;
        //             j.schedule()
        //                 .and_then(|s| s.upcoming(Utc)
        //                     .take(1)
        //                     .find(|_| true)
        //                     .map(|next| next - now))

                    
        //         };
        //         diff
        //     })
        //     .filter(|d| d.is_some())
        //     .map(|d| d.unwrap())
        //     .min();

        let m = min
            .unwrap_or_else(chrono::Duration::zero)
            .to_std()
            .unwrap_or_else(|_| std::time::Duration::new(0, 0));
        Ok(m)
    }
}
