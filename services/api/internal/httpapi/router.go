package httpapi

import (
	"encoding/json"
	"net/http"

	"github.com/runroute/runroute/services/api/internal/health"
)

func NewRouter(service *health.Service) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /api/v1/health", func(writer http.ResponseWriter, request *http.Request) {
		report := service.Check(request.Context())
		statusCode := http.StatusOK
		if report.Status != health.StatusUp {
			statusCode = http.StatusServiceUnavailable
		}
		writer.Header().Set("Cache-Control", "no-store")
		writer.Header().Set("Content-Type", "application/json")
		writer.WriteHeader(statusCode)
		_ = json.NewEncoder(writer).Encode(report)
	})
	return mux
}
