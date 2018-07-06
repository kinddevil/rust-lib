//go:generate protoc -I ../helloworld --go_out=plugins=grpc:../helloworld ../helloworld/helloworld.proto

package main

import (
	"log"
	"net"

	"golang.org/x/net/context"
	"google.golang.org/grpc"
	// pb "google.golang.org/grpc/examples/helloworld/helloworld"
	"google.golang.org/grpc/reflection"
	// "gopkg.in/alexcesaro/statsd.v2"
	pb "taskpb"
	// "sync/atomic"
)

const (
	port = ":50051"
)

type server struct{}

func (s *server) CreateTask(ctx context.Context, in *pb.TaskReq) (*pb.TaskReply, error) {
	log.Printf("receive task msg %v %v %v %v", in.SchoolID, in.ArrangementID, in.CalendarID, in.TaskID)

	return &pb.TaskReply{TaskID: "unique task id"}, nil
}

func main() {

	lis, err := net.Listen("tcp", port)
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}
	s := grpc.NewServer()
	pb.RegisterCoursesServer(s, &server{})
	// Register reflection service on gRPC server.
	reflection.Register(s)
	log.Println("start server at" + port)
	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
