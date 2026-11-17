//Initialize dependencies, start HTTP server, handle shutdown

package main

import (
	"fmt"
	"time"
	"worker/internals"
)

func periodicHeartbeat() {
	fmt.Println("Heartbeat thread launched");
	for {
		time.Sleep(5 * time.Second)
		internals.Heartbeat();
	}
}

func main() {
	go periodicHeartbeat();
	internals.ProtoServer()
}
