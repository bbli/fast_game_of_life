use std::sync::{Arc,Mutex,MutexGuard,RwLock,RwLockWriteGuard,RwLockReadGuard};
use std::ops::{Deref, DerefMut};

pub struct MyArcMut<T>(Arc<Mutex<T>>);
impl<T> MyArcMut<T>{
    pub fn new(value: T)->Self{
        MyArcMut(Arc::new(Mutex::new(value)))
    }
    // use this only if set and get are too limiting
    pub fn grab_lock(&self) -> MutexGuard<T>{
        self.0.lock().unwrap()
    }
}

impl<T:Clone> MyArcMut<T>{
    pub fn set(&mut self, value : T){
        *self.0.lock().unwrap().deref_mut() = value;
    }
    pub fn get(&self) -> T{
        self.0.lock().unwrap().deref().clone()
    }
}

impl<T> Clone for MyArcMut<T>{
    fn clone(&self) -> MyArcMut<T>{
        MyArcMut(self.0.clone())
    }
}

pub struct MyArcRwLock<T>(Arc<RwLock<T>>);
impl<T> MyArcRwLock<T>{
    pub fn new(value: T)->Self{
        MyArcRwLock(Arc::new(RwLock::new(value)))
    }
    pub fn grab_writer_lock(&self) -> RwLockWriteGuard<T>{
        self.0.write().unwrap()
    }
    pub fn grab_reader_lock(&self) -> RwLockReadGuard<T>{
        self.0.read().unwrap()
    }
}
impl<T> Clone for MyArcRwLock<T>{
    fn clone(&self) -> MyArcRwLock<T>{
        MyArcRwLock(self.0.clone())
    }
}
