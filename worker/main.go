// Initialize dependencies, start HTTP server, handle shutdown

package main

import (
	"context"
	"log"
	"os"
	"strconv"

	"github.com/joho/godotenv"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"worker/internals"
	pb "worker/proto"
)

func main() {
	if err := godotenv.Load(); err != nil {
		log.Fatal("Error loading .env file")
	}

	grpcAddr := os.Getenv("COORDINATOR_GRPC")

	workerID := int64(1)
	if value := os.Getenv("WORKER_ID"); value != "" {
		id, err := strconv.ParseInt(value, 10, 32)
		if err != nil {
			log.Fatalf("Invalid WORKER_ID: %v", err)
		}
		workerID = id
	}

	conn, err := grpc.NewClient(
		grpcAddr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	client := pb.NewHeartbeatClient(conn)
	heartbeat := internals.NewHeartbeatControl()

	go heartbeat.Run(context.Background(), client, int32(workerID))
	go internals.StartMetricsServer(heartbeat)
	internals.ProtoServer()
}
