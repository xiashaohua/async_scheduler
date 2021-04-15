use tokio_cron_scheduler::{Job, JobScheduler,JobToRun,ClientTask};
use std::{pin::Pin, process::Output, time::Duration};
use async_trait::async_trait;
use tokio::sync::RwLock;


use uuid::Uuid;
use std::sync::{Arc,Mutex};
use std::future::Future;
use tokio;
use async_std;

pub const CRON:i8 = 1;
pub const ONCE:i8 = 2;



struct ClientTaskInfo where  {
    pub schedule_type:i8,
    pub task_interval:String,

    pub task_name:String,
    pub task_handler: Uuid
}


impl ClientTaskInfo {
    pub fn new(schedule_type:i8, task_interval:String, task_name: String, task_handler: Uuid) -> Self {
        ClientTaskInfo {
            schedule_type, // 0,1,2
            task_interval,
            task_name,
            task_handler:task_handler,
        }
    }
}

pub struct TaskSchedulerLocked(Arc<RwLock<TaskManager>>);

impl Clone for TaskSchedulerLocked {
    fn clone(&self) -> Self {
        TaskSchedulerLocked(self.0.clone())
    }
}




pub type JobFn = Box<dyn Send + Sync + Fn() -> Pin<Box<dyn Send + Future<Output = ()>>>>;

struct TaskManager {
    jobs: Vec<ClientTaskInfo>,
    scheduler_instance: JobScheduler,
    test : Vec<JobFn>,
}

impl TaskSchedulerLocked {
    pub  fn new() -> Self {
        let  sched = JobScheduler::new();
        let r = TaskManager {
            jobs: vec![],
            scheduler_instance: sched.clone(),
            test: vec![]
           
        };
        TaskSchedulerLocked(Arc::new(RwLock::new(r)))
        
    }

    pub async  fn register_task<F, Fut>(&mut  self, schedule_type: i8, task_interval: String, task_name: String, task: F)
    where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
    F: 'static + Send + Sync,
    Fut: 'static + Send, Fut: Sync
    {
        let job: Job;
        



        if schedule_type == CRON {


            job = Job::new(
                &task_interval,
                Box::new(move |_uuid,_l|  {Box::pin(task())} )
            ).unwrap();
        }else if schedule_type == ONCE {
            job = Job::new_one_shot(
                Duration::from_secs(1),
                Box::new(move |_uuid,_l|  {Box::pin(task())} )
                ).await.unwrap();
        }else{
            let interval = task_interval.parse::<u64>();
            let interval = match interval {
                Ok(v) => {
                    v
                }
                Err(error) => {
                    println!("faild to parse interval string");
                    return
                }
            };
            println!("interval is {}",interval);
            job = Job::new_repeated(
                Duration::from_secs(interval),
                    Box::new(move |_uuid,_l|  {Box::pin(task())} )).await.unwrap();
        }
        let mut self_w = self.0.write().await;

        let task_handler = job.guid().await;
        let task = ClientTaskInfo::new(schedule_type, task_interval, task_name, task_handler);
        self_w.scheduler_instance.add(job).await;
        self_w.jobs.push(task);
        
    }




    pub async fn start(self) {
        
        //self_w.scheduler_instance
        let mut self_w = self.0.read().await;
        self_w.scheduler_instance.start().await;
        loop {
            tokio::time::sleep(core::time::Duration::from_millis(500)).await;
        }
       

    }


    pub async fn start_test(self) {
        

        tokio::spawn(async move
        {
            let l = self.clone();
            let mut self_w = l.0.read().await;

            for task in &self_w.test {
                task().await;
            }
        });
    }
}

#[tokio::main]
async fn main() {


    let mut   tm  = TaskSchedulerLocked::new();
    // tm.register_task(CRON, "1/10 * * * * *".to_string() , "task1".to_string(), || {test()});
    // tm.register_task(ONCE, "1".to_string(), "test2".to_string(), || {test1()});
    let a = Task1{id:3};
    tm.register_task(CRON, "1/5 * * * * *".to_string(), "test3".to_string(),   || async {test2().await;} ).await;
    tm.start().await;



}


fn test() {
    println!("test");
}


fn test1() {
    println!("test1");
}

async fn test2() {
    println!("test3");
}