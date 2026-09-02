FROM golang:1.22.9-bookworm AS build
WORKDIR /src/services/api
COPY services/api/go.mod services/api/go.sum* ./
RUN go mod download
COPY services/api/ ./
RUN CGO_ENABLED=0 GOOS=linux go build -trimpath -ldflags="-s -w" -o /out/api ./cmd/api

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=build /out/api /api
COPY migrations/ /migrations/
EXPOSE 8080
ENTRYPOINT ["/api"]
