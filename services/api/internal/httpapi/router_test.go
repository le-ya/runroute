package httpapi_test

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	routev1 "github.com/runroute/runroute/services/api/internal/gen/route/v1"
	"github.com/runroute/runroute/services/api/internal/health"
	"github.com/runroute/runroute/services/api/internal/httpapi"
	"google.golang.org/grpc"
)

type failedDatabase struct{}

func (failedDatabase) Ping(context.Context) error { return errors.New("unavailable") }

type failedOptimizer struct{}

func (failedOptimizer) Check(context.Context, *routev1.CheckRequest, ...grpc.CallOption) (*routev1.CheckResponse, error) {
	return nil, errors.New("unavailable")
}

func TestHealthEndpointReturnsServiceUnavailableWhenDependencyIsDown(t *testing.T) {
	graphHopper := httptest.NewServer(http.HandlerFunc(func(writer http.ResponseWriter, _ *http.Request) {
		writer.WriteHeader(http.StatusServiceUnavailable)
	}))
	defer graphHopper.Close()
	service := health.NewService(failedDatabase{}, failedOptimizer{}, graphHopper.URL, graphHopper.Client(), time.Second)
	request := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	response := httptest.NewRecorder()

	httpapi.NewRouter(service).ServeHTTP(response, request)

	if response.Code != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want %d", response.Code, http.StatusServiceUnavailable)
	}
	if got := response.Header().Get("Cache-Control"); got != "no-store" {
		t.Fatalf("Cache-Control = %q", got)
	}
	var report health.Report
	if err := json.NewDecoder(response.Body).Decode(&report); err != nil {
		t.Fatal(err)
	}
	if report.Status != health.StatusDown {
		t.Fatalf("report status = %q", report.Status)
	}
}
