use crate::task_queue::queue::Tasks;

pub struct State {
    task: Arc<Mutex<Tasks>>,
}

impl State {
    pub fn new(tasks: Arc<Mutex<Tasks>>) -> State {
        State { tasks };
    }
}
