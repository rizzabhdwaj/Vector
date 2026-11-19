// Recieve's HTTP requests , decodes JSON , returns reponse
package internals

import (
	"context"
	"fmt"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
	"log"
	"net"
	"os"
	"worker/proto"
)
// server implements pb.GreeterServer.
type server struct {
	pb.UnimplementedTaskServer // Required for forward compatibility
}

func (s *server) FetchTask(ctx context.Context, req *pb.TaskContent) (*pb.Result, error) {

	//execute task
	response := execute_task(req.GetId(), req.GetCommand(), req.GetArgs())
	workerID := os.Getenv("WORKER_GRPC")
	
	status := "failed"
	
	if response.success {
		status = "compleated"
	}
	
	fmt.Printf("worker : %s %s %d\n", workerID, status, response.id)
	
	return &pb.Result{
		Content: []string{string(response.content)},
		Id:      response.id,
		Success: response.success,
	}, nil

}

func ProtoServer() {
	addr := os.Getenv("WORKER_GRPC")
	if addr == "" {
		addr = ":50051"
	}
	lis, err := net.Listen("tcp", addr)
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	grpcServer := grpc.NewServer()

	pb.RegisterTaskServer(grpcServer, &server{})

	reflection.Register(grpcServer)

	log.Printf("gRPC server listening on %s", addr)

	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}
