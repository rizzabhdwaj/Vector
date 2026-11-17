// executes task and caputre results

package internals;
import ("log"
	"os/exec") 

type Response struct {
	content []byte ;  
}

func execute_task(req_task Task) (*Response , error ) {
	cmd := exec.Command(req_task.command, req_task.args);
	out, err := cmd.Output()

	if err != nil {
		log.Fatalf("Command failed: %v", err)
		return nil , err ;
	}
	return &Response{content:out},nil;
	
}
