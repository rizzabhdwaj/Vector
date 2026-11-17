// defines the Task Struct and related Functionalities 
package internals;

import (
	"encoding/json"
	"log"
)

type Task struct{
	Id int `json:"id"`;
	command string `json:"command"`;	
	args string `json:"args"`;
}


func new_task(data []byte) (*Task , error){
	var new_task Task ; 
	err := json.Unmarshal(data, &new_task);
	
	if err != nil {
		log.Fatalf("Error unmarshaling JSON: %v", err)
		return nil , err ;
	}
	
	return &new_task, nil; 
}


