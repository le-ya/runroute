package main

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/runroute/runroute/services/api/internal/database"
	routev1 "github.com/runroute/runroute/services/api/internal/gen/route/v1"
	"github.com/runroute/runroute/services/api/internal/health"
	"github.com/runroute/runroute/services/api/internal/httpapi"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func main() {
	var err error
	if len(os.Args) == 2 && os.Args[1] == "healthcheck" {
		err = checkLocalHealth()
	} else {
		err = run()
	}
	if err != nil {
		slog.Error("api stopped", "error", err)
		os.Exit(1)
	}
}

func checkLocalHealth() error {
	client := &http.Client{Timeout: 3 * time.Second}
	response, err := client.Get(envOrDefault("API_HEALTHCHECK_URL", "http://127.0.0.1:8080/api/v1/health"))
	if err != nil {
		return err
	}
	defer response.Body.Close()
	if response.StatusCode != http.StatusOK {
		return fmt.Errorf("health endpoint returned %s", response.Status)
	}
	return nil
}

func run() error {
	listenAddress := envOrDefault("API_LISTEN_ADDRESS", ":8080")
	databaseURL := requiredEnv("DATABASE_URL")
	routeEngineAddress := envOrDefault("ROUTE_ENGINE_ADDRESS", "route-engine:50051")
	graphHopperURL := envOrDefault("GRAPHHOPPER_URL", "http://graphhopper:8989")
	migrationsDirectory := envOrDefault("MIGRATIONS_DIR", "/migrations")

	startupCtx, cancelStartup := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancelStartup()

	databasePool, err := pgxpool.New(startupCtx, databaseURL)
	if err != nil {
		return err
	}
	defer databasePool.Close()
	if err := database.Migrate(startupCtx, databasePool, migrationsDirectory); err != nil {
		return err
	}

	routeEngineConnection, err := grpc.NewClient(routeEngineAddress, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return err
	}
	defer routeEngineConnection.Close()

	healthService := health.NewService(
		databasePool,
		routev1.NewRouteOptimizerClient(routeEngineConnection),
		graphHopperURL,
		&http.Client{Timeout: 3 * time.Second},
		2*time.Second,
	)
	server := &http.Server{
		Addr:              listenAddress,
		Handler:           httpapi.NewRouter(healthService),
		ReadHeaderTimeout: 5 * time.Second,
		IdleTimeout:       60 * time.Second,
	}

	shutdownCtx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()
	go func() {
		<-shutdownCtx.Done()
		ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		if err := server.Shutdown(ctx); err != nil {
			slog.Error("api shutdown failed", "error", err)
		}
	}()

	slog.Info("api listening", "address", listenAddress)
	if err := server.ListenAndServe(); !errors.Is(err, http.ErrServerClosed) {
		return err
	}
	return nil
}

func envOrDefault(name, fallback string) string {
	if value := os.Getenv(name); value != "" {
		return value
	}
	return fallback
}

func requiredEnv(name string) string {
	value := os.Getenv(name)
	if value == "" {
		slog.Error("required environment variable is missing", "name", name)
		os.Exit(2)
	}
	return value
}
