use std::thread::JoinHandle;
use std::thread;
use std::mem;
use std::time::{SystemTime, UNIX_EPOCH};
use std::ops::{Deref, DerefMut};

// for globals
use super::*;

mod sync_utils;
use sync_utils::*;

mod b_matrix_vector;
pub use b_matrix_vector::*;

mod engine;
pub use engine::*;
// ************  MAIN CODE  ************   
pub struct MainWorkerHandle(JoinHandle<()>);
impl MainWorkerHandle{
    pub fn signal(&self){
        self.0.thread().unpark();
    }
}

#[derive(Copy,Clone)]
pub enum WorkFlag{
    InProgress,
    Done
}

pub struct BMatrix {
    pub vec: MyArcRwLock<BMatrixVector>,
    pub new_vec: MyArcMut<BMatrixVector>,
    main_worker_thread: MainWorkerHandle,
    status: MyArcMut<WorkFlag>,
}

impl BMatrix {
    pub fn new(update_method: Backend) -> Self {
        let vec = MyArcRwLock::new(BMatrixVector::default());
        let vec2 = vec.clone();
        let new_vec = MyArcMut::new(BMatrixVector::default());
        let new_vec2 = new_vec.clone();

        let status = MyArcMut::new(WorkFlag::Done);
        //let status = Arc::new(Mutex::new(WorkFlag::Done));
        let status2 = status.clone();

        // Spin up new thread and have it sleep until event loop starts and BMatrix calls signal
        let main_worker_thread = thread::spawn(
            move ||{
                let mut main_worker = MainWorker::new(update_method,status2,new_vec2,vec2);
                main_worker.sync_worker_do_work();
            });
        BMatrix {
            vec,
            new_vec,
            main_worker_thread: MainWorkerHandle(main_worker_thread),
            status,
        }
    }

    pub fn sync_main_update_backend(&mut self){
        if let WorkFlag::Done = self.status.get() {
            // no need to lock since MainWorker can't modify
            // until we call signal anyways
            self.update_vector();

            self.status.set(WorkFlag::InProgress);
            
            self.main_worker_thread.signal();
        }
    }
    fn update_vector(&mut self){
        // utilizing low level nature of swap function to do shallow swap
        let mut new_vec_lock = self.new_vec.grab_lock();
        let new_vec_raw: &mut BMatrixVector = new_vec_lock.deref_mut();

        let mut vec_lock = self.vec.grab_writer_lock();
        let vec_raw: &mut BMatrixVector = vec_lock.deref_mut();
        mem::swap(vec_raw,new_vec_raw);
    }
}

// ************  WORKER CODE  ************   
struct MainWorker{
    new_vec: MyArcMut<BMatrixVector>,
    vec: MyArcRwLock<BMatrixVector>,
    status: MyArcMut<WorkFlag>,
    update_engine: Box<dyn Engine>
}
impl MainWorker{
    fn new(update_method: Backend, status: MyArcMut<WorkFlag>, new_vec: MyArcMut<BMatrixVector>, vec: MyArcRwLock<BMatrixVector>)->Self{
        let update_engine = engine::create_engine(update_method);
        MainWorker{
            new_vec,
            vec,
            status,
            update_engine
        }
    }
    fn wait(&self){
        thread::park();
    }
    fn sync_worker_do_work(&mut self){
        loop{
            self.wait();

            let sys_time = SystemTime::now();
            self.backendMethodDispatch();
            print_time_lapse(sys_time);

            self.status.set(WorkFlag::Done);
        }
    }
    fn backendMethodDispatch(&mut self) {
        // so I can temporarily bypass arc + mutex restrictions
        // for multi-threading purposes
        let mut new_vec_lock = self.new_vec.grab_lock();
        let new_vec_raw = new_vec_lock.deref_mut();

        let vec_lock = self.vec.grab_reader_lock();
        let vec_raw = vec_lock.deref();

        self.update_engine.next_b_matrix(vec_raw,new_vec_raw);
    }
}

fn print_time_lapse(sys_time: SystemTime){
    let time_lapse = sys_time
        .elapsed()
        .expect("System clock did something funny");
    println!("Time elapsed: {}ms", time_lapse.as_millis());
}
