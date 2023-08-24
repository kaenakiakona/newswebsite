use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

/*
**??BROAD STROKES: 
In the main, a thread pool with 4 worker threads is created using new function, then each request
by the user is sent to the channel through ThreadPool.execute

Within ThreadPool.new, 4 new worker threads are created using Worker.new, passing in the receiver
that matches the sender in the channel created just before Worker.new is called

The code executes requests as each worker thread is created (within Worker.new), where the requests are represented as 
closures (in this case called Job)

 */

trait FnBox //use this instead of FnOnce() to move ownership of T inside of Box<T> in Worker.new()
{
    fn call_box(self: Box<Self>); //takes ownership of self and moves ownership out of Box<T>
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)() //uses *self to move F out of closure 
    }
}

type Job =  Box<dyn FnBox + Send + 'static>; //why need dyn?
//types = closures

enum Message //for worker threads to know if they should run because there is a job for them or exit loop and stop
{
    NewJob(Job),
    Terminate,
}
pub struct ThreadPool
{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}



impl ThreadPool
{
    pub fn new(size: usize) -> ThreadPool //size var needs to be type usize for with_capacity to work
    {
        assert!(size > 0); //program panics if size is 0
        
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel(); //mpsc::channel returns sender and reciever pair
        let receiver = Arc::new(Mutex::new(receiver)); //arc allows us to pass receiver to multiple worker threads, mutex ensures that only one worker is trying to request a job at a time

        for id in 0..size
        {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        ThreadPool //name of returned fields needs to be the same as in struct definition
        {
            workers,
            sender, //stores sending end of a channel that sends Job instances
        }

    }

//note: Closure is a one off function, like lambda in Python; in this case, used to represent task for worker thread to complete
    pub fn execute<F>(&self, f: F) //F is closure type that will be passed into execute
        where
            F: FnOnce() + Send + 'static 
            //above line defines the rules for the closure function F
            //FnOnce allows closure function to be called once (task can only be completed one time) 
            //Send allows the closure to be sent across thread bounds; allows it to be executed by another thread in the pool
            //static is lifetime of F
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap(); //sends job to receiver using .send member function for Sender type, 

    
        

    }
}

impl Drop for ThreadPool
{
    fn drop(&mut self)
    {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap(); //send terminate message to each thread; stops from executing
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers //worker is pointer/iterator going over each element in the vector workers?
        {
            println!("Shutting down worker {}", worker.id);

           
           if let Some(thread) = worker.thread.take() //if there is a thread (not None), take ownership and join it
           {
                thread.join().unwrap(); 
           }
           

        }
    }
}


struct Worker //private cus external code never needs it
{
    id: usize,
    thread: Option<thread::JoinHandle<()>> //JoinHandle basically just represents the threads; generally used for waiting for the completion of each thread
    //use option so that when we join the threads at the end of the program, it can be None
}

impl Worker
{
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker //***??Receiver<Job> basically represents the job it has to do, where Receiver holds the function to execute and Job provides the rules for how to execute that function
    {
        let thread = thread::spawn(move ||{
            loop{
                let message = reciever.lock().unwrap().recv().unwrap(); //lock gets Mutex; panics if it is poisoned; recv just gets a job from the channel
                match message
                {
                    Message::NewJob(job) => //****Where is job coming from?; we just changed the variable name to message?
                    {
                        println!("Worker {} got a job; executing.", id);
                
                        job.call_box();
                    },
                    Message::Terminate => 
                    {
                        println!("Worker {} was told to terminate.", id);

                        break;
                    },

                }
                

                //(*job)(); //issue because when we try to call FnOnce that is stored in Box<T>, closure needs to be able to move itself out of Box<T>. When we call the closure it takes ownership of self

            }
            
        });
        Worker
        {
            id,
            thread: Some(thread), //wrap in Some because thread is of type Option while in this function, its of type JoinHandle
    
        }
    }
}

//api key: 16d768c860044a7e9e0aaef6c617d71e

