/*
 * should contain a struct called orphan_mgr which should contain mutex of worker's and a mutex of
 * task_queue 
 * impl should contain a functionc called run which will check if any worker is dead after every
 * timeperiod of const TIMEOUT (5s) , if any worker is found dead then it should run the
 * task_queue.re_schedule(task) for the dead task and remove th deadtask from the worker 
