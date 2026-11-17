// Recieve's HTTP requests , decodes JSON , returns reponse
package internals;
import (
	"context"
	"log"
	"net"
	"worker/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

// server implements pb.GreeterServer.
type server struct {
	pb.UnimplementedTaskServer // Required for forward compatibility
}

func (s *server) FetchTask(ctx context.Context, req *pb.TaskContent ) (*pb.Result, error) {
	
	//execute task 
	response := execute_task(req.GetId(), req.GetCommand(), req.GetArgs());

	return &pb.Result{
		Content: []string{string(response.content)},
		Id:      response.id,
		Success: response.success,
	}, nil

}

func ProtoServer() {
	// 1. Open a TCP listener on port 50051
	lis, err := net.Listen("tcp", ":50051")
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	grpcServer := grpc.NewServer()

	pb.RegisterTaskServer(grpcServer, &server{})

	reflection.Register(grpcServer)

	log.Println("gRPC server listening on :50051")

	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}

