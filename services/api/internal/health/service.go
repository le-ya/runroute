package health

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	routev1 "github.com/runroute/runroute/services/api/internal/gen/route/v1"
	"google.golang.org/grpc"
)

const ServiceVersion = "phase1"

type Database interface {
	Ping(context.Context) error
}

type Optimizer interface {
	Check(context.Context, *routev1.CheckRequest, ...grpc.CallOption) (*routev1.CheckResponse, error)
}

type Status string

const (
	StatusUp   Status = "up"
	StatusDown Status = "down"
)

type Dependency struct {
	Status         Status `json:"status"`
	Version        string `json:"version,omitempty"`
	DatasetVersion string `json:"dataset_version,omitempty"`
	Error          string `json:"error,omitempty"`
	LatencyMS      int64  `json:"latency_ms"`
}

type Report struct {
	Status       Status                `json:"status"`
	Version      string                `json:"version"`
	Dependencies map[string]Dependency `json:"dependencies"`
}

type Service struct {
	database       Database
	optimizer      Optimizer
	graphHopperURL string
	httpClient     *http.Client
	probeTimeout   time.Duration
}

func NewService(database Database, optimizer Optimizer, graphHopperURL string, httpClient *http.Client, probeTimeout time.Duration) *Service {
	return &Service{
		database:       database,
		optimizer:      optimizer,
		graphHopperURL: strings.TrimRight(graphHopperURL, "/"),
		httpClient:     httpClient,
		probeTimeout:   probeTimeout,
	}
}

func (s *Service) Check(ctx context.Context) Report {
	report := Report{
		Status:  StatusUp,
		Version: ServiceVersion,
		Dependencies: map[string]Dependency{
			"database":     s.checkDatabase(ctx),
			"route_engine": s.checkOptimizer(ctx),
			"graphhopper":  s.checkGraphHopper(ctx),
		},
	}
	for _, dependency := range report.Dependencies {
		if dependency.Status != StatusUp {
			report.Status = StatusDown
			break
		}
	}
	return report
}

func (s *Service) checkDatabase(ctx context.Context) Dependency {
	return s.timedProbe(ctx, func(probeCtx context.Context) (Dependency, error) {
		if err := s.database.Ping(probeCtx); err != nil {
			return Dependency{}, err
		}
		return Dependency{Status: StatusUp}, nil
	})
}

func (s *Service) checkOptimizer(ctx context.Context) Dependency {
	return s.timedProbe(ctx, func(probeCtx context.Context) (Dependency, error) {
		response, err := s.optimizer.Check(probeCtx, &routev1.CheckRequest{})
		if err != nil {
			return Dependency{}, err
		}
		dependency := Dependency{Version: response.GetServiceVersion()}
		if response.GetStatus() != routev1.ServingStatus_SERVING_STATUS_UP {
			return dependency, fmt.Errorf("route engine status is %s", response.GetStatus())
		}
		for _, provider := range response.GetDependencies() {
			if provider.GetStatus() != routev1.ServingStatus_SERVING_STATUS_UP {
				return dependency, fmt.Errorf("route engine dependency %s is %s", provider.GetName(), provider.GetStatus())
			}
		}
		dependency.Status = StatusUp
		return dependency, nil
	})
}

type graphHopperInfo struct {
	Version    string            `json:"version"`
	ImportDate string            `json:"import_date"`
	DataDate   string            `json:"data_date"`
	Profiles   []json.RawMessage `json:"profiles"`
}

func (s *Service) checkGraphHopper(ctx context.Context) Dependency {
	return s.timedProbe(ctx, func(probeCtx context.Context) (Dependency, error) {
		request, err := http.NewRequestWithContext(probeCtx, http.MethodGet, s.graphHopperURL+"/info", nil)
		if err != nil {
			return Dependency{}, err
		}
		response, err := s.httpClient.Do(request)
		if err != nil {
			return Dependency{}, err
		}
		defer response.Body.Close()

		if response.StatusCode != http.StatusOK {
			return Dependency{}, fmt.Errorf("info endpoint returned %s", response.Status)
		}
		var payload graphHopperInfo
		if err := json.NewDecoder(response.Body).Decode(&payload); err != nil {
			return Dependency{}, fmt.Errorf("decode info response: %w", err)
		}
		datasetVersion := payload.DataDate
		if payload.ImportDate != "" {
			datasetVersion += "/" + payload.ImportDate
		}
		dependency := Dependency{Version: payload.Version, DatasetVersion: datasetVersion}
		if payload.Version == "" {
			return dependency, errors.New("GraphHopper version is empty")
		}
		if len(payload.Profiles) == 0 {
			return dependency, errors.New("GraphHopper has no routing profiles")
		}
		if payload.DataDate == "" {
			return dependency, errors.New("GraphHopper data date is empty")
		}
		dependency.Status = StatusUp
		return dependency, nil
	})
}

func (s *Service) timedProbe(ctx context.Context, probe func(context.Context) (Dependency, error)) Dependency {
	started := time.Now()
	probeCtx, cancel := context.WithTimeout(ctx, s.probeTimeout)
	defer cancel()

	dependency, err := probe(probeCtx)
	dependency.LatencyMS = time.Since(started).Milliseconds()
	if err != nil {
		dependency.Status = StatusDown
		dependency.Error = err.Error()
	}
	return dependency
}
