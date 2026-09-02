package health_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	routev1 "github.com/runroute/runroute/services/api/internal/gen/route/v1"
	"github.com/runroute/runroute/services/api/internal/health"
	"google.golang.org/grpc"
)

type databaseStub struct {
	err error
}

func (stub databaseStub) Ping(context.Context) error {
	return stub.err
}

type optimizerStub struct {
	response *routev1.CheckResponse
	err      error
}

func (stub optimizerStub) Check(context.Context, *routev1.CheckRequest, ...grpc.CallOption) (*routev1.CheckResponse, error) {
	return stub.response, stub.err
}

func TestCheckReportsAllLiveDependencies(t *testing.T) {
	graphHopper := graphHopperServer(t, http.StatusOK, `{"version":"10.2","import_date":"2026-08-31","data_date":"2026-08-30","profiles":[{"name":"foot"}]}`)
	service := health.NewService(
		databaseStub{},
		optimizerStub{response: &routev1.CheckResponse{
			Status:         routev1.ServingStatus_SERVING_STATUS_UP,
			ServiceVersion: "phase1",
			Dependencies: []*routev1.DependencyStatus{{
				Name:   "graphhopper",
				Status: routev1.ServingStatus_SERVING_STATUS_UP,
			}},
		}},
		graphHopper.URL,
		graphHopper.Client(),
		time.Second,
	)

	report := service.Check(context.Background())

	if report.Status != health.StatusUp {
		t.Fatalf("status = %q, want up: %#v", report.Status, report)
	}
	if got := report.Dependencies["graphhopper"].DatasetVersion; got != "2026-08-30/2026-08-31" {
		t.Fatalf("dataset version = %q", got)
	}
}

func TestCheckReportsDependencyFailures(t *testing.T) {
	graphHopper := graphHopperServer(t, http.StatusServiceUnavailable, `{}`)
	service := health.NewService(
		databaseStub{err: errors.New("database unavailable")},
		optimizerStub{err: errors.New("route engine unavailable")},
		graphHopper.URL,
		graphHopper.Client(),
		time.Second,
	)

	report := service.Check(context.Background())

	if report.Status != health.StatusDown {
		t.Fatalf("status = %q, want down", report.Status)
	}
	for _, name := range []string{"database", "route_engine", "graphhopper"} {
		dependency := report.Dependencies[name]
		if dependency.Status != health.StatusDown || dependency.Error == "" {
			t.Fatalf("dependency %s did not expose failure: %#v", name, dependency)
		}
	}
}

func graphHopperServer(t *testing.T, status int, body string) *httptest.Server {
	t.Helper()
	server := httptest.NewServer(http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
		if request.URL.Path != "/info" {
			t.Errorf("path = %q, want /info", request.URL.Path)
		}
		writer.WriteHeader(status)
		_, _ = writer.Write([]byte(body))
	}))
	t.Cleanup(server.Close)
	return server
}
