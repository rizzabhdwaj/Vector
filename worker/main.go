//Initialize dependencies, start HTTP server, handle shutdown

package main;

import (
	"worker/internals"
)

func main(){
	internals.ProtoServer();
}
