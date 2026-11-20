
# Observing and Figuring out what im building 

![Vector Architecture](./ithink.jpg)

## Finished the basic working of all 3 components (Frotnend , Workers , Scheduler)
### i belive the scheduler is not working correctly cuz when i send the 2nd request after the 1st one on /task i dont see the 2nd worker busy immediately i wonder whats wrong 
#### lets print what worker is doing what task

```rust 

            for worker in &mut workers.workers {
                if worker.available {
                    let Some(task) = tasks.content.pop_front() else {
                        break;
                    };
                    println!("assigned task {0} to worker {1}", task.id, worker.id);
                    worker.available = false;
                    assignments.push((worker.id, task));
                }
```
```output
assigned task 597705660 to worker 5003  -- t = 0
assigned task 2045042590 to worker 5003 -- t = 5
assigned task 1764651568 to worker 5003 -- t = 10 
```
ok soo .. rust is not assigning the task to the free worker instead waiting for the latest one to be free 

### Solution 
## this dumb ahh scheduler was waiting for the worker to compleate the work before ending the funciton with worker.available = true again (yeah AI aint taking my job)

### how i found it ? 
## professional debugging - [ println!() println!() println!() println!() ]

Im scared of crashing my Software , even delibarately ..... i should work on that .

