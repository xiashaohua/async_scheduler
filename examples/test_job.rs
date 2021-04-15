use tokio_cron_scheduler::{Job, JobScheduler};
use std::{pin::Pin, process::Output, time::Duration};


use uuid::Uuid;
use std::sync::{Arc,RwLock,Mutex};
use std::future::Future;
use tokio;
use lazy_static::lazy_static;

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

lazy_static! {
    static ref TASK_CCC:Arc<Mutex<TaskManager>> = Arc::new(Mutex::new(TaskManager::new()));
}


pub type JobFn = Box<dyn Send + Sync + Fn() -> Pin<Box<dyn Send + Future<Output = ()>>>>;

struct TaskManager {
    jobs: Vec<ClientTaskInfo>,
    scheduler_instance: JobScheduler,
    test : Vec<JobFn>,
}

impl TaskManager {
    pub  fn new() -> Self {
        let  sched = JobScheduler::new();
        let r = TaskManager {
            jobs: vec![],
            scheduler_instance: sched.clone(),
            test: vec![]
           
        };
        r
    }

    pub async  fn register_task<F, Fut>(&mut  self, schedule_type: i8, task_interval: String, task_name: String, task: F)
    where
        F: Fn() -> Fut,
        Fut: Future<Output = ()>,
        F: 'static + Send + Sync,
        Fut: 'static + Send + Sync,
    {
        let job: Job;
        
        //let mut self_w = self.0.write().unwrap();
       

   
        self.test.push(Box::new(move || Box::pin(task())));

        return
    //     if schedule_type == CRON {


    //         job = Job::new(
    //             &task_interval,
    //             Box::new(move |_uuid,_l|  {Box::pin(task())} )
    //         ).unwrap();
    //     }else if schedule_type == ONCE {
    //         job = Job::new_one_shot(
    //             Duration::from_secs(1),
    //             Box::new(move |_uuid,_l|  {Box::pin(task())} )
    //             ).unwrap();
    //     }else{
    //         let interval = task_interval.parse::<u64>();
    //         let interval = match interval {
    //             Ok(v) => {
    //                 v
    //             }
    //             Err(error) => {
    //                 println!("faild to parse interval string");
    //                 return
    //             }
    //         };
    //         println!("interval is {}",interval);
    //         job = Job::new_repeated(
    //             Duration::from_secs(interval),
    //                 Box::new(move |_uuid,_l|  {Box::pin(task())} )).unwrap();
    //     }
    //     let mut self_w = self.0.write().unwrap();

    //     let task_handler = job.guid();
    //     let task = ClientTaskInfo::new(schedule_type, task_interval, task_name, task_handler);
       
    //     self_w.scheduler_instance.add(job).await;
    //     self_w.jobs.push(task);
        
    }



    pub fn unregister_task(&mut self, task_name: String) {
        let mut task_instance = Uuid::new_v4();
        let mut found_index = usize::MAX;
        //let mut self_w = &mut self.0.read().unwrap();
        for (index, task) in self.jobs.iter_mut().enumerate() {
            if task_name == task.task_name {
                task_instance = task.task_handler.clone();
                found_index = index;
                break;
            }
        }

        if found_index != usize::MAX {
            self.scheduler_instance.remove(&task_instance);
            self.jobs.remove(found_index);
        }
    }


    pub async fn start(self) {
        
        //self_w.scheduler_instance
        //let mut self_w = self.0.read().unwrap();
        self.scheduler_instance.start().await
       

    }


    pub async fn start_test(&'static self) {
        tokio::spawn(async move
        {
            //let mut self_w = &self.0.lock().unwrap();
            for task in &self.test {
                task().await;
            }
        });
    }
}

#[tokio::main]
async fn main() {



   
    //let mut   tm  = TaskManager::new();
    // tm.register_task(CRON, "1/10 * * * * *".to_string() , "task1".to_string(), || {test()});
    // tm.register_task(ONCE, "1".to_string(), "test2".to_string(), || {test1()});
    let mut ddd = TASK_CCC.lock().unwrap();
    ddd.register_task(CRON, "1/5 * * * * *".to_string(), "test3".to_string(),  || async {test2().await;}).await;
    println!("ccc");
    TASK_CCC.start_test().await;
    // tokio::spawn(
    //     async move {
    //         tm.start().await;

    //     }
    // );
    


    println!("ccc");
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