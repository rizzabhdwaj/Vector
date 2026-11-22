// Initialize dependencies, start HTTP server, handle shutdown

package main

import (
	"context"
	"log"
	"os"
	"strconv"
	"time"

	"github.com/joho/godotenv"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"worker/internals"
	pb "worker/proto"
)

func periodicHeartbeat(ctx context.Context, client pb.HeartbeatClient, workerID int32) {
	for {
		time.Sleep(5 * time.Second)
		internals.Heartbeat(ctx, client, workerID)
	}
}

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

	go periodicHeartbeat(context.Background(), client, int32(workerID))
	internals.ProtoServer()
}
